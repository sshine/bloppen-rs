#!/usr/bin/env bash
# Extract an APK or XAPK into its split APKs.
#
# An XAPK is a ZIP whose entries include one or more *.apk files plus a
# manifest.json describing which split is the base. A plain APK is copied as
# base.apk. The output directory always ends up with:
#
#   base.apk            the base APK, ready for jadx/apktool
#   <split>.apk         every other split under its original name
#   splits.tsv          sha256, size, file, and split id, sorted by id
#   xapk-manifest.json  the XAPK manifest, when the input had one
#   icon.png            the XAPK icon, when the input had one
#
# Only files named by the previous splits.tsv are removed on a rerun, so the
# script is idempotent without deleting unrelated files in the output
# directory.
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: re/apk_extract.sh <app.apk|app.xapk> [output-directory]

The output directory defaults to apk/extracted. If the input is omitted, the
single *.apk or *.xapk under apk/raw is used.
EOF
  exit 64
}

die() {
  printf 'apk_extract: %s\n' "$*" >&2
  exit 1
}

require() {
  command -v "$1" >/dev/null 2>&1 || die "required tool not found: $1 (run inside nix develop)"
}

require unzip
require jq
require sha256sum

if [[ $# -gt 2 ]]; then
  usage
fi

input=${1:-}
if [[ -z $input ]]; then
  mapfile -t candidates < <(find apk/raw -maxdepth 1 -type f \( -iname '*.apk' -o -iname '*.xapk' \) 2>/dev/null | sort)
  if ((${#candidates[@]} != 1)); then
    die "expected exactly one APK/XAPK under apk/raw, found ${#candidates[@]}; pass the input path explicitly"
  fi
  input=${candidates[0]}
fi

out=${2:-apk/extracted}

[[ -f $input ]] || die "no such file: $input"
[[ -r $input ]] || die "cannot read: $input"

# A corrupt archive must fail before anything is copied over a previous run.
unzip -tq "$input" >/dev/null 2>&1 || die "not a valid ZIP/APK/XAPK: $input"

mkdir -p "$out"

# Remove only the outputs the previous run recorded, plus the fixed names.
if [[ -f $out/splits.tsv ]]; then
  while IFS=$'\t' read -r _sha _size file _id; do
    [[ $file == file ]] && continue
    rm -f -- "$out/$file"
  done <"$out/splits.tsv"
fi
rm -f -- "$out/base.apk" "$out/splits.tsv" "$out/xapk-manifest.json" "$out/icon.png"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

mapfile -t inner_apks < <(unzip -Z1 "$input" | grep -Ei '\.apk$' || true)

if ((${#inner_apks[@]} == 0)); then
  # Plain APK: the input is the base.
  cp -f -- "$input" "$out/base.apk"
  {
    printf 'sha256\tsize\tfile\tid\n'
    printf '%s\t%s\t%s\t%s\n' \
      "$(sha256sum "$out/base.apk" | cut -d' ' -f1)" \
      "$(wc -c <"$out/base.apk" | tr -d '[:space:]')" \
      "base.apk" \
      "base"
  } >"$out/splits.tsv"
  printf 'extracted plain APK -> %s\n' "$out"
  exit 0
fi

# XAPK: expand into a scratch directory, then copy the splits by declared id.
unzip -q -o "$input" -d "$tmp"

manifest=$(find "$tmp" -maxdepth 2 -type f -name manifest.json | head -n 1 || true)

declare -A id_by_file=()
if [[ -n $manifest ]]; then
  while IFS=$'\t' read -r file id; do
    [[ -n $file ]] || continue
    id_by_file["$file"]=$id
  done < <(jq -r '.split_apks[]? | [.file, .id] | @tsv' "$manifest")
fi

# Walk the expanded tree so nested APK names are handled, but keep the basename
# that the manifest uses. XAPKs put splits at the archive root in practice.
mapfile -t found < <(find "$tmp" -type f -iname '*.apk' -printf '%P\n' | sort)

declare -a rows=()
base_written=0
for rel in "${found[@]}"; do
  base=$(basename "$rel")
  id=${id_by_file[$base]:-$(basename "$base" .apk)}
  cp -f -- "$tmp/$rel" "$out/$base"
  if [[ $id == base ]]; then
    if [[ $base != base.apk ]]; then
      ln -f -- "$out/$base" "$out/base.apk"
    fi
    base_written=1
  fi
  size=$(wc -c <"$out/$base" | tr -d '[:space:]')
  printf -v row '%s\t%s\t%s\t%s' "$(sha256sum "$out/$base" | cut -d' ' -f1)" "$size" "$base" "$id"
  rows+=("$row")
done

if ((base_written == 0)); then
  # A malformed or minimal XAPK may not label a base. The largest APK is the
  # safest fallback, and saying so is better than silently picking the first.
  largest=$(find "$tmp" -type f -iname '*.apk' -printf '%s\t%P\n' | sort -rn | head -n 1 | cut -f2-)
  [[ -n $largest ]] || die "no APK entries found in $input"
  base=$(basename "$largest")
  ln -f -- "$out/$base" "$out/base.apk"
  base_written=1
  printf 'apk_extract: no split declared as base; using largest APK %s\n' "$base" >&2
  # Replace the fallback row's id so splits.tsv agrees with base.apk.
  for i in "${!rows[@]}"; do
    IFS=$'\t' read -r sha size file id <<<"${rows[$i]}"
    if [[ $file == "$base" ]]; then
      printf -v "rows[$i]" '%s\t%s\t%s\tbase' "$sha" "$size" "$file"
    fi
  done
fi

{
  printf 'sha256\tsize\tfile\tid\n'
  printf '%s\n' "${rows[@]}" | sort -t$'\t' -k4,4 -k3,3
} >"$out/splits.tsv"

if [[ -f $tmp/manifest.json ]]; then
  cp -f -- "$tmp/manifest.json" "$out/xapk-manifest.json"
fi
if [[ -f $tmp/icon.png ]]; then
  cp -f -- "$tmp/icon.png" "$out/icon.png"
fi

printf 'extracted XAPK -> %s (%d APKs)\n' "$out" "${#rows[@]}"
