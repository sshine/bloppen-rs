---
id: TASK-15
title: Trace username/password login and the VMCMS cookie exchange
status: To Do
assignee: []
created_date: '2026-11-06T20:08:00Z'
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
- [ ] #1 re/analysis/07-auth.md documents the login request (path, method, body fields, headers) and response
- [ ] #2 It identifies where the VMCMS cookie is obtained, its name/domain, and how it is attached to later requests
- [ ] #3 It documents the login token model, storage location, refresh or re-login behaviour, and logout
- [ ] #4 It states whether the token is sent as a header, query parameter, cookie, or a combination
- [ ] #5 It includes a sequence diagram or ordered step list from credentials to an authenticated request
- [ ] #6 It records what is still unknown and what needs live validation in Phase 4
<!-- AC:END -->
