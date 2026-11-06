---
id: TASK-24
title: Probe authenticated pages and choose HTML or JSON
status: Done
assignee: []
created_date: '2026-11-06T20:29:00Z'
updated_date: '2026-11-06T22:35:00Z'
labels: []
dependencies:
  - TASK-22
priority: high
type: task
ordinal: 24000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The app renders loyalty, stand, sale, club, profile, notifications, and menu pages as HTML. Phase 4 must decide whether the Rust client parses that HTML or whether the pages expose internal JSON endpoints that are not visible in mobileapp.js.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The probe requests each authenticated tab and menu page and records status, final URL, content type, size, and title
- [x] #2 Any JSON response is identified by content type and top-level shape
- [x] #3 Page scripts are inspected for internal XHR/JSON endpoints worth testing
- [x] #4 re/analysis/13-live-pages.md states whether HTML parsing is required or a JSON path exists
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
All nine pages returned 200 HTML with titles. Public search.json returns JSON {data,pages,total} with 100 product objects; data[0] keys are discount, full_image_url, image_url, name, price, price_sold, size, sold, stand, store. Internal AJAX endpoints were catalogued; write endpoints were not called. Saved pages live under gitignored secrets/pages/.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Surveyed all authenticated pages and the page JavaScript. Account data is HTML plus internal AJAX; product search has a public JSON endpoint. Documented page sizes/titles, search.json shape, read-only candidates, write endpoints, and the HTML-vs-JSON decision in re/analysis/13-live-pages.md.
<!-- SECTION:FINAL_SUMMARY:END -->
