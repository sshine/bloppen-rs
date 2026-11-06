---
id: TASK-3
title: Package the Backlog.md CLI into the flake
status: To Do
assignee: []
created_date: '2026-11-06T18:23:00Z'
labels: []
dependencies: []
priority: low
type: chore
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The upstream Backlog.md 1.55.3 CLI ships as a dynamically linked Bun binary that NixOS cannot execute directly. bin/backlog currently wraps it with nix shell and steam-run, which downloads the npm package into ~/.npm on first use and depends on unfree steam-run. A proper derivation would make the task workflow work offline and in CI.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 backlog is available on PATH inside nix develop without network access after the first build
- [ ] #2 The wrapper and steam-run workaround are removed or documented as unnecessary
- [ ] #3 just backlog task list works from a clean checkout
<!-- AC:END -->
