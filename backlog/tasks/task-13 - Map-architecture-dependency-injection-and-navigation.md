---
id: TASK-13
title: Map architecture, dependency injection, and navigation
status: Done
assignee: []
created_date: '2026-11-06T20:08:00Z'
updated_date: '2026-11-06T20:10:00Z'
labels: []
dependencies: []
priority: high
type: task
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Phase 3 needs a map of how the decompiled app is wired before tracing individual endpoints. The app is one MainActivity with Compose navigation, an application class, use cases, and DI modules. Knowing where auth state gates navigation and where the API implementation is constructed keeps the later endpoint and auth work from guessing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 re/analysis/05-architecture.md documents the application class, DI modules, and repository/use-case layers
- [x] #2 The Compose navigation graph, destinations, and start destination are listed
- [x] #3 The document identifies where login state is read and where it gates navigation
- [x] #4 The relationship between BoerneloppenAPI, BoerneloppenAPIImpl, RetrofitBoerneloppenApi, and their callers is explained
- [x] #5 Every tangents discovered while mapping architecture is recorded as a backlog task
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Confirmed Koin (not Hilt/Dagger), a single MainActivity, three Compose routes, and a four-tab WebView shell. Recorded that there is no auth interceptor: the Cookie header is passed per call. Beagle and HttpLoggingInterceptor(BODY) remain in release; TASK-18 covers them.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrote re/analysis/05-architecture.md: application startup, Koin bindings, OkHttp interceptor order, Compose navigation and start destination, login-state gate, storage keys, and the MainScreenViewModel -> use case -> MainRepository -> BoerneloppenAPI -> Retrofit chain. Also records the WebView-heavy design and two observations for later phases.
<!-- SECTION:FINAL_SUMMARY:END -->
