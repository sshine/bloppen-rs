# Live login and VMCMS cookie validation

**Date:** 2026-11-06
**Build:** Børneloppen 1.0.4 (versionCode 23)
**Credentials:** the developer's own account, supplied by `secretspec run` from
the keyring. No credential value is recorded here.

## Command

```sh
just probe --token-login
```

The probe performs `GET /users/login`, `POST /users/login` with
`email`/`password`, then `GET /mit-boerneloppen/forside`.

## Result

| Step | Status | Final URL | Evidence |
| --- | --- | --- | --- |
| `GET /users/login` | 200 | `/users/login` | Sets anonymous session cookies |
| `POST /users/login` | 200 | `/mit-boerneloppen/forside` | Authenticated page reached |
| `GET /mit-boerneloppen/forside` | 200 | `/mit-boerneloppen/forside` | Title `Mit Børneloppen`; authenticated |

No CAPTCHA or 2FA challenge appeared.

## Cookies

Only names and lengths are recorded; values are not.

| Cookie | Before login | After login |
| --- | --- | --- |
| `vmcms` | 32 bytes | 32 bytes, **unchanged** |
| `hasLogin` | absent | 1 byte |
| `HttpAccelerator` | absent | 14 bytes |
| `ShopWishlist_UserUUID` | absent | 36 bytes |
| `cookieSafe` | 35 bytes | 35 bytes |
| `vm-gdpr` | 86 bytes | 86 bytes |
| `vm-gdpr-accept` | 1 byte | 1 byte |

The important finding is that **`vmcms` does not rotate at login**. It is a
stable session identifier. The server marks that session as logged in and
returns additional state cookies (`hasLogin`, `HttpAccelerator`,
`ShopWishlist_UserUUID`). This corrects the Phase 3 expectation that login
would replace `vmcms`.

The app's `LoginScreen` reads `CookieManager.getCookie(...)`, keeps the entry
containing `vmcms`, and sends it as the `Cookie` header. That still works
because the session ID is unchanged and the server-side state is what changed.
A Rust client must keep the whole cookie jar, not only `vmcms`, because the
WebView pages may rely on the additional cookies.

## Failed login

A deliberately fake account (`nobody@example.invalid` with a random password)
was used to exercise the failure path. This is not the real account and no
real credential was sent.

| Step | Status | Final URL | Evidence |
| --- | --- | --- | --- |
| `POST /users/login` | 500 | `/users/login` | Server error for the fake account |
| `GET /mit-boerneloppen/forside` | 200 | `/users/login?redirect=%2Fmit-boerneloppen%2Fforside` | Title `Mit Børneloppen | Login`; authenticated=false |

The probe reports the failure clearly and exits non-zero. A 500 is specific to
this fake-account attempt; a real account with a wrong password may return a
different status. The reliable success test is the final URL and page title,
not the HTTP status alone.

## Implications for the Rust client

- Use a cookie jar and keep every cookie the server sets.
- `POST /users/login` with `email` and `password` and follow redirects.
- Treat a final URL under `/mit-boerneloppen/` with a non-login title as
  success.
- Do not expect `vmcms` to change at login.
- Store only the cookie jar needed for the session; clear it on logout.

## Reproduction

```sh
just probe --token-login
```

The probe redacts passwords, cookie values, and token values.
