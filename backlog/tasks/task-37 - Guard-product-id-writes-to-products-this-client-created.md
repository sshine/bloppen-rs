---
id: TASK-37
title: Guard product-id writes to products this client created
status: Done
assignee: []
created_date: '2026-11-07T10:10:00Z'
updated_date: '2026-11-07T10:13:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 37000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user clarified that a booking is a stand rental period, not an item, and asked that pre-existing items not be modified. The write layer should therefore permit product-id operations only on products this client created, with an explicit override for the rare case where touching an existing listing is intended.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 re/analysis/17-bookings-vs-products.md explains the booking/stand/product distinction
- [x] #2 CLI records ids returned by product add in created-products.json with mode 0600
- [x] #3 CLI product-id operations refuse unknown ids unless --allow-existing-products is given
- [x] #4 MCP records ids from save_products and refuses unknown ids unless --allow-existing-products is given
- [x] #5 Tests cover the CLI guard, the MCP guard, and modifying a product the server just created
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
CLI uses created-products.json beside the session file; MCP uses an in-process HashSet populated from newProductIds. The guard is bypassed by --allow-existing-products. Tests: CLI unknown-id refusal, MCP unknown-id refusal, and MCP save-then-modify success. just ci passes.

New products are also hidden by default: ProductDraft::new uses active=false, the CLI needs --active to show one, and the MCP ProductInput defaults active to false. This matches the do-not-put-things-for-sale-without-intent policy.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Clarified that a booking is a stand rental period, not an item, and enforced the user constraint that pre-existing products are not modified. Product-id writes default to ids created by the same CLI/MCP process, with an explicit --allow-existing-products override. Documented in re/analysis/17-bookings-vs-products.md.
<!-- SECTION:FINAL_SUMMARY:END -->
