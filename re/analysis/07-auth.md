# Authentication and session flow

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`
**Chosen path:** username/password login on the website + the `vmcms` session
cookie, with the native login token used only for biometric re-entry.

## Credentials and cookie

The app has no native username/password form. The login screen is a WebView
pointed at:

```text
https://www.boerneloppen.dk/users/login
```

The form is ordinary HTML:

```html
<form method="post" accept-charset="utf-8" action="/users/login">
  <input type="email" name="email">
  <input type="password" name="password">
  <input type="submit" value="Log ind">
</form>
```

No CSRF field was present in the served form.

The server sets the session cookie on `GET /users/login` even before login:

```text
Set-Cookie: vmcms=<session>; expires=<+7 days>; Max-Age=604800;
            path=/; secure; HttpOnly; SameSite=Lax
```

After a successful login the server is expected to rotate or replace `vmcms`
for the authenticated session. The app does not care about the old value: it
re-reads the cookie after the page reports login success.

The login WebView sets its user agent to `BoerneloppenApp` from
`@string/user_agent` and enables JavaScript.

## How the app learns that login succeeded

The website's `mobileapp.js` calls the native bridge:

```js
function onLoginSuccess() {
  if (window.Android) window.Android.loginSuccess();
  else if (window.webkit) window.webkit.messageHandlers.loginSuccess.postMessage(...)
}
```

The native bridge is registered as `Android` on the login WebView and on every
menu WebView. `WebAppInterface.loginSuccess()` then:

1. Reads `CookieManager.getInstance().getCookie(loginUrl)`.
2. Splits the cookie string on `;`.
3. Picks the first entry containing the substring `"vmcms"`.
4. Stores that raw `name=value` string via `VmcmsCookieUseCase` under the
   `vmcms_cookie` preference.
5. If `GetLoginTokenUseCase()` is blank, calls
   `RequestLoginTokenUseCase(cookie)`.
6. Sets the login state to `true`.

`CookieManager.getCookie` returns `name=value; name2=value2`, so the stored
value is the `vmcms=...` pair, not the `Set-Cookie` attributes.

The state update happens **even when no `vmcms` cookie is found**. The method
always calls `onUpdateLoginStatus(true)` at the end. A successful page callback
is therefore treated as proof of login, independently of the cookie check.

## Getting the native login token

`RequestLoginTokenUseCase` calls `MainRepository.requestLoginToken(cookie)`,
which calls:

```text
GET https://www.boerneloppen.dk/boerneloppen-theme/mobileApp/getLoginToken.json
Accept: application/json
Cookie: vmcms=<session>
```

The response model is `LoginTokenModel(status, token, message)`.

An unauthenticated request returns HTTP 200 with:

```json
{"status":"error","message":"Log ind med brugernavn/adgangskode først."}
```

A successful response is expected to return `status == "success"` and a
non-null `token`. The app does not check the status: it saves `token` whenever
the parsed field is non-null, and swallows every exception around the call.
If the request fails, the user is still considered logged in from step 6 above;
only the login token is missing.

### Token storage

`MainRepository.saveLoginToken(token)`:

1. Calls `KeystoreHelper.encrypt(EncryptionSecret("boerneloppen_login_token", token))`.
2. Generates an AES-128 key in `AndroidKeyStore` under the alias
   `boerneloppen_login_token`, with GCM and no padding.
3. Encrypts the token and Base64-encodes data and IV.
4. Writes JSON `{"alias", "data", "iv"}` to the `login_token` SharedPreferences
   key.

`getLoginToken()` reverses that: reads the JSON, decrypts with the Keystore
key, and returns the plaintext. The VMCMS cookie is **not** encrypted; it is
stored as plain text under `vmcms_cookie`.

## What the login token is for

The token is not sent to the native JSON API. The app injects it into the
website:

```js
javascript:loginWithToken('<stored token>')
```

The website's `mobileapp.js` implements:

```text
POST /boerneloppen-theme/mobileApp/loginWithToken.json
Content-Type: application/x-www-form-urlencoded

token=<stored token>
```

On success the website sets the session cookie and returns `status: success`.
The native app does not read that response; it relies on the WebView's cookie
jar.

So there are two credentials:

| Credential | Where used | Lifetime |
| --- | --- | --- |
| `vmcms` cookie | Every native JSON call and every WebView page | Server session, about 7 days |
| Encrypted login token | Website `loginWithToken` only | Not documented; used for biometric re-entry |

## Biometric login

There are two biometric entry points.

### Explicit bridge call

The website can call `initBiometricsLogin()`. The bridge method:

1. If the stored login token is blank, opens a "biometrics missing" alert.
2. Otherwise injects `javascript:loginWithToken('<token>')` into the WebView.
3. In the callback, checks `BiometricManager.canAuthenticate(255)`.
4. On a usable device, shows `BiometricPrompt`.
5. On success, reads the cookie again, saves it, and sets login state true.

### Automatic prompt after the login page loads

`LoginScreen`'s `onPageFinished` does the same when:

- the stored login token is not blank, and
- the page URL contains the string `login`.

That means a returning user who has a token can be prompted for biometrics
before typing a password again.

## Sequence

```text
User            App/WebView                 Website                    Native API
 |                   |                         |                          |
 |-- open app ------>|                         |                          |
 |                   |-- GET /users/login ---->|                          |
 |                   |<-- 200 + vmcms cookie --|                          |
 |-- email/password->|-- POST /users/login --->|                          |
 |                   |<-- 302/200 + new vmcms -|                          |
 |                   |<-- onLoginSuccess() ----|                          |
 |                   |-- Android.loginSuccess()                          |
 |                   |-- read CookieManager                            |
 |                   |-- save vmcms_cookie                            |
 |                   |-- GET getLoginToken.json + Cookie -------------->|
 |                   |<-- {status,token,message} ----------------------|
 |                   |-- save token in AndroidKeyStore                 |
 |                   |-- is_logged_in = true                           |
 |                   |-- navigate to webViewsScreen                    |
```

## Logout

The menu's logout item opens `/users/logout` in a WebView. `MenuItemScreen`
detects that URL and calls `MainScreenViewModel.logout()`, which calls
`UpdateLoginStatusUseCase(false)`. That only writes `is_logged_in = false`.

`MainRepository.logout()` exists and would prune the entire SharedPreferences
file, but the inspected call sites do not use it: `LogoutUseCase` is bound in
Koin but not injected into `MainScreenViewModel`. Consequences:

- The stored `vmcms_cookie`, encrypted `login_token`, `firebase_token`, and
  Facebook consent preferences survive logout.
- The server-side session is cleared by `/users/logout`, and the WebView cookie
  jar is updated by the website, but the app's saved copy is stale.
- The stored login token remains usable for `loginWithToken`, so biometric
  login can restore a session after a logout that only flipped the flag.

The Rust client should implement the intended behaviour rather than the
observed bug: clear the cookie jar, clear the stored token, and call
`/users/logout`.

## Open questions for Phase 4

1. Exact success body and status for `getLoginToken.json`, and whether `token`
   is always present.
2. `loginWithToken.json` success body, cookie rotation, and failure mode.
3. Login token expiry and whether any refresh endpoint exists (none is visible
   in the app or `mobileapp.js`).
4. Whether the login POST needs any hidden field, header, or CSRF value beyond
   what a fresh client receives.
5. The redirect target and status code after a successful login, and whether a
   failed login stays on `/users/login` with HTML or JSON.
6. Whether the server enforces 2FA, CAPTCHA, or rate limiting for the account
   used in testing.

## Recommended Rust login flow

1. Use a `reqwest` cookie jar.
2. `GET https://www.boerneloppen.dk/users/login` to obtain the anonymous
   `vmcms` cookie and any other session cookies.
3. `POST https://www.boerneloppen.dk/users/login` with
   `email`, `password`, and the cookie jar, following redirects.
4. Confirm the response set a new `vmcms` cookie and that a logged-in page
   (`/mit-boerneloppen/forside`) returns authenticated content.
5. `GET /boerneloppen-theme/mobileApp/getLoginToken.json` with the cookie jar.
   Require `status == "success"` and a non-empty `token`.
6. Persist the cookie and token under `~/.local/share/bloppen/` with restrictive
   permissions; do not log either value.
7. For re-entry, `POST /boerneloppen-theme/mobileApp/loginWithToken.json` with
   the stored token and let the cookie jar absorb the new `vmcms`.
8. On logout, clear the jar and token, then `GET /users/logout`.
