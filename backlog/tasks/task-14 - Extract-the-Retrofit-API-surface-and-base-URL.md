---
id: TASK-14
title: Extract the Retrofit API surface and base URL
status: To Do
assignee: []
created_date: '2026-11-06T20:08:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 14000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The Rust client needs a complete endpoint catalogue, not a few URLs found by grep. This task extracts the Retrofit interfaces, HTTP methods, paths, query/body parameters, headers, interceptors, and the base URL(s) the app uses, including any environment switching.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 re/analysis/06-api-surface.md lists every Retrofit method with HTTP verb, path, parameters, and response type
- [ ] #2 The base URL(s), environment switching, and any BuildConfig or constants that select them are documented
- [ ] #3 Request headers and interceptors are documented, including auth, accept, user-agent, and error handling
- [ ] #4 Endpoints that work without authentication are marked
- [ ] #5 Any endpoint that is not reachable from the decompiled code is called out
<!-- AC:END -->
