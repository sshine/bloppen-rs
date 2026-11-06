---
id: TASK-8
title: Decode and document the AndroidManifest
status: To Do
assignee: []
created_date: '2026-11-06T18:38:00Z'
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
- [ ] #1 apktool decodes base.apk and produces a readable AndroidManifest.xml
- [ ] #2 re/analysis/02-manifest.md lists activities, services, receivers, providers, and permissions
- [ ] #3 Intent filters, deep links, and exported components are called out
- [ ] #4 The network security config and third-party SDK providers are documented
- [ ] #5 The document names the likely entry points for login, loyalty, and stand features
<!-- AC:END -->
