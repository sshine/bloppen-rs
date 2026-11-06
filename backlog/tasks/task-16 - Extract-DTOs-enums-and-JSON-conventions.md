---
id: TASK-16
title: Extract DTOs, enums, and JSON conventions
status: Done
assignee: []
created_date: '2026-11-06T20:08:00Z'
updated_date: '2026-11-06T20:13:00Z'
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
- [x] #1 re/analysis/08-data-models.md lists the request and response models used by the API client
- [x] #2 JSON field names, nullability, optionality, and enum values are recorded
- [x] #3 Serialization annotations or custom converters are identified
- [x] #4 Mismatches or ambiguities that need live validation are called out
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Only three wire models exist: LoginTokenModel, ResponseModel, and ErrorResponseModel. All fields are nullable Strings; Gson uses exact property names and only ErrorResponseModel has explicit @SerializedName. UI models (MainScreenViewState, MenuItem, Tabs, Alert) were separated from the API contract.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrote re/analysis/08-data-models.md: the three API models, JSON keys and nullability, Gson behaviour, the 200-with-status-error convention, UI-only models, observed enum values, and what Phase 4 must confirm.
<!-- SECTION:FINAL_SUMMARY:END -->
