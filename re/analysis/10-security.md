# Security controls and SDK configuration

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`
**Scope:** app-side controls that matter to a third-party Rust client. This is
not a penetration-test report.

## Findings at a glance

| # | Finding | Severity | Evidence |
| --- | --- | --- | --- |
| 1 | Cleartext HTTP allowed for every domain | Medium | `usesCleartextTraffic="true"`, `base-config cleartextTrafficPermitted="true"` |
| 2 | No certificate pinning | Medium | No `CertificatePinner`, no custom `TrustManager`; only ISRG roots + system trust |
| 3 | Full HTTP body logging in release | High | `HttpLoggingInterceptor.Level.BODY`, Beagle network logger |
| 4 | Firebase token written to Timber | Medium | `Timber.d("Firebase token is: " + token)` |
| 5 | `vmcms` cookie stored in plaintext | Medium | `SharedPreferences` key `vmcms_cookie` |
| 6 | Login token key is not user-auth-bound | Medium | AES/GCM Keystore key without `setUserAuthenticationRequired(true)` |
| 7 | Logout leaves cookie and token on disk | High | `MainScreenViewModel.logout()` only flips `is_logged_in` |
| 8 | Broad FileProvider paths | Low | `external-path path="."`, `pictures`, `cache` |
| 9 | Debug tooling in release | Low/Medium | Beagle panel and OkHttp profiler compiled in |
| 10 | Stale legacy storage flag | Low | `requestLegacyExternalStorage="true"` |

## Network transport

The manifest sets:

```xml
<application
    android:usesCleartextTraffic="true"
    android:networkSecurityConfig="@xml/network_security_config" ...>
```

and the config is:

```xml
<network-security-config>
    <base-config cleartextTrafficPermitted="true">
        <trust-anchors>
            <certificates src="@raw/isrg_root_x2" />
            <certificates src="@raw/isrg_root_x1" />
            <certificates src="system" />
        </trust-anchors>
    </base-config>
</network-security-config>
```

Consequences:

- HTTP is permitted app-wide. The current base URL is HTTPS, but any downgrade
  or future HTTP URL would be accepted.
- There is no `<domain-config>`, no `<pin-set>`, and no leaf/CA restriction.
  The bundled ISRG roots widen trust on old Android versions; they do not pin
  Let's Encrypt.
- A search of the app package for `CertificatePinner`, `X509TrustManager`,
  `TrustManager`, `sslSocketFactory`, and `hostnameVerifier` found nothing.
- The native OkHttp client does not set an app User-Agent; the WebView sets
  `BoerneloppenApp`.

For the Rust client: use HTTPS only, verify certificates with the platform
trust store, and do not copy the cleartext exception.

## Logging and debug tooling

`ModuleKt` builds one `HttpLoggingInterceptor` at `Level.BODY` and adds it to
the release client. At `BODY`, OkHttp logs the request line, headers (including
`Cookie`), and the response body. The same client also adds Beagle's OkHttp
logger and `OkHttpProfilerInterceptor`. Beagle's in-app panel is initialized in
`BoerneloppenApp` and is not gated by `BuildConfig.DEBUG`.

`MainRepository.saveNewFirebaseMessagingToken` logs the token directly:

```java
Timber.d("Firebase token is: " + token, new Object[0]);
```

`BoerneloppenApp.setupTimber` forwards Timber to Beagle's log module, so this is
visible in the debug panel as well as logcat.

For the Rust client: never log the `vmcms` cookie, the login token, or the FCM
token. The app's logging is a reason not to reuse its client configuration.

## Credential and token storage

| Value | Storage | Protection |
| --- | --- | --- |
| `vmcms` cookie | `SharedPreferences` key `vmcms_cookie` | **Plaintext** |
| Native login token | `SharedPreferences` key `login_token`, JSON `{alias,data,iv}` | AES/GCM with AndroidKeyStore alias `boerneloppen_login_token` |
| Firebase token | `SharedPreferences` key `firebase_token` | Plaintext |
| Login flag | `is_logged_in` | Plaintext boolean |
| Facebook consent | `request_facebook_consent`, `facebook_consent_status` | Plaintext booleans |
| Notification preference | `show_notifications` | Plaintext |

`KeystoreHelper`:

- Generates an AES-128 key in `AndroidKeyStore`.
- Uses `AES/GCM/NoPadding`.
- Sets purposes `3` (encrypt and decrypt).
- Does **not** call `setUserAuthenticationRequired(true)` and does **not**
  call `setInvalidatedByBiometricEnrollment(true)`.

The biometric prompt in `WebAppInterface`/`LoginScreen` is therefore an
app-level gate around the code path that injects `loginWithToken(...)`. It is
not a cryptographic gate on the Keystore key: any process or code path that can
reach `getLoginToken()` can decrypt the token once the device is unlocked.

`allowBackup="false"` and the empty backup/data-extraction rules are good; they
reduce ADB/cloud backup exposure. They do not protect a rooted device or a live
app process.

## Logout behaviour

This is the most actionable finding for the Rust client.

`MenuItemScreen` detects `/users/logout` and calls
`MainScreenViewModel.logout()`, which calls `UpdateLoginStatusUseCase(false)`.
That writes `is_logged_in = false` only.

`MainRepository.logout()` would call `KeyValueStore.prune()` (clearing the
whole preferences file) and then set the flag, but its only caller is
`LogoutUseCase`, which is bound in Koin and not injected into the view model.
The inspected logout path therefore leaves:

- the plaintext `vmcms_cookie`,
- the encrypted `login_token`,
- the `firebase_token`,
- the Facebook consent flags,

on disk. The website's `/users/logout` clears the server session and the
WebView cookie jar, but the app's saved copy is stale, and the stored login
token can still drive `loginWithToken` for biometric re-entry.

The Rust client should implement the intended behaviour: clear the cookie jar,
delete the token file, then call `/users/logout`.

## WebView and JavaScript bridge

The login and menu WebViews:

- enable JavaScript,
- set the user agent to `BoerneloppenApp`,
- register a JavaScript interface named `Android`.

Login bridge (`WebAppInterface`):

| Method | Effect |
| --- | --- |
| `loginSuccess()` | Reads the `vmcms` cookie, saves it, optionally requests the login token, sets logged in true |
| `initBiometricsLogin()` | Injects the stored token via `loginWithToken`, prompts biometrics, saves the cookie on success |
| `saveVmcmsCookie()` | Reads and saves the `vmcms` cookie |

Menu bridge (`WebAppInterface`):

| Method | Effect |
| --- | --- |
| `pushCheck()` | Answers `pushStatus(true/false)` from the notification permission |
| `pushActivate()` | Requests an FCM token and uploads it |
| `gotoTab(name, options)` | Routes to a native tab or menu route |
| `cameraTriggered()` | Opens the picture picker |

The bridge is only exposed to the loaded site. Because the site is trusted and
loaded over HTTPS, the main residual risk is an XSS or compromised page calling
`gotoTab`, `pushActivate`, or `loginSuccess`. `loginSuccess` cannot accept an
arbitrary cookie value; it re-reads the WebView's cookie jar. It can still flip
the login flag even when no `vmcms` cookie is present.

## FileProvider

`res/xml/filepaths.xml` exposes:

```xml
<external-path name="external_files" path="." />
<cache-path name="cache" path="." />
<external-path name="pictures" path="Pictures" />
```

The provider is `androidx.core.content.FileProvider` with
`exported="false"` and `grantUriPermissions="true"`. Only URIs the app
explicitly grants should be reachable, but the external path is broad. It is
used for page-driven image uploads.

## Third-party SDK configuration

| SDK | Configuration | Notes |
| --- | --- | --- |
| Facebook Login | App ID and client token in `strings.xml`; `AutoInitEnabled=false`, `AutoLogAppEventsEnabled=false`, `AdvertiserIDCollectionEnabled=false` | Consent flags handled locally |
| Firebase Messaging | Default notification icon/channel in manifest; custom service | Token uploaded to `/saveFirebaseToken.json` when a cookie exists |
| Firebase Analytics | `google_app_id`, `google_api_key` in resources | Client-side configuration, not a server secret |
| Firebase Crashlytics | Collection enabled in `BoerneloppenApp` | Crash reports leave the device |
| Google Play services | `default_web_client_id`, `google_api_key` | Used by Firebase/Google auth configuration |
| Beagle | In-app debug panel, log and network modules | Present in release |
| OkHttp Profiler | Interceptor in release client | Present in release |

Exact Facebook and Google values are kept in `secrets/manifest-strings.xml`
(gitignored). They are client identifiers, not user secrets, but they should
not be copied into tracked notes or logs.

## Manifest surface

From [`02-manifest.md`](02-manifest.md):

- `MainActivity` is the only app activity and is exported as the launcher.
- `Facebook CustomTabActivity` is exported for Facebook login callbacks.
- `BoerneloppenFirebaseMessagingService` is exported for
  `com.google.firebase.MESSAGING_EVENT`.
- `FirebaseInstanceIdReceiver` is exported with the C2DM `SEND` permission.
- `ProfileInstallReceiver` is exported with `android.permission.DUMP`.
- `FileProvider` and the Firebase/Facebook/AndroidX providers are not exported.
- `allowBackup=false`; backup and data-extraction rules are empty.

## Recommendations for the Rust client

1. HTTPS only; reject HTTP even though the app allows it.
2. Keep the cookie and login token in memory or a 0600 file; never log them.
3. Clear the cookie jar and token on logout, then call `/users/logout`.
4. Do not reuse the app's `HttpLoggingInterceptor`/Beagle configuration.
5. Treat HTTP 200 + `{"status":"error"}` as an auth failure.
6. Do not automate write operations (booking, sale edits, uploads) until they
   have been validated with your own account in Phase 4.
7. Store no PII in tracked files; captured responses belong in `secrets/`.
