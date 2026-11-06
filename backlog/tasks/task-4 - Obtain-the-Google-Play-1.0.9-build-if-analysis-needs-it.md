---
id: TASK-4
title: Obtain the Google Play 1.0.9 build if analysis needs it
status: To Do
assignee: []
created_date: '2026-11-06T18:23:00Z'
labels: []
dependencies: []
priority: low
type: spike
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Play serves 1.0.9 (versionCode 36) but APKPure stops at 1.0.4 (versionCode 23). The gap is 13 version codes and may include API changes. Do not block Phase 2 on this: use 1.0.4 until a decompiled behaviour cannot be explained, then obtain an AAS token and run apkeep.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Either 1.0.9 is downloaded and its manifest and checksums are recorded, or the PRD records why 1.0.4 remains sufficient
- [ ] #2 Any new endpoints or permissions in 1.0.9 are noted in the analysis
<!-- AC:END -->
