---
id: TASK-32
title: Map stand write endpoints and implement bloppen-api writes
status: Done
assignee: []
created_date: '2026-11-07T00:02:00Z'
updated_date: '2026-11-07T00:03:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 32000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The Android app and website expose product and stand mutations through the same AJAX endpoints the stand editor uses. This task reads the editor JavaScript to recover paths and payloads, implements the stand-scoped API, and reuses the session/reauth machinery so writes are as robust as reads.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The write paths and payloads are documented in re/analysis/16-live-writes.md
- [x] #2 Client::booking exposes product save/active/delete/unprint/move, discount, note, print, upload, info-seen, and shop state
- [x] #3 Writes send X-Requested-With and Referer like the site AJAX calls
- [x] #4 Every write runs the expiry-aware and reactive reauth path
- [x] #5 Buying and booking payments are deliberately not implemented
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Mapped saveBooking, active/inactive, delete, unPrint, moveToCategory, move, infoSeen, noteActiveToggle, saveDiscount, aktiverShop/deaktiverShop, printPriceTags, uploadImage, and getProductsForBooth from scripts.min.js. Writes send X-Requested-With and Referer and call ensure_authenticated. Buying and payment operations were omitted.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented the stand/product write surface in bloppen-api with typed ProductDraft, payloads matching the site editor, repeated product_ids[], JSON form fields, multipart upload, and payment-free shop state. Documented in re/analysis/16-live-writes.md.
<!-- SECTION:FINAL_SUMMARY:END -->
