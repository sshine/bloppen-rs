# Live login-token validation

**Date:** 2026-11-06
**Build:** Børneloppen 1.0.4 (versionCode 23)
**Credentials:** the developer's own account via secretspec/keyring.

## `getLoginToken.json`

Requested with the authenticated cookie jar:

```text
GET /boerneloppen-theme/mobileApp/getLoginToken.json
Accept: application/json
Cookie: <session cookies>
```

Observed:

| Field | Value |
| --- | --- |
| HTTP status | 200 |
| Body size | 80 bytes |
| `status` | `"success"` |
| `token` | present, 49 bytes (value not recorded) |
| `message` | absent |

This matches `LoginTokenModel(status, token, message)` with nullable fields.
The success response does not carry a message.

For comparison, the anonymous request observed during Phase 3 returned HTTP
200 with `{"status":"error","message":"Log ind med brugernavn/adgangskode først."}`. The Rust client must check `status`, not the HTTP code.

## `loginWithToken.json`

A fresh cookie jar was used:

1. `GET /users/login` to obtain an anonymous `vmcms` session.
2. `POST /boerneloppen-theme/mobileApp/loginWithToken.json` with form field
   `token`.
3. `GET /mit-boerneloppen/forside` to verify the session.

Observed:

| Field | Value |
| --- | --- |
| HTTP status | 200 |
| Body size | 52 bytes |
| `status` | `"success"` |
| `message` | `"Du er nu logget ind"` |
| `token` | absent |
| New cookies | none beyond the anonymous login-page cookies |
| Home page after | authenticated, title `Mit Børneloppen` |

The token login **does not rotate or add a session cookie**. It marks the
existing `vmcms` session as logged in on the server. The resulting session is
authenticated, which is why the app can inject
`javascript:loginWithToken('<stored token>')` into a WebView that already has
the anonymous `vmcms` cookie.

## Implications

- The token is a re-entry credential, not a bearer token for the native JSON
  endpoints.
- A Rust client should create a cookie jar, `GET /users/login` for an anonymous
  session, `POST loginWithToken.json` with the token, and then reuse that jar.
- The token value is 49 bytes; its internal format is not needed and was not
  printed.
- No refresh endpoint was observed. Treat the token as long-lived until a
  `loginWithToken` call fails, then fall back to password login.

## Discrepancies with Phase 3

- Phase 3 expected login to rotate `vmcms`; live login does not.
- Phase 3 expected `loginWithToken` to set a session cookie; it instead relies
  on the pre-existing `vmcms` and server-side state.
- `getLoginToken` success has no `message`; `loginWithToken` success has a
  message and no token. Both fit the nullable model.

## Reproduction

```sh
just probe --token-login
```
