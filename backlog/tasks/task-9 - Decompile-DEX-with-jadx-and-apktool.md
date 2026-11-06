---
id: TASK-9
title: Decompile DEX with jadx and apktool
status: Done
assignee:
  - '@sshine'
created_date: '2026-11-06T18:38:00Z'
updated_date: '2026-11-06T18:50:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Phase 3 needs source-like output to map architecture, endpoints, auth, and data models. jadx is the primary Java/Kotlin decompiler; apktool supplies decoded resources and smali for anything jadx gets wrong. Output is generated and must stay out of git.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 re/decompile.sh runs jadx on every DEX file and apktool on base.apk
- [x] #2 Output lands under re/decompiled/jadx and re/decompiled/apktool and remains gitignored
- [x] #3 Fatal decompilation failures are documented with the affected classes or files
- [x] #4 re/analysis/03-decompilation.md records tool versions, commands, timings, and known gaps
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Write re/decompile.sh with jadx (source only) and apktool (resources + smali), logs, and timings.
2. Run it against apk/extracted/base.apk into re/decompiled/.
3. Count outputs and scan the jadx log for fatal errors.
4. Write re/analysis/03-decompilation.md with versions, commands, timings, output sizes, and gaps.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
jadx 1.5.6 exits 3 with 228 errors but produces 19,734 Java files; only three app-package files (all Compose screens) contain error comments. apktool 3.0.3 succeeds with 80 unresolved resource-name warnings. The script now treats partial jadx output as a warning when sources exist, and only fails when jadx produces no sources or apktool fails.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added re/decompile.sh and generated the gitignored jadx/apktool trees. Documented tool versions, command, 51 s + 11 s timings, output sizes, error categories, the three affected Compose files, apktool warnings, the decompiled package structure, and the Phase 3 starting classes (BoerneloppenAPI, RetrofitBoerneloppenApi, interceptors, LoginTokenModel, VmcmsCookie use cases).
<!-- SECTION:FINAL_SUMMARY:END -->
