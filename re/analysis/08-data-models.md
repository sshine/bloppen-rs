# Data models and JSON conventions

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`
**Serialization:** Gson via `GsonConverterFactory`

## API models

The native API returns three flat models. All fields are nullable Kotlin
`String`s; Gson does not enforce Kotlin nullability, so every field must be
treated as optional on the wire.

### `LoginTokenModel`

Source: `api/models/LoginTokenModel.kt`

| Kotlin field | JSON key | Type | Notes |
| --- | --- | --- | --- |
| `status` | `status` | `String?` | `"success"` or `"error"` in observed traffic |
| `token` | `token` | `String?` | Present on success; absent or null on error |
| `message` | `message` | `String?` | Human-readable, Danish in observed error |

No `@SerializedName` annotations; Gson uses the Kotlin property names directly.

Observed error body:

```json
{"status":"error","message":"Log ind med brugernavn/adgangskode først."}
```

Expected success shape (to confirm in Phase 4):

```json
{"status":"success","token":"...","message":"..."}
```

**Live result (2026-11-06):** the success body is
`{"status":"success","token":"..."}` with no `message`; the error body is
`{"status":"error","message":"..."}` with no token. `loginWithToken.json`
returns `{"status":"success","message":"Du er nu logget ind"}` with no token.
All fields remain nullable; absent fields deserialize as null.

### `ResponseModel`

Source: `api/models/ResponseModel.kt`

| Kotlin field | JSON key | Type | Notes |
| --- | --- | --- | --- |
| `status` | `status` | `String?` | The view model checks for `"success"` |
| `message` | `message` | `String?` | Optional human-readable text |

Used by `saveFirebaseToken.json`. No annotations.

### `ErrorResponseModel`

Source: `api/models/ErrorResponseModel.kt`

| Kotlin field | JSON key | Type | Notes |
| --- | --- | --- | --- |
| `error` | `error` | `String?` | Explicit `@SerializedName("error")` |
| `message` | `message` | `String?` | Explicit `@SerializedName("message")` |

`isSuccess()` is true only when both fields are null. `RetrofitErrorInterceptor`
uses this model for non-2xx responses whose `Content-Type` is
`application/json` or `application/json;charset=UTF-8`.

## Serialization behaviour

- `ModuleKt` builds Gson with `setLenient()`,
  `enableComplexMapKeySerialization()`, and `setPrettyPrinting()`. The pretty
  printing affects only serialization, not parsing.
- There is no field-naming policy, no date adapter, and no custom enum adapter.
  Field names map 1:1 to JSON keys.
- `api/extensions/FromJsonKt.fromJson` is a reified helper around
  `Gson.fromJson`; it does not alter field mapping.
- The app never uses a Kotlin `enum` on the wire. The only enum, `Tabs`, is a
  UI navigation enum, not an API type.

## UI-side models (not part of the API contract)

These are useful for understanding the app but should not be copied into the
Rust client:

| Type | Purpose |
| --- | --- |
| `MainScreenViewState` | Compose state: 18 fields covering loading flags, URLs, dialogs, notification tab/value, picture callback, and Facebook consent |
| `MenuItem` | `(title, url, tag)` for the menu WebViews |
| `Tabs` | `home`, `stand`, `sales`, `club`, `book`, `find`, `boost`, `faq`, `profile`, `notifications`; `fromString` matches exactly and throws on unknown values |
| `Alert` | UI alert with id, `IAlertMessage`, colour, icon and content-description resources |
| `IAlertMessage`, `Error`, `Success`, `Warning` | Sealed alert-message variants; resource IDs or strings |
| `MainScreenViewState` loading flags | `homePageIsLoading`, `standPageIsLoading`, `salePageIsLoading`, `clubPageIsLoading` |

`MenuItem` and `Tabs` are navigation metadata, not server data. `Alert` and the
message variants are local UI resources.

## Status conventions

The app has two different error conventions:

1. **HTTP 200 with `status == "error"`** — the normal auth/business failure for
   the mobileApp JSON endpoints. `LoginTokenModel.status` is the signal; the
   HTTP status is 200.
2. **Non-2xx with `{error, message}`** — handled by `RetrofitErrorInterceptor`.
   403 becomes `ForbiddenException`; other non-2xx JSON becomes
   `GenericApiError`; non-JSON becomes a generic message.

A Rust client should therefore:

- Deserialize 2xx bodies into `LoginTokenModel`/`ResponseModel` regardless of
  the HTTP status.
- Check `status == "success"` explicitly.
- Deserialize non-2xx bodies into `ErrorResponseModel`, but fall back to the
  HTTP status when the body is not JSON.
- Treat `token` and `message` as optional, and never require a `message`.

## Enums and values seen so far

| Value | Context |
| --- | --- |
| `success` | Expected success status for login-token and Firebase-token responses |
| `error` | Observed auth failure status |
| `login` | Fragment of the login URL checked by the WebView (`onPageFinished`) |
| `home`, `stand`, `sales`, `club`, `book`, `find`, `boost`, `faq`, `profile`, `notifications` | UI tab names passed to the website's `tabActive`/`gotoTab` bridge |
| `minus`, `reset` | `handleBadgeCount.json` query values |

## Unknowns for Phase 4

- Exact success body for `getLoginToken.json`, `loginWithToken.json`,
  `saveFirebaseToken.json`, and `handleBadgeCount.json`.
- Whether `token` or `message` is ever an empty string rather than null.
- Whether the website returns additional fields that Gson silently ignores.
- Whether any domain data is available as JSON (none is visible in the native
  app; the tabs are HTML).
- Whether `status` uses any value other than `success`/`error` (for example a
  numeric code or a localisation key).
