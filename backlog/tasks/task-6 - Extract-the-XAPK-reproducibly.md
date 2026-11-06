---
id: TASK-6
title: Extract the XAPK reproducibly
status: Done
assignee:
  - '@sshine'
created_date: '2026-11-06T18:38:00Z'
updated_date: '2026-11-06T18:40:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Phase 1 extracted the APK ad hoc. Phase 2 needs a repeatable extraction step so a fresh checkout can regenerate the split APKs from the downloaded XAPK, and so a future 1.0.9 download uses the same path. The script must handle both a plain APK and an XAPK (a ZIP of splits).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 re/apk_extract.sh accepts an XAPK or APK path and an output directory
- [x] #2 It extracts the outer archive, copies every inner APK, and writes a split manifest
- [x] #3 Re-running on the same input is idempotent
- [x] #4 Missing or corrupt input fails with a clear message and a non-zero exit status
- [x] #5 The script runs inside nix develop with only declared tools
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Write re/apk_extract.sh with XAPK/APK detection, integrity check, manifest-aware split copying, and a deterministic splits.tsv.
2. Test it twice against the 1.0.4 XAPK into a temporary directory and compare output checksums.
3. Run it against apk/extracted so the repository layout matches the script.
4. Document the script in re/prd.md if the layout changes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified against the 1.0.4 XAPK: two independent runs produced identical splits.tsv and checksums; a duplicate rerun into the same directory succeeded; a missing file and a 4 KiB random file both exited 1 with a clear message; a plain APK produced base.apk with id base.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added re/apk_extract.sh, which turns an APK or XAPK into base.apk plus its splits, records sha256/size/id in splits.tsv, and removes only files it previously wrote. Added findutils/coreutils to the devshell for portable find -printf and wc. Documented the script in re/README.md.
<!-- SECTION:FINAL_SUMMARY:END -->
