---
id: TASK-17
title: Map domain flows for loyalty, stands, alerts, and menu
status: Done
assignee: []
created_date: '2026-11-06T20:08:00Z'
updated_date: '2026-11-06T20:14:00Z'
labels: []
dependencies: []
priority: medium
type: task
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Beyond auth, the app exposes features whose endpoints and state transitions the Rust CLI will need. This task maps the use cases and screens for the loyalty card and points, stand information or booking, alerts, menu items, and any receipt or QR features.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 re/analysis/09-domain-flows.md maps each domain to its use cases, API calls, and screens
- [x] #2 The loyalty, stand, alert, menu, and receipt/QR flows are described with their endpoints
- [x] #3 Read-only flows are separated from write operations so the CLI can start safely
- [x] #4 Unknown or unused endpoints are recorded rather than guessed
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Confirmed the domain data is HTML/WebView, not native JSON. Documented the four tabs, seven menu destinations, push intent keys, badge endpoint, scanner/camera bridge, and the gotoTab("agent") mismatch where the JS sends a tab name absent from the native Tabs enum.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrote re/analysis/09-domain-flows.md: tab and menu map, loyalty/club, stands, sales, notifications and push, find/search agent, profile/consent, scanner and pictures, a read/write split, and the unmapped website endpoints.
<!-- SECTION:FINAL_SUMMARY:END -->
