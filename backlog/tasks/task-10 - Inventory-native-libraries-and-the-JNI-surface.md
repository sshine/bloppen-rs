---
id: TASK-10
title: Inventory native libraries and the JNI surface
status: To Do
assignee: []
created_date: '2026-11-06T18:38:00Z'
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
- [ ] #1 Every lib/<abi>/*.so is listed with architecture and size, or the document states none exist with evidence
- [ ] #2 Exported JNI symbols and interesting strings are summarized in re/analysis/04-native-libs.md
- [ ] #3 Any certificate pinning or crypto-related native code is called out
<!-- AC:END -->
