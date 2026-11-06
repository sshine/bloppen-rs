---
id: TASK-30
title: Add live tests and showcase for API and CLI
status: Done
assignee: []
created_date: '2026-11-06T22:46:00Z'
updated_date: '2026-11-06T23:01:00Z'
labels: []
dependencies:
  - TASK-27
  - TASK-28
priority: high
type: task
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The live suite must prove login, reauth-relevant session status, search, page access, and logout against the real account. The CLI showcase should exercise every read-only command so regressions are visible. Both run through secretspec.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Live tests are #[ignore] and run only via just live-test with secretspec
- [x] #2 Live tests cover sign-in, session status, search, page fetch, token presence, and logout
- [x] #3 just showcase runs search and page read-only commands against the live session
- [x] #4 Unit, wiremock, CLI, MCP, and live suites pass
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Live suite: sign-in, status/expiry, public search (2,413 Lego results, 25 pages), authenticated home (207,624 bytes), token re-entry in a fresh jar, logout and file removal. `just showcase` runs login/status/search/page/token. `just ci` passes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the ignored live suite and the read-only CLI showcase, both driven by secretspec. All unit, wiremock, CLI, MCP, and live suites pass; just ci is green.
<!-- SECTION:FINAL_SUMMARY:END -->
