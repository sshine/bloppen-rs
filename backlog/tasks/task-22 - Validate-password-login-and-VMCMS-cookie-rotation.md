---
id: TASK-22
title: Validate password login and VMCMS cookie rotation
status: To Do
assignee: []
created_date: '2026-11-06T20:29:00Z'
labels: []
dependencies:
  - TASK-20
  - TASK-21
priority: high
type: task
ordinal: 22000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The Phase 3 auth document is static analysis. This task exercises GET /users/login, POST /users/login with email/password, and an authenticated page, and records whether the vmcms cookie rotates and how a failed login is signalled.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 GET /users/login returns a vmcms cookie before login
- [ ] #2 POST /users/login with the real account reaches an authenticated page
- [ ] #3 The probe reports whether the cookie value changed after login
- [ ] #4 A failed login is distinguished from a successful one without printing the password or cookie
- [ ] #5 re/analysis/11-live-login.md records the observed status codes, redirects, cookie names, and failures
<!-- AC:END -->
