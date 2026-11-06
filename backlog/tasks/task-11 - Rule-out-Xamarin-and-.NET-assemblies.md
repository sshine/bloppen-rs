---
id: TASK-11
title: Rule out Xamarin and .NET assemblies
status: To Do
assignee: []
created_date: '2026-11-06T18:38:00Z'
labels: []
dependencies: []
priority: low
type: task
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The android-app-reverser skill calls for ilspycmd when an app is Xamarin-based. The Phase 1 inspection saw four DEX files and no Flutter, React Native, or Xamarin payload, but this task records the check and its evidence so the decision is not folklore.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The APK is checked for an assemblies/ directory, *.dll files, and known Xamarin runtime libraries
- [ ] #2 The result and evidence are recorded in re/analysis/01-file-census.md
- [ ] #3 No .NET decompilation work is scheduled unless assemblies are actually found
<!-- AC:END -->
