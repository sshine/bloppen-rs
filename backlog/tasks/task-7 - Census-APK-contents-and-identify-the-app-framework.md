---
id: TASK-7
title: Census APK contents and identify the app framework
status: To Do
assignee: []
created_date: '2026-11-06T18:38:00Z'
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
- [ ] #1 File-type counts and the largest entries are recorded in re/analysis/01-file-census.md
- [ ] #2 DEX count, native library ABIs, asset trees, and resource tables are identified
- [ ] #3 The framework is confirmed as native Android, Flutter, React Native, Xamarin, or something else, with evidence
- [ ] #4 Obfuscation indicators such as mapping files and short class names are checked
<!-- AC:END -->
