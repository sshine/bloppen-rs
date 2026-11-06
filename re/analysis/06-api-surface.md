# API surface and base URL

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`
**Native base URL:** `https://www.boerneloppen.dk/`
**WebView pages:** same host

## Two API layers

The app uses the same host for two different things:

1. **Native JSON API** — two Retrofit calls under
   `/boerneloppen-theme/mobileApp/`. These are the only calls that go through
   OkHttp and Retrofit.
2. **Server-rendered website** — the authenticated tabs and menu entries load
   HTML pages in WebViews. The website's `mobileapp.js` also exposes a few JSON
   helpers that the native app does not call through Retrofit.

A Rust client that mirrors the native app needs the first layer for login-token
and push registration, and must decide in Phase 4 whether to use the website
endpoints or parse the HTML for the rest.

## Base URL and flavors

`BuildConfig.BASE_URL` is fixed at `https://www.boerneloppen.dk/` in this
release build (`FLAVOR = boerneloppen_dk`). The `Constants` class contains URLs
for the other market flavors and a staging host, but the flavor is chosen at
build time; there is no runtime environment switch.

| Flavor | Base host in `Constants` |
| --- | --- |
| `boerneloppen_dk` (this APK) | `https://www.boerneloppen.dk/` |
| `boerneloppen_be` | `https://www.borneloppen.be/` |
| `barnaloppan_is` | `https://www.barnaloppan.is/` |
| `boerneloppen_se` | `https://www.borneloppen.se/` |
| `extraloppan_is` | `https://www.extraloppan.is/` |
| `staging` | `http://boerneloppen.dk.test.vjm.dk/` |

## Native Retrofit endpoints

`RetrofitBoerneloppenApi` is the complete native interface:

| Method | Path | Headers | Parameters | Response |
| --- | --- | --- | --- | --- |
| `GET` | `boerneloppen-theme/mobileApp/getLoginToken.json` | `Accept: application/json`, `Cookie: vmcms=...` | none | `LoginTokenModel` |
| `POST` | `boerneloppen-theme/mobileApp/saveFirebaseToken.json` | `Accept: application/json`, `Cookie: vmcms=...` | form field `token` | `ResponseModel` |

`Cookie` is passed as a raw header, not through an OkHttp cookie jar. The value
is the selected `vmcms=...` pair from `CookieManager.getCookie(homePageUrl)`,
with no extra attributes.

### Live response conventions

A request to `getLoginToken.json` without an authenticated session returns
**HTTP 200** with a JSON error body, not 401:

```json
{"status":"error","message":"Log ind med brugernavn/adgangskode først."}
```

The app therefore treats `LoginTokenModel.status` as the success signal, not
the HTTP status. `RetrofitErrorInterceptor` only handles non-2xx responses and
403; a 200 with `"status":"error"` reaches the view model as a model value.

A successful `getLoginToken.json` is expected to return:

```json
{"status":"success","token":"...","message":"..."}
```

Exact success shape is a Phase 4 validation item.

`saveFirebaseToken.json` is expected to return `ResponseModel` with
`status == "success"`; the view model checks that string before marking the
token as uploaded.

## Website JSON helpers (not called by the native app)

`mobileapp.js` on the website defines these endpoints. The comments say they
"should not be used if app can call json direct with session cookie", which is
why only the two Retrofit endpoints above appear in DEX.

| Method | Path | Parameters | Response |
| --- | --- | --- | --- |
| `GET` | `/boerneloppen-theme/mobileApp/getLoginToken.json` | none (session cookie) | `{status, token, message}` |
| `POST` | `/boerneloppen-theme/mobileApp/loginWithToken.json` | form `token` | `{status, ...}`; sets the session cookie on success |
| `POST` | `/boerneloppen-theme/mobileApp/saveFirebaseToken.json` | form `token` | `{status, ...}` |
| `GET` | `/boerneloppen-theme/mobileApp/handleBadgeCount.json` | query `type=minus` or `type=reset` | `{status, badge}` |

`loginWithToken.json` is the endpoint behind the biometric login path. The
native app does not call it directly; it injects
`javascript:loginWithToken('<stored token>')` into the login WebView, and the
website JS posts the token.

## Website pages used by the WebViews

| Tab or menu item | URL |
| --- | --- |
| Login / register | `/users/login` |
| Logout | `/users/logout` |
| Home | `/mit-boerneloppen/forside` |
| Stand | `/mit-boerneloppen/stand` |
| Sale | `/mit-boerneloppen/salg` |
| Club | `/mit-boerneloppen/klub` |
| Notifications | `/mit-boerneloppen/notifikationer` |
| Profile | `/mit-boerneloppen/profil` |
| Boost sales | `/mit-boerneloppen/boost-dit-salg` |
| Book a stand | `/book-en-stand?referer=mit-boerneloppen` |
| Find goods | `/praktisk-info/soeg-efter-varer?referer=mit-boerneloppen` |
| FAQ | `/praktisk-info/oftest-stillede-spoergsmaal?referer=mit-boerneloppen` |
| Scanner feedback | `/customer/boerneloppen-theme/customer/scannerFeedback/{scanId}` |

The login page's form is:

```html
<form method="post" accept-charset="utf-8" action="/users/login">
  <input type="email" name="email">
  <input type="password" name="password">
  <input type="submit" value="Log ind">
</form>
```

There is no CSRF token in the login form as served to a fresh client.

## Headers, interceptors, and authentication

| Concern | Behaviour |
| --- | --- |
| `Accept` | `application/json` is set per Retrofit method; the interceptor named `AcceptTypeHeaderChangerInterceptor` is currently a no-op |
| `Cookie` | Passed per call as a raw header; there is no cookie-jar auth interceptor |
| User-Agent (WebView) | `BoerneloppenApp` from `@string/user_agent` |
| User-Agent (OkHttp) | OkHttp default; no explicit app User-Agent is set for native calls |
| Offline | `NetworkInterceptor` throws `NetworkException` before the request |
| 403 | `RetrofitErrorInterceptor` throws `ForbiddenException` |
| Other non-2xx | JSON error body becomes `GenericApiError`; non-JSON becomes a generic message |
| Logging | `HttpLoggingInterceptor` at `BODY` and Beagle's network logger are active in release |

There is no CSRF header and no bearer token. The only credential used by the
native endpoints is the `vmcms` cookie.

## Authentication marking

| Endpoint | Needs `vmcms` cookie? | Needs login token? |
| --- | --- | --- |
| `GET getLoginToken.json` | Yes, and the session must be logged in | No |
| `POST saveFirebaseToken.json` | Yes, and the session must be logged in | No |
| `POST loginWithToken.json` | No; the token establishes the session | Yes (stored login token) |
| `GET handleBadgeCount.json` | Yes | No |
| Website tabs and menu pages | Yes | No |

`getLoginToken.json` with an anonymous `vmcms` cookie returns the Danish "log in
first" error above, so an anonymous cookie is not enough.

## Gaps and unreachable surface

- No other Retrofit interface exists in the app; searching the DEX for
  `retrofit2.http` annotations finds only the two methods above.
- `GetVmcmsCookieUseCase` is present but the inspected call sites use
  `VmcmsCookieUseCase` and `MainRepository.getVmcmsCookie()`.
- The `access_token` preference key is declared but not written by the code
  inspected here. The login token lives under `login_token` and
  `boerneloppen_login_token`.
- The website's full HTML endpoints (stand booking, sale editing, loyalty
  history) are not mapped here. They are Phase 3 domain-flow targets; the Rust
  client should not assume they are JSON.
