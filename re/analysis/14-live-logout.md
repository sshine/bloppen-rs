# Live logout and session-expiry behaviour

**Date:** 2026-11-06
**Build:** Børneloppen 1.0.4 (versionCode 23)
**Session:** developer account via secretspec/keyring

## Logout

`GET /users/logout` with the authenticated jar:

| Field | Value |
| --- | --- |
| HTTP status | 200 |
| Final URL | `/users/login` |
| Cookies before | `vmcms`, `hasLogin`, `HttpAccelerator`, `ShopWishlist_UserUUID`, `cookieSafe`, `vm-gdpr`, `vm-gdpr-accept` |
| Cookies after | the same names remain; `vmcms` is still present |
| `GET /mit-boerneloppen/forside` after | 200, final URL `/users/login?redirect=%2Fmit-boerneloppen%2Fforside`, title `Mit Børneloppen | Login`, authenticated=false |
| `GET getLoginToken.json` after | 200, `{"status":"error","message":"Log ind med brugernavn/adgangskode først."}` |

The server clears the login state without clearing the cookie jar. Checking for
the `vmcms` cookie is therefore **not** a session-validity check. The reliable
signals are the final URL of an authenticated page and the `status` field of
`getLoginToken.json`.

## Cookie lifetime

The `vmcms` cookie is set with `Max-Age=604800` (7 days) on the first
`GET /users/login`. Login does **not** refresh it:

```text
vmcms expiry before login: unix=1794609889 remaining=604800s
vmcms expiry after login:  unix=1794609889 remaining=604800s
```

A client that logs in on day 6 of an old cookie will have a session that expires
within a day. The client must either start a fresh cookie jar before logging in
or treat the expiry timestamp as the session deadline.

## What an expired/invalid session looks like

There is no 401 or 403 in the paths observed so far. The server answers:

| Request | Response |
| --- | --- |
| Authenticated HTML page | 200 with final URL `/users/login?redirect=...` and the login title |
| `getLoginToken.json` | 200 with `status:"error"` and the Danish "log in first" message |
| `search.json` | still 200 JSON; it is public and gives no auth signal |

So a robust client should:

1. Treat an authenticated page that ends on `/users/login` as `AuthExpired`.
2. Treat `getLoginToken.json` with `status != "success"` as `AuthExpired`.
3. Proactively re-authenticate when the stored `vmcms` expiry is within a
   safety margin (for example five minutes), because login does not extend it.
4. On `AuthExpired`, re-login once and retry the request if credentials are
   available; otherwise return the error.
5. Clear the local cookie jar and token on an explicit logout; do not rely on
   the server to remove the cookie.

## Reproduction

```sh
just probe --no-pages --logout
```

This logs out only the probe's session.
