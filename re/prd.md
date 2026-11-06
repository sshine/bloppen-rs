# PRD: Børneloppen Android app → Rust client

**Status:** Phase 4 live validation complete for login, token, and pages; logout and the Phase 5 gate pending
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

## 6.2 Phase 3 results (2026-11-06)

- **Architecture**: Koin DI, one `MainActivity`, Compose navigation with three
  routes (`loginScreen`, `webViewsScreen`, `menuScreen/{url}/{tag}`), four
  WebView tabs (home, stand, sale, club), and a
  ViewModel → use case → `MainRepository` → `BoerneloppenAPI` → Retrofit chain.
- **Native API surface is tiny**: only `GET /boerneloppen-theme/mobileApp/getLoginToken.json` and `POST /boerneloppen-theme/mobileApp/saveFirebaseToken.json`, both with a raw
  `Cookie` header. The website's `mobileapp.js` adds `loginWithToken.json` and
  `handleBadgeCount.json`, which the native app does not call through Retrofit.
- **Auth**: the login screen is a WebView form at `/users/login` with
  `email`/`password`. The server sets a `vmcms` cookie (HttpOnly, Secure,
  SameSite=Lax, 7 days). The app reads it from `CookieManager`, stores it,
  calls `getLoginToken.json` with `Cookie: vmcms=...`, and saves the returned
  token encrypted with AndroidKeyStore (AES/GCM). The token is only used for
  biometric re-entry via the website's `loginWithToken`.
- **Logout bug**: the UI logout only writes `is_logged_in=false`; it does not
  clear the stored `vmcms` cookie or login token. The pruning repository method
  exists but is not wired up.
- **Models**: three flat, nullable-string models (`LoginTokenModel`,
  `ResponseModel`, `ErrorResponseModel`). Auth failures are HTTP 200 with
  `status:"error"`, not 401.
- **Domain data is HTML**: loyalty, stands, sales, profile, notifications, and
  the menu are server-rendered WebView pages. There are no native domain DTOs
  or JSON list endpoints in the app.
- **Security**: global cleartext, no certificate pinning,
  `HttpLoggingInterceptor(BODY)` and Beagle in release, Firebase token logged
  via Timber, plaintext `vmcms` cookie, non-auth-bound Keystore key, and the
  logout gap above.
- **Rust implications**: the `vmcms` cookie is the primary credential; the
  login token is optional biometric re-entry. Domain data is HTML until Phase 4
  discovers the website's internal JSON endpoints. The Rust client should clear
  local credentials on logout rather than reproduce the app's bug.

## 6.3 Phase 4 setup (2026-11-06)

- **Live credentials use secretspec with the keyring provider.** The global
  secretspec configuration already defaults to `provider = "keyring"`.
  `secretspec.toml` declares `BORNELOPPEN_EMAIL` and `BORNELOPPEN_PASSWORD`
  (required) and `BORNELOPPEN_TEST_WRITES` (optional). No value is committed.
- **Commands:** `just secrets-set` prompts and stores the two credentials;
  `just secrets-check` reports missing names without prompting; `just probe`
  runs the probe through `secretspec run --reason "bloppen live API validation"`, which injects the values as environment variables.
- **Rust probe:** the workspace contains `crates/bloppen-probe`, a reqwest
  client with a persistent cookie jar, redirect following, the
  `BoerneloppenApp` User-Agent, and output that redacts passwords, cookie
  values, and tokens. It checks `BORNELOPPEN_EMAIL`/`BORNELOPPEN_PASSWORD`
  from the environment only.
- **Status:** the probe builds; `cargo fmt`, `cargo clippy`, and
  `--check-env` pass. The keyring currently reports both required secrets
  missing, so live login has not run yet. Phase 4 continues after the
  credentials are set locally.

## 6.4 Phase 4 live results (2026-11-06)

- **Login works with username/password + cookie jar.** `POST /users/login` with
  `email`/`password` returns 200 and lands on `/mit-boerneloppen/forside` with
  the title `Mit Børneloppen`. No CAPTCHA or 2FA appeared.
- **`vmcms` does not rotate.** It stays a 32-byte session identifier before and
  after login. The server adds `hasLogin`, `HttpAccelerator`, and
  `ShopWishlist_UserUUID` cookies. The Rust client must keep the whole jar, not
  just `vmcms`.
- **Failed login is distinguishable.** A fake account produced HTTP 500 and a
  redirect back to `/users/login`; the probe reports `authenticated=false` and
  exits non-zero. A real account with a wrong password may behave differently.
- **Login token works.** `getLoginToken.json` returns
  `{"status":"success","token":"..."}` with a 49-byte token and no message.
  `loginWithToken.json` with a fresh cookie jar returns
  `{"status":"success","message":"Du er nu logget ind"}` and authenticates the
  existing `vmcms` session without setting a new cookie.
- **Pages are HTML.** All nine tabs/menu pages returned 200 `text/html` with
  the expected titles. The `sale` tab is the payout page (`Udbetaling`).
- **A public JSON endpoint exists for product search.**
  `GET /boerneloppen-theme/searches/search.json?query=...` returns
  `application/json` with `data`, `pages`, and `total`; `data` is an array of
  product objects with name, size, store, stand, price, sold price, discount,
  and image URLs. It works without authentication.
- **Other domain endpoints need mapping.** The pages reference internal AJAX
  endpoints for sales history, move options, available prices, payment
  markups, booking changes, product status, uploads, and purchases. Several
  return 500 without the page's parameters; writes were not called.
- **Logout was not exercised** to avoid invalidating the developer's other
  sessions. It remains TASK-25.

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
| 2026-11-06 | Use username/password + `vmcms` cookie as the primary auth path | User direction and the decompiled login flow both point to the website form and session cookie |
| 2026-11-06 | Treat the login token as optional biometric re-entry, not API auth | The native API only uses the cookie; the token is injected into the website's `loginWithToken` |
| 2026-11-06 | Treat domain data as HTML until Phase 4 proves otherwise | The native app has no domain DTOs or JSON list endpoints |
| 2026-11-06 | Do not reproduce the app's logout behaviour in the Rust client | The app leaves the cookie and token on disk after logout |
| 2026-11-06 | Use secretspec with the keyring provider for live credentials | Matches `nemlig-rs`; keeps the account password out of files, chat, and argv |
| 2026-11-06 | Keep the whole cookie jar; `vmcms` is stable, not rotated at login | Live validation showed login state comes from additional cookies and server-side session state |
| 2026-11-06 | Use `searches/search.json` for product search; HTML/AJAX for account pages | It is public, JSON, and already returns the product fields the CLI needs |
