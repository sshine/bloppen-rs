---
id: TASK-19
title: Review Phase 3 findings and populate Phase 4
status: In Progress
assignee:
  - '@sshine'
created_date: '2026-11-06T20:15:00Z'
updated_date: '2026-11-06T20:15:00Z'
labels: []
dependencies:
  - TASK-13
  - TASK-14
  - TASK-15
  - TASK-16
  - TASK-17
  - TASK-18
priority: high
type: task
ordinal: 19000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Phase 3 produced the architecture, API, auth, model, domain-flow, and security documents. Phase 4 is live validation: build a minimal Rust client, authenticate with the user own account, call the endpoints, and reconcile the docs with real responses. This gate consolidates the findings, records the decisions the Rust client must make, and populates Phase 4 only after the user confirms.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All six Phase 3 documents are reviewed together for contradictions and gaps
- [x] #2 The Phase 4 plan states how credentials are supplied without committing them
- [x] #3 The plan separates cookie-based page access from the optional login-token biometric path
- [x] #4 The plan states whether domain data will be read as HTML or through discovered website JSON endpoints
- [ ] #5 Phase 4 tasks are populated only after the user confirms advancing
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review the six Phase 3 documents for contradictions: architecture vs API surface, auth vs security, models vs domain flows.
2. Phase 4 will use a minimal Rust probe with reqwest and a cookie jar. Credentials come from an interactive prompt or environment variables in the developer shell; they are never written to git or chat. Session cookies and captured responses go under secrets/ with 0600 permissions.
3. The probe will (a) GET /users/login, (b) POST /users/login with email/password and follow redirects, (c) verify the authenticated home page, (d) call getLoginToken.json and capture success/error shapes, (e) test loginWithToken.json, (f) inspect the authenticated pages and their JS for internal JSON endpoints.
4. Domain data stays HTML unless an internal JSON endpoint is found and validated. Push registration is out of scope unless needed.
5. After live validation, reconcile the docs and create Phase 5 library tasks.
<!-- SECTION:PLAN:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @sshine
created: 2026-11-06T20:15:00Z

---

Phase 3 gate review complete. The six documents are consistent: the native API is two cookie-authenticated JSON calls, the login token is biometric-only, domain data is HTML, and the security findings are app-side. Phase 4 needs the user account credentials, supplied interactively or via env vars, plus a decision on whether to accept HTML parsing if no JSON endpoint is found. No Phase 4 task has been created yet.

---
<!-- COMMENTS:END -->
