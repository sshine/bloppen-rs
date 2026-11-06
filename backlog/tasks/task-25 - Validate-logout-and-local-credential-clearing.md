---
id: TASK-25
title: Validate logout and local credential clearing
status: To Do
assignee: []
created_date: '2026-11-06T20:29:00Z'
labels: []
dependencies:
  - TASK-22
priority: medium
type: task
ordinal: 25000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The app leaves the vmcms cookie and login token on disk after logout. This task verifies the server-side /users/logout behaviour and defines the Rust client cleanup contract, even though the app itself does not implement it correctly.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 GET /users/logout is recorded with its status, redirect, and effect on a later authenticated page request
- [ ] #2 The Rust cleanup contract clears the cookie jar and token file before or after the logout call
- [ ] #3 re/analysis/14-live-logout.md records the observed server behaviour and the chosen client behaviour
<!-- AC:END -->
