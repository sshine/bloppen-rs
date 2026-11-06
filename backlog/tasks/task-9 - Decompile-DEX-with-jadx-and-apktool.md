---
id: TASK-9
title: Decompile DEX with jadx and apktool
status: To Do
assignee: []
created_date: '2026-11-06T18:38:00Z'
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
- [ ] #1 re/decompile.sh runs jadx on every DEX file and apktool on base.apk
- [ ] #2 Output lands under re/decompiled/jadx and re/decompiled/apktool and remains gitignored
- [ ] #3 Fatal decompilation failures are documented with the affected classes or files
- [ ] #4 re/analysis/03-decompilation.md records tool versions, commands, timings, and known gaps
<!-- AC:END -->
