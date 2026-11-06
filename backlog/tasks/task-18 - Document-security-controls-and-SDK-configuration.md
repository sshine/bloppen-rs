---
id: TASK-18
title: Document security controls and SDK configuration
status: Done
assignee: []
created_date: '2026-11-06T20:09:00Z'
updated_date: '2026-11-06T20:14:00Z'
labels: []
dependencies: []
priority: medium
type: task
ordinal: 18000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The manifest already shows global cleartext and no pinning. This task follows that into the code and configuration: keystore and biometric storage, token persistence, Firebase and Facebook initialization, and any runtime checks that affect a third-party client.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 re/analysis/10-security.md documents where tokens and cookies are stored and whether they are encrypted or backed by the keystore
- [x] #2 Biometric and keystore use cases are explained
- [x] #3 Cleartext, certificate pinning, and any network trust overrides are confirmed from code as well as manifest
- [x] #4 Firebase, Facebook, and Google client configuration relevant to a third-party client is summarized without copying secrets into tracked files
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Confirmed no CertificatePinner or custom TrustManager. The login-token Keystore key is AES/GCM but is not user-authentication-bound; the biometric prompt only gates the app code path. Logout leaves the cookie and token on disk because the pruning LogoutUseCase is not wired into the view model. Beagle and HttpLoggingInterceptor(BODY) are in the release build.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrote re/analysis/10-security.md: cleartext and pinning, release logging, plaintext cookie and encrypted token storage, the non-auth-bound Keystore key, the logout bug, WebView bridge surface, FileProvider paths, SDK configuration, and Rust-client recommendations.
<!-- SECTION:FINAL_SUMMARY:END -->
