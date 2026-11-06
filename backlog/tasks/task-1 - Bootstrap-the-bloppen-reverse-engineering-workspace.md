---
id: TASK-1
title: Bootstrap the bloppen reverse-engineering workspace
status: Done
assignee: []
created_date: '2026-11-06T18:23:00Z'
updated_date: '2026-11-06T18:24:00Z'
labels: []
dependencies: []
priority: medium
type: chore
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The android-app-reverser skill needs a persistent PRD, agent guide, and task backlog before any analysis starts. This task created the dendritic flake modelled on mxroute-rs (flake-parts + import-tree), the PRD, AGENTS.md, .gitignore, Justfile, pinned Rust toolchain, and Backlog.md project.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 nix flake show --all-systems evaluates every output
- [x] #2 nix develop provides jadx, apktool, apkeep, aapt2, rustc, cargo, and treefmt
- [x] #3 re/prd.md and AGENTS.md describe scope, milestones, and working rules
- [x] #4 treefmt --fail-on-change passes
- [x] #5 The APK and generated output are gitignored
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Bootstrap commit 0798e35. The flake evaluates for all three systems; the x86_64-linux devshell was entered and its tools checked; treefmt and nix flake check pass.
<!-- SECTION:NOTES:END -->
