---
id: TASK-25
title: Validate logout and local credential clearing
status: Done
assignee: []
created_date: '2026-11-06T20:29:00Z'
updated_date: '2026-11-06T22:45:00Z'
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
- [x] #1 GET /users/logout is recorded with its status, redirect, and effect on a later authenticated page request
- [x] #2 The Rust cleanup contract clears the cookie jar and token file before or after the logout call
- [x] #3 re/analysis/14-live-logout.md records the observed server behaviour and the chosen client behaviour
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Live: GET /users/logout 200 final /users/login; cookies are not cleared; home redirects to login; getLoginToken returns 200 status error "Log ind...". vmcms Max-Age is 604800s and login does not refresh it. Documented the auth-expiry signatures and the proactive/reactive reauth contract.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @sshine
created: 2026-11-06T22:35:00Z

---

Not run during the first live pass to avoid invalidating the developer other sessions. The probe supports `just probe --logout`; run it only when the user agrees to a server-side logout.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Validated logout and the session-expiry signals. There is no 401/403: expired or logged-out sessions show as a login redirect on pages and status=error on getLoginToken. The Rust client should check the vmcms expiry, re-login once on AuthExpired, and clear local state on logout. Evidence in re/analysis/14-live-logout.md.
<!-- SECTION:FINAL_SUMMARY:END -->
