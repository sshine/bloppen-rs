---
id: TASK-2
title: Acquire and verify the Børneloppen APK
status: Done
assignee: []
created_date: '2026-11-06T18:23:00Z'
updated_date: '2026-11-06T18:24:00Z'
labels: []
dependencies: []
references:
  - re/prd.md
priority: high
type: task
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Phase 1 needs a concrete APK to analyse. APKPure hosts 1.0.4 (versionCode 23) as an XAPK; Google Play serves 1.0.9 but no accessible mirror has it and apkeep requires an AAS token. The 1.0.4 XAPK was downloaded, integrity-checked, and split into base and config APKs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 XAPK and base APK pass ZIP integrity checks
- [x] #2 Manifest reports package dk.boerneloppen.boerneloppen, versionName 1.0.4, versionCode 23
- [x] #3 Base APK contains four DEX files and no Flutter, React Native, or Xamarin payload
- [x] #4 SHA-256 checksums are recorded in re/prd.md
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
XAPK sha256 d20cd98f...d037ed2d; base APK sha256 54a2a806...e55bb3. Artifacts live under apk/ and are gitignored.
<!-- SECTION:NOTES:END -->
