# PRD: Børneloppen Android app → Rust client

**Status:** Phase 2 complete; Phase 3 gate awaiting user confirmation
**Last updated:** 2026-11-06

## 1. Summary

Reverse-engineer the Danish **Børneloppen** Android app into a
production-quality Rust API library and CLI, and document the app's API,
authentication flow, and data model.

Børneloppen is a Danish chain of children's flea-market supermarkets. The app
is used for the "Loppeklub" loyalty programme and for stand holders. This is a
native Android app (Java/Kotlin), not Flutter, React Native, or Xamarin.

## 2. Target

| Field | Value |
| --- | --- |
| Package | `dk.boerneloppen.boerneloppen` |
| App name | Børneloppen |
| Version under analysis | **1.0.4** (versionCode 23) |
| Latest on Google Play | 1.0.9 (versionCode 36), not mirrored |
| minSdk / targetSdk | 23 / 35 |
| Distribution | Google Play; APKPure XAPK for 1.0.4 |
| Locales | Danish (the APK also ships an English split) |

The Swedish package `se.boerneloppen.boerneloppen` is a separate app and is out
of scope unless the user asks for it.

### Acquired artifact

```text
apk/raw/Børneloppen_1.0.4_APKPure.xapk   49,376,405 bytes
  sha256 d20cd98fae0f72669ab66d8771b80056ac94ae95611e4ac26274f177d037ed2d

apk/extracted/base.apk                   49,180,237 bytes
  sha256 54a2a8062319c3d9c31f23f11a31541eec1a28ba82e8186e1c374e41fce55bb3
```

The XAPK contains one base APK (`dk.boerneloppen.boerneloppen.apk`) plus
`config.en.apk` and `config.mdpi.apk`. The base APK has four DEX files and no
Flutter/React Native/Xamarin payload.

## 3. Goals

- Map the app's architecture, API endpoints, authentication flow, and data
  models from the decompiled APK.
- Validate those findings against the live service using the developer's own
  account, without relying on an emulator or a traffic proxy.
- Implement a typed Rust client library (`bloppen-api`) and CLI (`bloppen-cli`)
  for the read-only and safe write operations that the API exposes.
- Cover the client with unit tests, fixture tests from captured responses, and
  opt-in live tests.
- Keep the repository free of APKs, decompiled output, tokens, and PII.

## 4. Non-goals

- Modifying, repackaging, or redistributing the app.
- Bypassing authentication, paywalls, rate limits, or other access controls.
- Collecting or publishing other users' personal data.
- Building a full replacement UI. The deliverable is a CLI and library.
- A FUSE filesystem unless the user explicitly opts in at Phase 8.

## 5. Milestones

| # | Milestone | Gate |
| --- | --- | --- |
| M1 | Project setup: dendritic flake, PRD, agent guide, backlog, APK acquisition | `nix develop` evaluates; `just fmt-check` passes |
| M2 | Extract and decompile: split APKs, jadx, apktool, manifest, native libraries | `re/analysis/` documents file census and manifest |
| M3 | Static analysis: architecture, endpoints, auth flow, DTOs, domain model | Endpoint/auth/model catalogue reviewed |
| M4 | Live API validation: minimal Rust client, authenticate, call each endpoint | Discrepancies documented; fixtures saved under `secrets/` |
| M5 | Rust library: HTTP client, auth, session, models, service layer, tests | `cargo test` and `just e2e` pass |
| M6 | CLI: `clap` subcommands per domain, human and `--json` output | `just showcase` runs all non-destructive commands |
| M7 | Integration testing: live tests (ignored by default), fixtures, showcase | Live suite passes with a valid session |
| M8 | Advanced features (opt-in) | User chooses; none assumed |
| M9 | Security and cleanup: PII scan, history scrub if needed, README, LICENSE | `git log` and tracked files are clean |

## 6. Constraints and decisions

- **Static first, then direct API calls.** No emulator, proxy, or runtime
  instrumentation is assumed. If static analysis stalls, revisit this.
- **Version gap accepted for now.** Google Play serves 1.0.9, but every
  accessible mirror stops at 1.0.4. A Google Play download needs an AAS token
  (`apkeep -a dk.boerneloppen.boerneloppen -d google-play -t <token>`). Revisit
  if analysis hits behaviour that 1.0.4 does not explain.
- **Nix dendritic flake.** `flake.nix` only wires inputs and
  `import-tree ./nix`; each `nix/*.nix` file is an independent flake-parts
  module. New tooling belongs in its own module, not in one growing shell.
- **Backlog.md is the task source of truth.** The CLI is wrapped by
  `bin/backlog` because the upstream binary is not NixOS-native.
- **All generated artifacts are disposable.** The APK and decompiled output can
  be regenerated; only the PRD, analysis notes, scripts, and source are tracked.
- **One task per agent, sequentially.** Tangents become new backlog tasks.

## 6.1 Phase 2 results (2026-11-06)

- **Native Android app**: 855 entries, four DEX files (45.1 MiB
  uncompressed), no native libraries, no managed assemblies, and no
  Flutter/React Native/Xamarin/Capacitor/Cordova markers.
- **No strong obfuscation**: 0.44% of DEX type descriptors have 1-2 character
  class names and there is no `mapping.txt`. Descriptor evidence shows
  Retrofit2, OkHttp, Okio, Coil, Timber, Kotlin coroutines, and Jetpack Compose.
- **Manifest**: `BoerneloppenApp` is the application class and `MainActivity`
  is the only app activity. Facebook Login callbacks are the only deep links.
  `BoerneloppenFirebaseMessagingService` is the only custom service. A
  `FileProvider` is configured but not exported.
- **Network security**: `usesCleartextTraffic=true` and
  `base-config cleartextTrafficPermitted=true` apply to every domain. Trust
  anchors are the bundled ISRG roots plus the system store; there is no
  pinning. Phase 3/4 must determine whether the real API is HTTP or HTTPS and
  whether tokens ever travel cleartext.
- **Decompilation**: jadx 1.5.6 produced 19,734 Java files in 51 s. It reported
  228 partial errors, but only three app-package files are affected
  (`HomeScreenKt`, `MainScreenKt`, `MenuItemScreenKt`, all Compose).
  apktool 3.0.3 produced 30,749 smali files plus resources in 11 s.
- **Phase 3 leads**: `BoerneloppenAPI`, `BoerneloppenAPIImpl`,
  `RetrofitBoerneloppenApi`, the API interceptors, `LoginTokenModel`,
  `VmcmsCookieUseCase`, `RequestLoginTokenUseCase`, and
  `GetLoginStatusUseCase`.
- **Language**: the API client and CLI will be Rust (`bloppen-api` and
  `bloppen-cli`).

## 7. Deliverables

- `re/` analysis: file census, manifest, architecture, endpoint catalogue,
  auth flow, data models, domain model, discrepancies.
- `bloppen-api`: Rust library crate.
- `bloppen-cli`: Rust CLI crate.
- Tests: unit, fixtures, opt-in live.
- `README.md` with scope, build, authentication, and token storage.
- `LICENSE` (MIT or Apache-2.0; choose at M9).
- A clean repository: no APKs, no decompiled trees, no secrets, no PII.

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Obfuscated DEX makes static analysis slow | Try jadx and apktool; use strings and manifest first; document dead ends |
| Auth is a non-standard redirect or WebView flow | Start with the decompiled auth code; use `--manual` login as the fallback |
| API responses differ from decompiled DTOs | Treat Phase 4 as an iterative "call, deserialize, fix" loop |
| 1.0.4 is missing endpoints added by 1.0.9 | Obtain an AAS token and re-extract; compare manifests and strings |
| Live responses contain PII | Store captures in `secrets/`; anonymise anything quoted in `re/` |
| Scope creep into a full UI or FUSE | Phase 8 is opt-in; ask the user before adding tasks |

## 9. Repository layout

```text
.
├── AGENTS.md              # agent rules and backlog workflow
├── flake.nix              # inputs + import-tree ./nix
├── nix/                   # dendritic flake-parts modules
│   ├── systems.nix
│   ├── rust-overlay.nix
│   ├── devshell.nix
│   ├── re-tools.nix
│   ├── rust.nix
│   └── treefmt.nix
├── apk/                   # downloaded XAPK and extracted splits (ignored)
├── re/                    # PRD, scripts, analysis, generated output (partly ignored)
├── backlog/               # Backlog.md tasks
├── bin/backlog            # pinned Backlog.md CLI wrapper
├── justfile
└── rust-toolchain.toml
```

## 10. Decision log

| Date | Decision | Rationale |
| --- | --- | --- |
| 2026-11-06 | Target `dk.boerneloppen.boerneloppen`, not the Swedish package | Working directory is `bloppen`; user is in Denmark |
| 2026-11-06 | Proceed with 1.0.4 | 1.0.9 is not available from any accessible mirror; requires an AAS token |
| 2026-11-06 | Use a dendritic flake instead of `shell.nix` | Mimics `~/Projects/nursery/mxroute-rs`; keeps each tool group independent |
| 2026-11-06 | Confirm Rust for the API client and CLI | User confirmed; the milestones target `bloppen-api` and `bloppen-cli` |
