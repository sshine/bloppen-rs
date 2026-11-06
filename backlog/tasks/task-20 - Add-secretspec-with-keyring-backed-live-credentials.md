---
id: TASK-20
title: Add secretspec with keyring-backed live credentials
status: Done
assignee:
  - '@sshine'
created_date: '2026-11-06T20:29:00Z'
updated_date: '2026-11-06T20:34:00Z'
labels: []
dependencies: []
priority: high
type: chore
ordinal: 20000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Phase 4 must use the developer real account without committing credentials. The nemlig-rs pattern declares the secret names in secretspec.toml, adds secretspec to the devshell, and runs live probes through `secretspec run`; the developer configures the keyring provider once. This task sets up the same pattern for bloppen and documents how to populate the keyring.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 secretspec.toml declares BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD for the default profile
- [x] #2 nix/secretspec.nix adds secretspec to the devshell
- [x] #3 just secrets-check and just probe route through secretspec run/check with a documented reason
- [x] #4 The global provider is keyring and the project does not commit any secret value
- [x] #5 secretspec check reports the exact missing names without prompting or leaking values
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add secretspec.toml with BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD.
2. Add nix/secretspec.nix to the devshell and watch it in .envrc.
3. Add just secrets-check, secrets-set, and probe recipes using `secretspec run --reason`.
4. Document the keyring setup in AGENTS.md and the PRD.
5. Run `secretspec check` and record the missing names without values.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The global secretspec config reports provider=keyring. `secretspec check --no-prompt --json` lists BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD as missing_required and BORNELOPPEN_TEST_WRITES as missing_optional, with no values. No secret was written to the repository.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added secretspec.toml for bloppen-rs, nix/secretspec.nix in the devshell, the .envrc watch, and just recipes for secrets-set, secrets-check, and probe. Live credentials resolve through the keyring provider via secretspec run.
<!-- SECTION:FINAL_SUMMARY:END -->
