---
id: TASK-29
title: Implement read-only bloppen-mcp server
status: Done
assignee: []
created_date: '2026-11-06T22:46:00Z'
updated_date: '2026-11-06T23:01:00Z'
labels: []
dependencies:
  - TASK-27
priority: high
type: task
ordinal: 29000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The MCP server exposes the authenticated session, product search, and page access to an agent. It follows the mxroute-rs pattern: rmcp over stdio, a tool router, structured results, output limits, --list-tools, and a startup check. It is read-only; no write endpoints exist in the library yet.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 bloppen-mcp serves session_status, search_products, and fetch_page over stdio
- [x] #2 Tools have rmcp annotations and structured JSON results with output limits
- [x] #3 --list-tools prints the schemas without credentials
- [x] #4 --check authenticates and exits without serving
- [x] #5 Transport tests drive a real client against a mock server over a pipe
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Three read-only tools: bloppen_session_status, bloppen_search_products, bloppen_fetch_page. Schemas, annotations, output limits, --list-tools, --check, and a real MCP client over duplex are tested. --check was also run against the live account. The subprocess transport test proves logs stay on stderr and stdout stays framed JSON.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Delivered bloppen-mcp: a read-only rmcp stdio server with three annotated tools, structured results, output limits, schema listing, a startup check, and surface/read-tool/transport tests.
<!-- SECTION:FINAL_SUMMARY:END -->
