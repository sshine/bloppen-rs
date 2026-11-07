---
id: TASK-35
title: Test write operations and validate the live error path
status: Done
assignee: []
created_date: '2026-11-07T00:02:00Z'
updated_date: '2026-11-07T00:03:00Z'
labels: []
dependencies:
  - TASK-32
  - TASK-33
  - TASK-34
priority: high
type: task
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Write operations change real data, so the request shapes must be pinned with mocks and the live path must be exercised conservatively.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Wiremock tests cover save, active/delete ids, move, discount, note, print, shop, and multipart upload
- [x] #2 CLI tests cover the write surface and the --yes guard
- [x] #3 MCP tests cover tier counts, write calls, delete confirmation, and failed writes
- [x] #4 The live suite calls a safe write and records the site refusal for a past booking without failing
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
8 API write wiremock tests, 3 CLI surface tests, 6 MCP surface tests, 5 MCP write-tool tests, and the full live suite pass. Live safe write to a past booking returned 200 {"status":"error","message":"Du har ikke adgang!"}, which the client surfaced; no active booking was available for a successful live write.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Covered the write shapes with wiremock and CLI/MCP tests, and recorded the live expired-booking refusal without failing the suite. just ci and just live-test pass.
<!-- SECTION:FINAL_SUMMARY:END -->
