---
id: TASK-14
title: Extract the Retrofit API surface and base URL
status: Done
assignee: []
created_date: '2026-11-06T20:08:00Z'
updated_date: '2026-11-06T20:11:00Z'
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
- [x] #1 re/analysis/06-api-surface.md lists every Retrofit method with HTTP verb, path, parameters, and response type
- [x] #2 The base URL(s), environment switching, and any BuildConfig or constants that select them are documented
- [x] #3 Request headers and interceptors are documented, including auth, accept, user-agent, and error handling
- [x] #4 Endpoints that work without authentication are marked
- [x] #5 Any endpoint that is not reachable from the decompiled code is called out
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Live probe with an anonymous vmcms cookie: GET getLoginToken.json returned HTTP 200 {"status":"error","message":"Log ind med brugernavn/adgangskode først."}. This confirms the app treats the JSON status field, not the HTTP status, as the auth signal. Also found the website mobileapp.js endpoints (loginWithToken, handleBadgeCount) that are not in DEX.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrote re/analysis/06-api-surface.md: the two native Retrofit endpoints, base URL and flavors, per-call Cookie header, interceptor order, website JSON helpers, WebView page routes, the login form, authentication requirements, and live 200-with-error convention.
<!-- SECTION:FINAL_SUMMARY:END -->
