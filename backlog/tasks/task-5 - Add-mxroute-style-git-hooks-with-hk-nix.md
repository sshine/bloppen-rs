---
id: TASK-5
title: Add mxroute-style git hooks with hk-nix
status: To Do
assignee: []
created_date: '2026-11-06T18:23:00Z'
labels: []
dependencies: []
priority: low
type: chore
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
mxroute-rs runs treefmt and betterleaks in pre-commit and deadnix, clippy, and readme checks in pre-push through hk-nix. bloppen currently relies on just fmt-check, so a commit can land unformatted. Add the hook module once the Rust crates exist, or earlier for treefmt alone.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 pre-commit runs treefmt and fails on unformatted files
- [ ] #2 pre-push runs the checks that exist at that phase
- [ ] #3 nix flake check still passes
<!-- AC:END -->
