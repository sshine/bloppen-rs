---
id: TASK-23
title: Validate getLoginToken and loginWithToken
status: Done
assignee: []
created_date: '2026-11-06T20:29:00Z'
updated_date: '2026-11-06T22:35:00Z'
labels: []
dependencies:
  - TASK-22
priority: high
type: task
ordinal: 23000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The native login token is the biometric re-entry credential. This task calls getLoginToken.json with the authenticated vmcms cookie, records the success shape, then uses loginWithToken.json with a fresh cookie jar to confirm the token establishes an authenticated session.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 getLoginToken.json returns status success and a non-empty token with the real session
- [x] #2 The response fields and their nullability match or correct re/analysis/08-data-models.md
- [x] #3 loginWithToken.json accepts the token and produces an authenticated page in a fresh cookie jar
- [x] #4 The probe reports token presence and length only, never the token value
- [x] #5 re/analysis/12-live-token.md records the observed behaviour and any discrepancies
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Live: getLoginToken.json 200, status success, token 49 bytes, no message. loginWithToken.json with a fresh jar 200, status success, message Du er nu logget ind, no token, no new cookie; the existing vmcms session becomes authenticated. Updated 08-data-models.md with the observed shapes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Validated getLoginToken and loginWithToken. The token is a re-entry credential that marks the existing vmcms session as logged in; it is not a bearer token and does not rotate the cookie. Documented the observed bodies, lengths, and Phase 3 corrections in re/analysis/12-live-token.md.
<!-- SECTION:FINAL_SUMMARY:END -->
