#!/usr/bin/env bash
# Decompile an APK into Java-like sources (jadx) and resources/smali (apktool).
#
# Output is generated and gitignored:
#
#   re/decompiled/jadx/        Java sources, one package tree per DEX class
#   re/decompiled/apktool/     decoded manifest, resources, smali, assets
#   re/decompiled/*.log        tool output, useful when a class fails
#
# Both tools are run fresh on every invocation so the result matches the input
# APK rather than a mixture of two builds.
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: re/decompile.sh [base.apk] [output-directory]

Defaults: apk/extracted/base.apk and re/decompiled.
EOF
  exit 64
}

die() {
  printf 'decompile: %s\n' "$*" >&2
  exit 1
}

require() {
  command -v "$1" >/dev/null 2>&1 || die "required tool not found: $1 (run inside nix develop)"
}

if [[ $# -gt 2 ]]; then
  usage
fi

apk=${1:-apk/extracted/base.apk}
out=${2:-re/decompiled}

[[ -f $apk ]] || die "no such file: $apk"
[[ -r $apk ]] || die "cannot read: $apk"
require jadx
require apktool

mkdir -p "$out"
rm -rf -- "$out/jadx" "$out/apktool"

# jadx defaults to a heap that is too small for a 49 MiB APK with four DEX
# files. 4 GiB is comfortable on this machine and still far below its RAM.
export JADX_OPTS=${JADX_OPTS:--Xmx4g}
export JAVA_OPTS=${JAVA_OPTS:--Xmx4g}

echo "== jadx $(jadx --version)" >&2
start=$(date +%s)
set +e
jadx --no-res -d "$out/jadx" "$apk" 2>&1 | tee "$out/jadx.log"
jadx_status=${PIPESTATUS[0]}
set -e
jadx_elapsed=$(($(date +%s) - start))
echo "== jadx finished in ${jadx_elapsed}s with status ${jadx_status}" >&2

echo "== apktool $(apktool --version)" >&2
start=$(date +%s)
set +e
apktool d -f -o "$out/apktool" "$apk" 2>&1 | tee "$out/apktool.log"
apktool_status=${PIPESTATUS[0]}
set -e
apktool_elapsed=$(($(date +%s) - start))
echo "== apktool finished in ${apktool_elapsed}s with status ${apktool_status}" >&2

echo "== results" >&2
du -sh "$out/jadx" "$out/apktool" >&2 || true
java_count=$(find "$out/jadx" -type f -name '*.java' 2>/dev/null | wc -l)
smali_count=$(find "$out/apktool" -type f -name '*.smali' 2>/dev/null | wc -l)
jadx_errors=$(grep -oE 'finished with errors, count: [0-9]+' "$out/jadx.log" | tail -n 1 | grep -oE '[0-9]+' || true)
printf 'java files: %s\n' "$java_count" >&2
printf 'smali files: %s\n' "$smali_count" >&2
printf 'jadx errors: %s\n' "${jadx_errors:-0}" >&2

# jadx commonly reports individual method failures on Kotlin coroutine and
# Compose code while still producing usable sources. Treat that as partial
# output, not a failed run, as long as sources exist; apktool has no such
# graceful-degradation mode, so its failure is fatal.
if ((jadx_status != 0 && java_count == 0)); then
  die "jadx produced no Java sources; inspect $out/jadx.log"
fi
if ((jadx_status != 0)); then
  printf 'decompile: warning: jadx reported partial errors; see %s/jadx.log and the decompilation analysis\n' "$out" >&2
fi
if ((apktool_status != 0)); then
  die "apktool failed; inspect $out/apktool.log"
fi
