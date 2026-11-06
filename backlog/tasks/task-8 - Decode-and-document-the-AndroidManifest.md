---
id: TASK-8
title: Decode and document the AndroidManifest
status: Done
assignee:
  - '@sshine'
created_date: '2026-11-06T18:38:00Z'
updated_date: '2026-11-06T18:46:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The manifest is the authoritative source for the app components, permissions, intent filters, deep links, SDK levels, and third-party SDK initializers. Phase 3 endpoint and auth analysis depends on knowing every entry point before reading code.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 apktool decodes base.apk and produces a readable AndroidManifest.xml
- [x] #2 re/analysis/02-manifest.md lists activities, services, receivers, providers, and permissions
- [x] #3 Intent filters, deep links, and exported components are called out
- [x] #4 The network security config and third-party SDK providers are documented
- [x] #5 The document names the likely entry points for login, loyalty, and stand features
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Decode base.apk with apktool into re/decompiled/apktool.
2. Parse the decoded manifest with a small Python script to list components, permissions, intent filters, deep links, exported flags, and metadata.
3. Read the network security config and other res/xml policies.
4. Write re/analysis/02-manifest.md, naming the likely login, loyalty, and stand entry points.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
apktool 3.0.3 decoded the manifest and resources. Key findings: only MainActivity is an app activity; Facebook callbacks are the only deep links; cleartext HTTP is allowed app-wide; no certificate pinning; secrets-like manifest strings were copied to gitignored secrets/manifest-strings.xml.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrote re/analysis/02-manifest.md from the apktool decode. It covers identity, permissions, components, exported status, the two Facebook callback URIs, the global cleartext policy, the ISRG/system trust anchors, and the third-party SDK inventory. It identifies BoerneloppenApp, MainActivity, and BoerneloppenFirebaseMessagingService as the app-owned entry points and points Phase 3 at Compose navigation and the auth classes.
<!-- SECTION:FINAL_SUMMARY:END -->
