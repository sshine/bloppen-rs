---
id: TASK-33
title: Add CLI product and shop write commands
status: Done
assignee: []
created_date: '2026-11-07T00:02:00Z'
updated_date: '2026-11-07T00:03:00Z'
labels: []
dependencies:
  - TASK-32
priority: high
type: task
ordinal: 33000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The CLI now needs the human/scriptable form of the write surface, with a guard before destructive operations.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 product list/add/active/inactive/delete/move/discount/note/print/unprint/upload are implemented
- [x] #2 shop activate/deactivate are implemented
- [x] #3 product delete requires --yes before any request
- [x] #4 Human and JSON output cover action and save responses
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
CLI live-read commands still pass. Write commands were compiled and tested with mock request-shape tests; product delete refuses without --yes before any request. The live account has no active booking, so a successful live write was not possible.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added CLI product list/add/active/inactive/delete/move/discount/note/print/unprint/upload and shop activate/deactivate, with human/JSON output and the --yes delete guard.
<!-- SECTION:FINAL_SUMMARY:END -->
