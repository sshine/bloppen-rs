#!/usr/bin/env python3
"""Produce a Markdown census of an APK.

The report answers the questions Phase 2 needs before decompiling: what file
types dominate, which DEX files exist, whether there is native or managed code,
which UI framework is in use, and whether the code looks obfuscated.

Usage:
    re/apk_census.py apk/extracted/base.apk > re/analysis/01-file-census.md
"""

from __future__ import annotations

import collections
import hashlib
import re
import sys
import zipfile
from pathlib import Path

# Names and DEX strings that identify a UI framework. Absence of all of them is
# evidence for a native Android app, but the DEX package checks are stronger
# than filenames, so both are reported.
FRAMEWORK_NAMES = {
    "Flutter": ["libflutter.so", "libapp.so", "flutter_assets/"],
    "React Native": ["libreactnativejni.so", "index.android.bundle", "libhermes.so"],
    "Xamarin / .NET": [
        "assemblies/",
        "libmonodroid.so",
        "libxamarin-app.so",
        "libmonosgen-2.0.so",
    ],
    "Capacitor": ["assets/capacitor.config.json", "assets/public/"],
    "Cordova": ["assets/www/", "cordova.js"],
    "Ionic": ["assets/www/", "assets/ionic/"],
}

FRAMEWORK_DEX_MARKERS = {
    "Flutter": [b"Lio/flutter/", b"Lio/flutter/embedding/"],
    "React Native": [b"Lcom/facebook/react/", b"Lcom/facebook/hermes/"],
    "Xamarin / .NET": [b"Lmono/android/", b"Lmono/", b"Lmicrosoft/maui/"],
    "Capacitor": [b"Lcom/getcapacitor/"],
    "Cordova": [b"Lorg/apache/cordova/"],
}

JAVA_PACKAGE_PREFIXES = (
    "java/",
    "javax/",
    "android/",
    "androidx/",
    "kotlin/",
    "kotlinx/",
    "com/google/",
    "com/android/",
    "org/jetbrains/",
    "dalvik/",
)


def human_size(n: int) -> str:
    for unit in ("B", "KiB", "MiB", "GiB"):
        if n < 1024 or unit == "GiB":
            return f"{n:.0f} {unit}" if unit == "B" else f"{n:.1f} {unit}"
        n /= 1024
    raise AssertionError("unreachable")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_uleb128(data: bytes, offset: int) -> tuple[int, int]:
    """Decode a DEX uleb128 value, returning (value, next_offset)."""
    result = 0
    shift = 0
    while True:
        byte = data[offset]
        offset += 1
        result |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return result, offset
        shift += 7


def dex_type_descriptors(data: bytes) -> list[bytes]:
    """Read the type descriptors from the DEX type_ids table.

    Scanning raw DEX bytes with a regex produces false matches across string
    boundaries. Parsing string_ids and type_ids is barely more code and keeps
    the obfuscation counts honest.
    """
    if len(data) < 0x70 or data[:4] != b"dex\n":
        raise ValueError("not a DEX file")

    string_ids_size = int.from_bytes(data[56:60], "little")
    string_ids_off = int.from_bytes(data[60:64], "little")
    type_ids_size = int.from_bytes(data[64:68], "little")
    type_ids_off = int.from_bytes(data[68:72], "little")

    strings: list[bytes] = []
    for index in range(string_ids_size):
        offset = int.from_bytes(data[string_ids_off + 4 * index : string_ids_off + 4 * index + 4], "little")
        _, cursor = read_uleb128(data, offset)
        end = data.index(b"\x00", cursor)
        strings.append(data[cursor:end])

    descriptors: list[bytes] = []
    for index in range(type_ids_size):
        offset = type_ids_off + 4 * index
        string_index = int.from_bytes(data[offset : offset + 4], "little")
        descriptor = strings[string_index]
        if descriptor.startswith(b"L") and descriptor.endswith(b";"):
            descriptors.append(descriptor)
    return descriptors


def is_standard_package(package: str) -> bool:
    """Whether a descriptor's package belongs to the JDK, Android, or Kotlin."""
    return any(
        package == prefix.rstrip("/") or package.startswith(prefix)
        for prefix in JAVA_PACKAGE_PREFIXES
    )


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <base.apk>", file=sys.stderr)
        return 64

    apk = Path(sys.argv[1])
    if not apk.is_file():
        print(f"census: no such file: {apk}", file=sys.stderr)
        return 1

    with zipfile.ZipFile(apk) as zf:
        infos = [info for info in zf.infolist() if not info.is_dir()]
        names = [info.filename for info in infos]

        ext_counts: collections.Counter[str] = collections.Counter()
        ext_bytes: collections.Counter[str] = collections.Counter()
        for info in infos:
            name = info.filename
            suffix = name.rsplit(".", 1)[1].lower() if "." in name.rsplit("/", 1)[-1] else "(none)"
            ext_counts[suffix] += 1
            ext_bytes[suffix] += info.file_size

        top_level: collections.Counter[str] = collections.Counter()
        for name in names:
            top_level[name.split("/", 1)[0] if "/" in name else "(root)"] += 1

        dex_infos = [i for i in infos if re.fullmatch(r"classes\d*\.dex", i.filename)]
        native_infos = [i for i in infos if i.filename.startswith("lib/")]
        asset_infos = [i for i in infos if i.filename.startswith("assets/")]
        res_infos = [i for i in infos if i.filename.startswith("res/")]
        managed_infos = [
            i
            for i in infos
            if i.filename.startswith("assemblies/")
            or i.filename.lower().endswith(".dll")
            or i.filename.lower().endswith(".exe")
        ]

        # Read DEX files once for framework and obfuscation evidence.
        dex_blobs = [zf.read(i) for i in dex_infos]
        dex_bytes = b"".join(dex_blobs)

        name_markers = {
            framework: [m for m in markers if any(m in n for n in names)]
            for framework, markers in FRAMEWORK_NAMES.items()
        }
        dex_markers = {
            framework: [m.decode() for m in markers if m in dex_bytes]
            for framework, markers in FRAMEWORK_DEX_MARKERS.items()
        }

    print(f"# File census: `{apk.name}`\n")
    print(f"- Size: {human_size(apk.stat().st_size)} ({apk.stat().st_size:,} bytes)")
    print(f"- SHA-256: `{sha256(apk)}`")
    print(f"- Entries: {len(infos):,} files")
    print()

    print("## Top-level layout\n")
    print("| Top-level path | Files |")
    print("| --- | ---: |")
    for name, count in top_level.most_common():
        print(f"| `{name}` | {count:,} |")
    print()

    print("## File types\n")
    print("| Extension | Files | Uncompressed size |")
    print("| --- | ---: | ---: |")
    for ext, count in ext_counts.most_common(20):
        print(f"| `.{ext}` | {count:,} | {human_size(ext_bytes[ext])} |")
    print()

    print("## Largest entries\n")
    print("| Size | Compressed | Entry |")
    print("| ---: | ---: | --- |")
    for info in sorted(infos, key=lambda i: i.file_size, reverse=True)[:20]:
        print(
            f"| {human_size(info.file_size)} | {human_size(info.compress_size)} | "
            f"`{info.filename}` |"
        )
    print()

    print("## DEX files\n")
    if dex_infos:
        print("| File | Size |")
        print("| --- | ---: |")
        for info in sorted(dex_infos, key=lambda i: i.filename):
            print(f"| `{info.filename}` | {human_size(info.file_size)} |")
    else:
        print("No `classes*.dex` files found.")
    print()

    print("## Native libraries\n")
    if native_infos:
        by_abi: collections.Counter[str] = collections.Counter()
        for info in native_infos:
            parts = info.filename.split("/")
            by_abi[parts[1] if len(parts) > 2 else "(unknown)"] += 1
        print("| ABI | Libraries |")
        print("| --- | ---: |")
        for abi, count in sorted(by_abi.items()):
            print(f"| `{abi}` | {count:,} |")
        print()
        print("| Library | Size |")
        print("| --- | ---: |")
        for info in sorted(native_infos, key=lambda i: i.filename):
            print(f"| `{info.filename}` | {human_size(info.file_size)} |")
    else:
        print("No `lib/<abi>/*.so` entries. Native-code inspection is not required.")
    print()

    print("## Assets and resources\n")
    print(f"- `assets/`: {len(asset_infos):,} files, "
          f"{human_size(sum(i.file_size for i in asset_infos))} uncompressed")
    print(f"- `res/`: {len(res_infos):,} files, "
          f"{human_size(sum(i.file_size for i in res_infos))} uncompressed")
    print(f"- `resources.arsc`: {'present' if any(n == 'resources.arsc' for n in names) else 'absent'}")
    for sample in sorted(i.filename for i in asset_infos)[:15]:
        print(f"  - `{sample}`")
    if len(asset_infos) > 15:
        print(f"  - ... {len(asset_infos) - 15:,} more")
    print()

    print("## Framework identification\n")
    print("| Framework | Filename evidence | DEX evidence |")
    print("| --- | --- | --- |")
    for framework in FRAMEWORK_NAMES:
        filename_hits = name_markers[framework]
        dex_hits = dex_markers.get(framework, [])
        filename_text = ", ".join(f"`{h}`" for h in filename_hits) or "-"
        dex_text = ", ".join(f"`{h}`" for h in dex_hits) or "-"
        print(f"| {framework} | {filename_text} | {dex_text} |")
    print()
    detected = [
        fw
        for fw in FRAMEWORK_NAMES
        if name_markers[fw] or dex_markers.get(fw)
    ]
    if detected:
        print(f"**Detected:** {', '.join(detected)}")
    else:
        print(
            "**Detected:** no Flutter, React Native, Xamarin/.NET, Capacitor, "
            "or Cordova markers. This is a native Android app (Java/Kotlin)."
        )
    print()

    print("## Managed-code check\n")
    if managed_infos:
        print("Managed assemblies found:")
        for info in managed_infos[:20]:
            print(f"- `{info.filename}` ({human_size(info.file_size)})")
        if len(managed_infos) > 20:
            print(f"- ... {len(managed_infos) - 20:,} more")
        print("\nRun `ilspycmd` before relying on DEX-only analysis.")
    else:
        print(
            "No `assemblies/` directory, `*.dll`, or `*.exe` entries. "
            "The Xamarin/.NET path does not apply."
        )
    print()

    print("## Obfuscation indicators\n")
    descriptors: list[bytes] = []
    parse_errors = 0
    for blob in dex_blobs:
        try:
            descriptors.extend(dex_type_descriptors(blob))
        except (ValueError, IndexError) as exc:
            parse_errors += 1
            print(f"- Could not parse a DEX for type descriptors: {exc}")
    short_classes = 0
    app_classes = 0
    app_namespace = 0
    package_roots: collections.Counter[str] = collections.Counter()
    for descriptor in descriptors:
        name = descriptor[1:-1].decode("ascii", errors="replace")
        package, _, class_name = name.rpartition("/")
        if len(class_name) <= 2:
            short_classes += 1
        if name.startswith("dk/boerneloppen/"):
            app_namespace += 1
        if not is_standard_package(package):
            app_classes += 1
            package_roots[package.split("/", 1)[0]] += 1
    print(f"- Type descriptors in DEX: {len(descriptors):,}")
    print(f"- DEX files that could not be parsed: {parse_errors:,}")
    print(f"- Descriptors in the app namespace `dk/boerneloppen/`: {app_namespace:,}")
    print(f"- Descriptors outside standard Java/Android/Kotlin packages: {app_classes:,}")
    print(f"- Descriptors whose class name is 1-2 characters: {short_classes:,}")
    print(
        "- `mapping.txt` in APK: "
        + ("yes" if any(n.endswith("mapping.txt") for n in names) else "no")
    )
    if descriptors:
        ratio = short_classes / len(descriptors)
        if ratio > 0.2:
            print(
                f"- **Likely R8/ProGuard-obfuscated**: {ratio:.2%} of descriptors "
                "have 1-2 character class names."
            )
        else:
            print(
                f"- No strong obfuscation signal: {ratio:.2%} of descriptors have "
                "1-2 character class names."
            )
    else:
        ratio = 0.0
    print()
    print("Non-standard package roots (top 15):")
    print()
    for root, count in package_roots.most_common(15):
        print(f"- `{root}`: {count:,} descriptors")
    print()

    print("## Summary\n")
    if detected:
        print(f"- Framework markers present: {', '.join(detected)}.")
    else:
        print(
            "- Native Android app (Java/Kotlin); no Flutter, React Native, "
            "Xamarin/.NET, Capacitor, or Cordova markers."
        )
    dex_size = sum(i.file_size for i in dex_infos)
    print(f"- DEX: {len(dex_infos)} files, {human_size(dex_size)} uncompressed.")
    print(
        "- Native libraries: "
        + (f"{len(native_infos)} files" if native_infos else "none")
    )
    print(
        "- Managed assemblies: "
        + (f"{len(managed_infos)} files" if managed_infos else "none")
    )
    print(
        f"- Obfuscation: {ratio:.2%} short class names; mapping.txt "
        + ("present." if any(n.endswith("mapping.txt") for n in names) else "absent.")
    )
    roots = [root for root, _ in package_roots.most_common(8) if root]
    if roots:
        print(f"- Notable non-standard package roots: {', '.join(f'`{r}`' for r in roots)}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
