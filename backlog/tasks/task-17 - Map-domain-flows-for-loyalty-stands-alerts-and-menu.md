---
id: TASK-17
title: Map domain flows for loyalty, stands, alerts, and menu
status: To Do
assignee: []
created_date: '2026-11-06T20:08:00Z'
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
- [ ] #1 re/analysis/09-domain-flows.md maps each domain to its use cases, API calls, and screens
- [ ] #2 The loyalty, stand, alert, menu, and receipt/QR flows are described with their endpoints
- [ ] #3 Read-only flows are separated from write operations so the CLI can start safely
- [ ] #4 Unknown or unused endpoints are recorded rather than guessed
<!-- AC:END -->
