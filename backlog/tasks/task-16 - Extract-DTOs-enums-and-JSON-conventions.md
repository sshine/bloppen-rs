---
id: TASK-16
title: Extract DTOs, enums, and JSON conventions
status: To Do
assignee: []
created_date: '2026-11-06T20:08:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 16000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The Rust library needs typed models that match the wire format. This task extracts request/response DTOs, domain entities, enums, nullable fields, and serialization annotations, and records where the decompiled names differ from JSON names.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 re/analysis/08-data-models.md lists the request and response models used by the API client
- [ ] #2 JSON field names, nullability, optionality, and enum values are recorded
- [ ] #3 Serialization annotations or custom converters are identified
- [ ] #4 Mismatches or ambiguities that need live validation are called out
<!-- AC:END -->
