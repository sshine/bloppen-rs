# Session robustness: expiry, reauth, and logout

**Date:** 2026-11-06
**Scope:** `bloppen-api` and its callers

## What the live site does

- `GET /users/login` sets `vmcms` with `Max-Age=604800` (seven days).
- `POST /users/login` does **not** refresh that expiry; it adds login-state
  cookies such as `hasLogin`.
- An expired or logged-out session is not a 401:
  - a private page returns `200` and redirects to `/users/login?redirect=...`;
  - `getLoginToken.json` returns `200` with `status:"error"` and a Danish
    "log in first" message;
  - the public `search.json` keeps returning `200` and gives no auth signal.
- `GET /users/logout` clears the server-side login state but leaves cookie
  names in the jar.

## What the client does

### Expiry awareness

- Reads the `vmcms` expiry from the cookie store, handling both `Expires` and
  `Max-Age`.
- Treats a session as unusable when the cookie is missing, untrusted, or
  within a five-minute `DEFAULT_REAUTH_MARGIN`.
- Sign-in starts a fresh cookie jar, so a reauthentication gets a new
  seven-day cookie rather than extending a stale one.

### Proactive reauth

Before an authenticated page fetch, `ensure_authenticated` checks the margin.
If the cookie is close to expiry, it signs in again using the stored
credentials, or the login token when no password is available.

### Reactive reauth

A private page whose final URL is the login page, or whose title contains
"login", is treated as `AuthExpired`. The session is marked untrusted, one
fresh login is attempted, and the page fetch is retried exactly once. The same
untrust-and-retry path covers `getLoginToken.json` returning `status:"error"`.

The untrust step matters: without it, `reauth` would see a still-unexpired
cookie and refuse to replace the session that the server has just rejected.

### Token fallback

`login_token` stores the 49-byte native token. `login_with_token` seeds a fresh
cookie jar with `GET /users/login` and posts the token to
`loginWithToken.json`; the server marks the existing `vmcms` session as logged
in without setting a new cookie. This is the biometric-style re-entry path.

### Persistence

- Cookies and token are saved to one JSON file via `cookie_store`'s serializer.
- Writes go through a temporary file and rename, with mode `0600` set before
  the write.
- Restore is best-effort: unreadable, wrong-version, or absent files mean a
  login rather than an error.

### Logout

`sign_out` calls `/users/logout`, then clears the cookie jar, the token, and
the session file even if the request fails. This is deliberately stronger than
the Android app, which leaves the cookie and token on disk.

## Coverage

Mocked tests (`crates/bloppen-api/tests`):

- successful password login and token capture;
- rejected password (`AuthError::Rejected`);
- unauthenticated status without reauth;
- a private page redirecting to login, triggering exactly one reauth;
- a cookie inside the reauth margin triggering a proactive reauth;
- session-file restore without a second login;
- logout removing the session file;
- foreign-origin rejection;
- public search parsing and decode failure;
- public vs private page access.

Live tests (`crates/bloppen-api/tests/live.rs`, `#[ignore]` + secretspec):

- sign-in, status, and expiry;
- public search;
- authenticated home page;
- token re-entry in a fresh jar;
- logout and local file removal.

## Remaining uncertainty

- The server may change the expiry semantics in a future build; the client
  trusts the cookie's own expiry and the reactive login-redirect signal, so a
  change in the numeric margin is handled.
- There is no refresh endpoint. If the login token itself expires, password
  login is the fallback.
