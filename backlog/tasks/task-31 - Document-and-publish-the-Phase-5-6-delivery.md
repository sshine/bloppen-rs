---
id: TASK-31
title: Document and publish the Phase 5/6 delivery
status: Done
assignee: []
created_date: '2026-11-06T22:46:00Z'
updated_date: '2026-11-06T23:01:00Z'
labels: []
dependencies:
  - TASK-28
  - TASK-29
  - TASK-30
priority: medium
type: task
ordinal: 31000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The repository needs a README that explains the API, CLI, MCP, session/expiry behaviour, and credential setup, plus updated analysis notes and a pushed commit.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 README covers install, auth, search, page fetch, MCP configuration, and reauth semantics
- [x] #2 PRD and backlog reflect the delivered library, CLI, and MCP server
- [x] #3 just fmt-check, just lint, just test, and just ci pass
- [x] #4 Changes are committed and pushed to github:sshine/bloppen-rs
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
README covers build, secretspec/keyring credentials, CLI commands, MCP configuration and tools, library usage, session/expiry semantics, tests, layout, and security. PRD 6.5 records the delivery. just ci passes. Commit 5359d12 is pushed to origin/main.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Documented and published the Phase 5/6 delivery: API library, CLI, and read-only MCP server, with the session/expiry design in README and re/analysis/15-session-robustness.md. Full CI passes and the branch is pushed.
<!-- SECTION:FINAL_SUMMARY:END -->
