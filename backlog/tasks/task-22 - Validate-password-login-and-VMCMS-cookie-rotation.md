---
id: TASK-22
title: Validate password login and VMCMS cookie rotation
status: Done
assignee: []
created_date: '2026-11-06T20:29:00Z'
updated_date: '2026-11-06T22:35:00Z'
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
- [x] #1 GET /users/login returns a vmcms cookie before login
- [x] #2 POST /users/login with the real account reaches an authenticated page
- [x] #3 The probe reports whether the cookie value changed after login
- [x] #4 A failed login is distinguished from a successful one without printing the password or cookie
- [x] #5 re/analysis/11-live-login.md records the observed status codes, redirects, cookie names, and failures
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Live: GET /users/login 200 with vmcms 32 bytes; POST /users/login 200 final /mit-boerneloppen/forside authenticated; vmcms unchanged; new cookies hasLogin, HttpAccelerator, ShopWishlist_UserUUID. Fake account: POST 500, final /users/login, home redirects to login, authenticated=false. Probe now prints clean errors without a backtrace.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Validated password login against the real account. The cookie jar is the session; vmcms is stable and does not rotate. Documented the cookie names, lengths, success criteria, and the fake-account failure path in re/analysis/11-live-login.md.
<!-- SECTION:FINAL_SUMMARY:END -->
