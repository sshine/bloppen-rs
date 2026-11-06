# Architecture, dependency injection, and navigation

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`
**Sources:** jadx output under `re/decompiled/jadx/sources/` (gitignored)

## Summary

Børneloppen is a single-activity Jetpack Compose app that wraps the
`www.boerneloppen.dk` website in WebView tabs. Native code handles startup,
dependency injection, two small JSON endpoints, secure token storage, Firebase
push, Facebook login callbacks, and biometric unlock. The business UI — home,
stand, sale, club, profile, notifications, and the menu pages — is server-driven
HTML inside WebViews.

## Layers

```text
MainActivity (Compose)
  └── MainScreen / LoginScreen / MainWebViews / MenuItemScreen
        └── MainScreenViewModel
              ├── GetLoginStatusUseCase / UpdateLoginStatusUseCase
              ├── GetLoginTokenUseCase / SaveLoginTokenUseCase
              ├── RequestLoginTokenUseCase
              ├── UploadFirebaseTokenUseCase
              ├── VmcmsCookieUseCase
              ├── GetFirebaseTokenUseCase / SaveNewFirebaseTokenUseCase
              ├── FacebookConsentUseCase
              └── LogoutUseCase
                    └── MainRepository
                          ├── KeyValueStore (SharedPreferences)
                          └── BoerneloppenAPI
                                └── BoerneloppenAPIImpl
                                      └── RetrofitBoerneloppenApi (Retrofit)
                                            └── OkHttpClient
```

There is no Hilt or Dagger. Dependency injection is **Koin**.

## Application startup

`dk.boerneloppen.boerneloppenapp.app.BoerneloppenApp` extends `Application` and
runs four setup steps in `onCreate()`:

1. `setupTimber()` — plants a Timber tree that forwards to Beagle's log module.
2. `setupBeagle()` — initializes the Beagle in-app debug/network panel.
3. `setupKoin()` — starts Koin with `ModuleKt.getAppModules()`.
4. `setUpFirebase()` — initializes Firebase and enables Crashlytics collection.

Beagle and the OkHttp `HttpLoggingInterceptor` are present in the release build,
not gated by `BuildConfig.DEBUG`. That is recorded again in
[`10-security.md`](10-security.md).

## Koin modules

`ModuleKt` defines one Koin module containing:

| Binding | Scope | Notes |
| --- | --- | --- |
| `SharedPreferences` | singleton | File name `default` |
| `KeyValueStore` | singleton | `SharedPreferencesKeyValueStore` |
| `OkHttpClient` (qualifier `noAuth`) | singleton | Connection pool disabled, HTTP/1.1 only, 1-minute timeouts, retry enabled |
| `RetrofitBoerneloppenApi` (qualifier `noAuth`) | singleton | Base URL `BuildConfig.BASE_URL`, Gson converter |
| `Gson` | singleton | Lenient, complex map keys, pretty printing |
| `HttpLoggingInterceptor` | singleton | Level `BODY` |
| `BoerneloppenAPI` | singleton | `BoerneloppenAPIImpl` |
| `MainRepository` | singleton | Takes `KeyValueStore` and `BoerneloppenAPI` |
| `*UseCase` | singletons | One per use case |
| `MainScreenViewModel` | factory | Injects all ten use cases |

## Network stack

`ModuleKt` builds the single `OkHttpClient` with these interceptors, in order:

1. Beagle's OkHttp logger, when available.
2. `AcceptTypeHeaderChangerInterceptor` — currently just rebuilds the request.
3. `NetworkInterceptor` — throws `NetworkException` when the device is offline.
4. `RetrofitErrorInterceptor` — maps HTTP 403 to `ForbiddenException` and other
   JSON error bodies to `GenericApiError`.
5. `HttpLoggingInterceptor` — logs full bodies.
6. `OkHttpProfilerInterceptor` — profiler overlay.

There is **no auth interceptor**. The only authenticated calls pass a `Cookie`
header explicitly from the use case through Retrofit. There is no base-url
environment switch in the release flavor: `BuildConfig.BASE_URL` is
`https://www.boerneloppen.dk/`. The `Constants` class contains staging and other
market URLs, but they are selected by product flavor, not at runtime.

## Navigation

`MainActivity` is the only activity (declared in the manifest; see
[`02-manifest.md`](02-manifest.md)). It hosts `MainScreen`, which observes login
state and calls `MainNavigation` in `navigation/NavigationKt`.

`MainNavigation` chooses the start destination from the login state:

```kotlin
val start = if (isLoggedIn) ConstantsKt.WEB_VIEWS else ConstantsKt.LOGIN
```

Routes:

| Route | Composable | Purpose |
| --- | --- | --- |
| `loginScreen` | `LoginScreen` | WebView login at `/users/login` |
| `webViewsScreen` | `MainWebViews` | Authenticated WebView tabs |
| `menuScreen/{menuItemUrl}/{menuItemTag}` | `MenuItemScreen` | One WebView per menu item, URL and tag as arguments |

`MainWebViews` keeps four WebView tabs: `home` (`/mit-boerneloppen/forside`),
`stand` (`/mit-boerneloppen/stand`), `sales` (`/mit-boerneloppen/salg`), and
`club` (`/mit-boerneloppen/klub`). The menu destinations are `book`, `find`,
`boost`, `faq`, `profile`, `notifications`, and `logout`.

The tab constants live in `constants/Tabs`; the route strings live in
`constants/ConstantsKt`.

## Login-state gate

`MainRepository` owns the login state:

- Constructor reads `is_logged_in` from `KeyValueStore` and initializes
  `MutableStateFlow<Boolean>`; absent means `false`.
- `updateLoggedInStatus(boolean)` updates the flow and writes `is_logged_in`.
- `GetLoginStatusUseCase` exposes the `StateFlow<Boolean>` to
  `MainScreenViewModel`.
- `MainNavigation` reads the state to pick the start destination, and
  `MainScreen`/`MainWebViews` use it to switch between login and content.

Logout prunes the whole `SharedPreferences` file (`KeyValueStore.prune()`) and
sets the login state to false. `MenuItemScreen` triggers this when the WebView
navigates to `BuildConfig.LOG_OUT_URL` (`/users/logout`).

## Storage and secure storage

`SharedPreferencesKeyValueStore` wraps the `default` preferences file. Keys are
declared in `util/store/KeyValueStoreKeys`:

| Key | Value |
| --- | --- |
| `is_logged_in` | Boolean |
| `login_token` | JSON `{alias, data, iv}` for the encrypted login token |
| `boerneloppen_login_token` | Keystore alias used to encrypt the login token |
| `vmcms_cookie` | Raw `vmcms=...` cookie string |
| `firebase_token` | Firebase Cloud Messaging token |
| `access_token` | Declared but not written by the code inspected here |
| `show_notifications` | Notification preference |
| `request_facebook_consent`, `facebook_consent_status` | Facebook consent flow |

The login token is encrypted with Android Keystore using AES/GCM/NoPadding
(`util/keystore/KeystoreHelper`) and stored as Base64 data plus IV. The VMCMS
cookie is stored in plain SharedPreferences, not encrypted. That distinction is
covered in [`10-security.md`](10-security.md).

## API construction and callers

- `BoerneloppenAPI` is a two-method interface: `getLoginToken(cookie)` and
  `uploadFirebaseToken(cookie, token)`.
- `BoerneloppenAPIImpl` delegates directly to `RetrofitBoerneloppenApi`.
- `RetrofitBoerneloppenApi` is the Retrofit interface with the actual paths and
  headers. It is the only source of native HTTP endpoints.
- `MainRepository` wraps the API and the key-value store; use cases are thin
  adapters around repository methods.
- `MainScreenViewModel` is the only caller of most use cases and the owner of
  `MainScreenViewState`.

The endpoint details are in [`06-api-surface.md`](06-api-surface.md), the login
flow in [`07-auth.md`](07-auth.md), and the models in
[`08-data-models.md`](08-data-models.md).

## App-owned entry points

| Entry point | Role |
| --- | --- |
| `MainActivity` | Launcher, Compose host, notification-intent handling |
| `MainScreenViewModel` | UI state, login token, Firebase upload, tabs, dialogs |
| `BoerneloppenFirebaseMessagingService` | Push token refresh and message handling |
| `LoginScreen` + `WebAppInterface` | WebView login and JavaScript bridge |
| `MainWebViews` / `HomeScreen` / `MenuItemScreen` | WebView tabs and menu pages |
| `Facebook` SDK activities | Facebook login callbacks (manifest) |

## Observations for later phases

- The native API surface is deliberately tiny; most data is HTML from
  `www.boerneloppen.dk`. Phase 4 should decide whether the Rust client talks to
  the same JSON endpoints, to the web pages, or both.
- `GetVmcmsCookieUseCase` exists but the inspected call sites use
  `VmcmsCookieUseCase` (get/set) and `MainRepository.getVmcmsCookie` directly.
  Treat it as a duplicate or legacy wrapper.
- Beagle and body-level HTTP logging in a release build are debug conveniences
  that also expose request/response bodies. They are recorded in the security
  task rather than changed here.
- The single-activity Compose design means there are no per-screen deep links;
  navigation is internal state, and push intents only carry a tab name and an
  optional value.
