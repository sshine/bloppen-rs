---
id: TASK-27
title: Implement bloppen-api with session persistence and reauth
status: Done
assignee: []
created_date: '2026-11-06T22:46:00Z'
updated_date: '2026-11-06T23:01:00Z'
labels: []
dependencies:
  - TASK-26
priority: high
type: task
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The live validation showed that vmcms is a seven-day session cookie that login does not refresh, and that expiry appears as a 200 login redirect or a status=error token response, not a 401. The library must own the cookie jar, persist it with 0600 permissions, expose the expiry, and re-login once when an authenticated request fails. It also needs typed search results and raw page access for the HTML domains.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 bloppen-api builds with Client, ClientBuilder, Credentials, Secret, Session, and typed errors
- [x] #2 Sign-in uses POST /users/login with email/password, follows redirects, and validates the authenticated page
- [x] #3 The session file stores cookies and the optional token with 0600 permissions and restores them
- [x] #4 The client records vmcms expiry and re-authenticates when it is within a safety margin
- [x] #5 An authenticated request that lands on /users/login or returns status=error triggers one re-login and retry
- [x] #6 search.json is typed and works without authentication
- [x] #7 Page access is same-host only and returns status, final URL, title, and HTML
- [x] #8 Unit and wiremock tests cover login, expiry, reauth, search, page auth failure, and session persistence
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Wiremock tests cover login, rejected password, unauthenticated status, reactive reauth after a login redirect, proactive reauth inside the margin, session restore, logout cleanup, foreign-origin rejection, search parsing, decode errors, and public/private pages. Live tests cover sign-in, status, search, page, token re-entry, and logout. The session file is 0600 and atomic; expiry handles both Expires and Max-Age.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Delivered bloppen-api: Client/ClientBuilder/Credentials/Secret, password and token login, session persistence, expiry-aware and reactive reauth, typed search, same-origin page access, and typed errors. Unit, wiremock, doc, and live tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
