---
id: TASK-34
title: Add tiered MCP write and delete tools
status: Done
assignee: []
created_date: '2026-11-07T00:02:00Z'
updated_date: '2026-11-07T00:03:00Z'
labels: []
dependencies:
  - TASK-32
priority: high
type: task
ordinal: 34000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The MCP server defaults to reads. Writes and deletes are separate tiers so a client can enable adding without enabling destructive deletion, and buying remains absent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Default surface is the three read tools
- [x] #2 --allow-writes adds the eight non-destructive write tools
- [x] #3 --allow-writes --allow-deletes adds bloppen_delete_products, which requires confirm=true
- [x] #4 --allow-deletes without --allow-writes is refused
- [x] #5 Buying and payment tools are not served
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Default 3 tools, --allow-writes 11, both flags 12. --allow-deletes without --allow-writes exits 2. bloppen_delete_products requires confirm=true and is the only destructive tool. Buying/payment tools are absent.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the tiered MCP write surface: eight non-destructive write tools behind --allow-writes, and bloppen_delete_products behind --allow-writes --allow-deletes. Surface, tier, and transport tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
