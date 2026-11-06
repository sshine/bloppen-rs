# Native libraries and JNI surface

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`
**Base APK SHA-256:** `54a2a8062319c3d9c31f23f11a31541eec1a28ba82e8186e1c374e41fce55bb3`

## Finding

The APK contains **no native libraries and no JNI surface**.

```console
$ unzip -Z1 apk/extracted/base.apk | grep -Ei '\.so$'
(none)
$ unzip -Z1 apk/extracted/base.apk | grep -E '^lib/'
(none)
$ unzip -Z1 apk/extracted/base.apk | grep -Ei 'jni|\.so$'
(none)
```

The file census in [`01-file-census.md`](01-file-census.md) independently
reports zero `lib/<abi>/*.so` entries.

## Consequences

- There is no ABI split to analyse, no `System.loadLibrary` target to inspect,
  and no native symbol table to dump with `readelf` or `radare2`.
- Certificate pinning, signing, or crypto logic, if present, must live in the
  DEX or in configuration. Phase 3 should check:
  - `res/xml/network_security_config.xml` (trust anchors and pin sets)
  - OkHttp `CertificatePinner` usage
  - any custom `X509TrustManager`
- The `assets/dexopt/baseline.prof` and `baseline.profm` files are Android
  baseline profiles, not native code.

## Reproducing the check

Run inside `nix develop`:

```sh
unzip -Z1 apk/extracted/base.apk | grep -Ei '\.so$|^lib/'
```

A future build that adds a native dependency will show entries such as
`lib/arm64-v8a/libfoo.so`. At that point this document should be replaced with
the ABI inventory and exported-symbol summary required by the task.
