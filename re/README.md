# Reverse-engineering workspace

Everything here supports the analysis described in [`prd.md`](prd.md). The
PRD is the source of truth; this file only explains the scripts.

## Scripts

| Script | Purpose |
| --- | --- |
| `apk_extract.sh` | Expand an APK or XAPK into `base.apk`, its splits, and `splits.tsv` |
| `apk_census.py` | Produce a Markdown census of file types, DEX, native code, framework markers, and obfuscation |

Generated output lives under `re/decompiled/` and `re/extracted/` and is
gitignored. Downloaded APKs live under `apk/`, also gitignored.

```sh
# Default: the single APK/XAPK under apk/raw, output to apk/extracted
./re/apk_extract.sh

# Explicit input and output
./re/apk_extract.sh apk/raw/Børneloppen_1.0.4_APKPure.xapk apk/extracted
```
