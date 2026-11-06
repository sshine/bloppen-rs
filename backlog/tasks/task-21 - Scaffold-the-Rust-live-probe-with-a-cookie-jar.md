---
id: TASK-21
title: Scaffold the Rust live probe with a cookie jar
status: Done
assignee: []
created_date: '2026-11-06T20:29:00Z'
updated_date: '2026-11-06T20:34:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 21000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Phase 4 needs a small Rust program that reads credentials injected by secretspec, keeps the vmcms session in a cookie jar, and can call the JSON and HTML endpoints without logging secrets. This is a probe, not the final library; Phase 5 will factor the reusable client out of it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A Rust workspace and bloppen-probe crate build inside nix develop
- [x] #2 The probe reads BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD from the environment, not from files or CLI arguments
- [x] #3 The probe uses a cookie jar, follows redirects, and sets the BoerneloppenApp User-Agent
- [x] #4 The probe never prints passwords, cookie values, or token values; it prints names, lengths, statuses, URLs, and titles
- [x] #5 cargo fmt, cargo clippy, and the probe --check-env mode pass without credentials
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
cargo build -p bloppen-probe, cargo fmt --check, and cargo clippy --workspace --all-targets --all-features -- -D warnings all pass. `cargo run -p bloppen-probe -- --check-env` reports all three names missing and exits 0 without network access. The probe prints cookie names and value lengths only, and token presence/length only.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Scaffolded the Cargo workspace and crates/bloppen-probe: a reqwest client with a cookie jar, redirects, BoerneloppenApp UA, env-only credentials, login, getLoginToken, optional loginWithToken, a read-only page sweep, optional logout, and redacted output. Build, fmt, clippy, and check-env pass.
<!-- SECTION:FINAL_SUMMARY:END -->
