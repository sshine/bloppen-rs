---
id: TASK-24
title: Probe authenticated pages and choose HTML or JSON
status: To Do
assignee: []
created_date: '2026-11-06T20:29:00Z'
labels: []
dependencies:
  - TASK-22
priority: high
type: task
ordinal: 24000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The app renders loyalty, stand, sale, club, profile, notifications, and menu pages as HTML. Phase 4 must decide whether the Rust client parses that HTML or whether the pages expose internal JSON endpoints that are not visible in mobileapp.js.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The probe requests each authenticated tab and menu page and records status, final URL, content type, size, and title
- [ ] #2 Any JSON response is identified by content type and top-level shape
- [ ] #3 Page scripts are inspected for internal XHR/JSON endpoints worth testing
- [ ] #4 re/analysis/13-live-pages.md states whether HTML parsing is required or a JSON path exists
<!-- AC:END -->
