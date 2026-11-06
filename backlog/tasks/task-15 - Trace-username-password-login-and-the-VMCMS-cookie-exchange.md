---
id: TASK-15
title: Trace username/password login and the VMCMS cookie exchange
status: Done
assignee: []
created_date: '2026-11-06T20:08:00Z'
updated_date: '2026-11-06T20:12:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 15000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
This is the critical path for the Rust client. The user has indicated that username/password authentication plus the VMCMS cookie is the intended approach. This task follows the login screen, use cases, API implementation, interceptors, token model, and secure storage to document the exact request/response flow and the cookie lifecycle.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 re/analysis/07-auth.md documents the login request (path, method, body fields, headers) and response
- [x] #2 It identifies where the VMCMS cookie is obtained, its name/domain, and how it is attached to later requests
- [x] #3 It documents the login token model, storage location, refresh or re-login behaviour, and logout
- [x] #4 It states whether the token is sent as a header, query parameter, cookie, or a combination
- [x] #5 It includes a sequence diagram or ordered step list from credentials to an authenticated request
- [x] #6 It records what is still unknown and what needs live validation in Phase 4
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The logout path is weaker than the repository API suggests: MainScreenViewModel.logout only writes is_logged_in=false; MainRepository.logout/LogoutUseCase would prune SharedPreferences but are not called. The saved vmcms_cookie and encrypted login_token survive logout, so biometric login can restore a session. Documented as an observed bug and a recommendation for the Rust client.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrote re/analysis/07-auth.md: the /users/login form and vmcms cookie, the JS bridge loginSuccess path, getLoginToken.json with the Cookie header, AES/GCM Keystore token storage, biometric login via loginWithToken, logout behavior, open questions, and a recommended Rust login flow.
<!-- SECTION:FINAL_SUMMARY:END -->
