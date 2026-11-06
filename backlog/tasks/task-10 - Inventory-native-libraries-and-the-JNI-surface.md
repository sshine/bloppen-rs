---
id: TASK-10
title: Inventory native libraries and the JNI surface
status: Done
assignee:
  - '@sshine'
created_date: '2026-11-06T18:38:00Z'
updated_date: '2026-11-06T18:44:00Z'
labels: []
dependencies: []
priority: medium
type: task
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Native code can hide API endpoints, signing logic, or certificate pinning that never appears in DEX. The file census will show whether lib/ exists, but Phase 2 still needs to check it explicitly and record the result.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every lib/<abi>/*.so is listed with architecture and size, or the document states none exist with evidence
- [x] #2 Exported JNI symbols and interesting strings are summarized in re/analysis/04-native-libs.md
- [x] #3 Any certificate pinning or crypto-related native code is called out
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Confirm the absence of lib/ from the zip listing, write re/analysis/04-native-libs.md with the evidence and the command to repeat the check for a future build, and note where certificate pinning would live if there is no native code.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Checked all .so entries, not just lib/: none. Documented that pinning/crypto, if present, must be found in network_security_config.xml or OkHttp in Phase 3, since there is no native code to inspect.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrote re/analysis/04-native-libs.md. The APK has no lib/ directory, no .so entries anywhere, and therefore no JNI surface; the document records the commands and points Phase 3 at the network security config and OkHttp for pinning.
<!-- SECTION:FINAL_SUMMARY:END -->
