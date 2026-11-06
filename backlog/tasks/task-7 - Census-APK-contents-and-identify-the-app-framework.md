---
id: TASK-7
title: Census APK contents and identify the app framework
status: Done
assignee:
  - '@sshine'
created_date: '2026-11-06T18:38:00Z'
updated_date: '2026-11-06T18:43:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Before decompiling, Phase 2 needs a factual inventory of what is in the APK: DEX files, native libraries, assets, resource tables, and any framework payload. This determines which decompilers and analysis paths are worth using, and gives a baseline for comparing 1.0.4 with a possible 1.0.9 build.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 File-type counts and the largest entries are recorded in re/analysis/01-file-census.md
- [x] #2 DEX count, native library ABIs, asset trees, and resource tables are identified
- [x] #3 The framework is confirmed as native Android, Flutter, React Native, Xamarin, or something else, with evidence
- [x] #4 Obfuscation indicators such as mapping files and short class names are checked
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Write re/apk_census.py to report extension counts, largest entries, DEX files, native libs, assets, framework markers, .NET/Xamarin evidence, and obfuscation indicators.
2. Run it against apk/extracted/base.apk and write re/analysis/01-file-census.md.
3. Cross-check the framework conclusion against DEX package strings.
4. Use the same report as evidence for TASK-11.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The census parses DEX string_ids/type_ids instead of regex-scanning bytes, so the 34,517 descriptor count and 0.44% short-class ratio are not inflated by cross-string matches. Framework evidence: no Flutter/RN/Xamarin/Capacitor/Cordova filenames or DEX markers; no lib/ entries; no assemblies/ or .dll/.exe. The APK is native Android.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added re/apk_census.py and generated re/analysis/01-file-census.md. The base APK has 855 entries, four DEX files (45.1 MiB uncompressed), no native libraries, no managed assemblies, and no cross-platform framework markers. It uses Retrofit2, OkHttp, Okio, Coil, and Timber by descriptor evidence. No strong obfuscation signal (0.44% short class names; no mapping.txt).
<!-- SECTION:FINAL_SUMMARY:END -->
