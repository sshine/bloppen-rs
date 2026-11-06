---
id: TASK-23
title: Validate getLoginToken and loginWithToken
status: To Do
assignee: []
created_date: '2026-11-06T20:29:00Z'
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
- [ ] #1 getLoginToken.json returns status success and a non-empty token with the real session
- [ ] #2 The response fields and their nullability match or correct re/analysis/08-data-models.md
- [ ] #3 loginWithToken.json accepts the token and produces an authenticated page in a fresh cookie jar
- [ ] #4 The probe reports token presence and length only, never the token value
- [ ] #5 re/analysis/12-live-token.md records the observed behaviour and any discrepancies
<!-- AC:END -->
