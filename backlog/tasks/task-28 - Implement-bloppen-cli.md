---
id: TASK-28
title: Implement bloppen-cli
status: Done
assignee: []
created_date: '2026-11-06T22:46:00Z'
updated_date: '2026-11-06T23:01:00Z'
labels: []
dependencies:
  - TASK-27
priority: high
type: task
ordinal: 28000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The CLI is the human and scriptable face of the library: auth login/status/logout/token, product search, and page fetch. It reads credentials from BORNELOPPEN_EMAIL/PASSWORD so secretspec can inject them, and uses the same persistent session file as the library.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 bloppen-cli provides auth login, auth status, auth logout, auth token, search, and page commands
- [x] #2 Credentials come from the environment and never from argv or tracked files
- [x] #3 Every command supports human-readable output and a --json mode where it makes sense
- [x] #4 The session file path defaults to the XDG data directory and is overridable
- [x] #5 Search supports a query, page, limit, and JSON output
- [x] #6 Page fetch supports a title summary and raw HTML output
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Live-tested auth login/status/logout/token, search (including pagination and limit), and page (summary and raw). Credentials come only from BORNELOPPEN_EMAIL/PASSWORD; the session path is XDG with a --session-file override. auth status after logout correctly omits the account and shows unauthenticated.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Delivered bloppen-cli with auth login/status/logout/token, search, and page commands, JSON modes, environment-only credentials, and an XDG session file. The live showcase passes.
<!-- SECTION:FINAL_SUMMARY:END -->
