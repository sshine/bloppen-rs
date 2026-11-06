# Domain flows: loyalty, stands, sales, notifications, and menu

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`

## Key finding

The business domains are **server-rendered website pages inside WebViews**, not
native API resources. The app has no domain DTOs, no native list/detail
endpoints, and no JSON for loyalty, stands, sales, or notifications. Native code
provides the tab shell, login, push registration, notification routing, and a
JavaScript bridge; the website owns the data and the forms.

A Rust client that wants domain data has two options:

1. Fetch the same HTML pages with the `vmcms` cookie and parse them.
2. Find the website's internal JSON/AJAX endpoints (not exposed in
   `mobileapp.js`) and use those.

Phase 4 should establish which pages return useful HTML or JSON. Do not assume
the native models cover domain data; they do not.

## Tabs and menu

Four tabs are always mounted as WebViews, switched by `MainScreenViewState`:

| Tab | URL | Native tab value |
| --- | --- | --- |
| Home | `/mit-boerneloppen/forside` | `home` |
| Stand | `/mit-boerneloppen/stand` | `stand` |
| Sale | `/mit-boerneloppen/salg` | `sales` |
| Club | `/mit-boerneloppen/klub` | `club` |

Menu destinations open `MenuItemScreen` with a URL and tag:

| Menu item | URL | Tag |
| --- | --- | --- |
| Book a stand | `/book-en-stand?referer=mit-boerneloppen` | `stand` |
| Find goods | `/praktisk-info/soeg-efter-varer?referer=mit-boerneloppen` | `find` |
| Boost sales | `/mit-boerneloppen/boost-dit-salg` | `boost` |
| FAQ | `/praktisk-info/oftest-stillede-spoergsmaal?referer=mit-boerneloppen` | `faq` |
| Profile | `/mit-boerneloppen/profil` | `profile` |
| Notifications | `/mit-boerneloppen/notifikationer` | `notifications` |
| Log out | `/users/logout` | `logout` |

`Tabs` is the native enum behind the JavaScript bridge:
`home`, `stand`, `sales`, `club`, `book`, `find`, `boost`, `faq`, `profile`,
`notifications`. `gotoTab(name, options)` matches exactly and routes to the
corresponding tab or menu route.

## Loyalty / Club

| Concern | Detail |
| --- | --- |
| Page | `/mit-boerneloppen/klub` |
| Native tab | `club` |
| Event deep link | JS `gotoTab('club', '{"event":<id>}')` opens the event modal in the page |
| Badge | `handleBadgeCount.json?type=reset` when the notifications tab is opened |
| Data source | HTML and page JavaScript |

The loyalty card, points, and club events are rendered by the website. The app
does not read a points balance or loyalty identifier natively.

## Stands

| Concern | Detail |
| --- | --- |
| Page | `/mit-boerneloppen/stand` |
| Native tab | `stand` |
| Booking page | `/book-en-stand?referer=mit-boerneloppen` (menu tag `stand`) |
| Booking deep link | JS `gotoTab('stand', '{"booking":<id>}')` navigates to `.../stand?booking=<id>` |
| Data source | HTML and page JavaScript |

Stand selection, booking, and the booking summary live in the page. The JS
`tabActive` handler refuses to refresh while `summary.changesPending` is true,
which is a page-side unsaved-changes guard.

## Sales

| Concern | Detail |
| --- | --- |
| Page | `/mit-boerneloppen/salg` |
| Native tab | `sales` |
| Boost page | `/mit-boerneloppen/boost-dit-salg` (menu tag `boost`) |
| Data source | HTML and page JavaScript |

Sale editing and the boost flow are website forms. The native side supplies a
picture picker through the `cameraTriggered` bridge and a `FileProvider`; the
website handles the upload.

## Notifications and push

| Concern | Detail |
| --- | --- |
| Notification page | `/mit-boerneloppen/notifikationer` |
| Native tab | `notifications` |
| Channel | `channel_notifications` with the app's default channel name |
| Push service | `BoerneloppenFirebaseMessagingService` |
| Payload keys | `title`, `body`, `tab`, `pushClickedValue` |
| Intent keys | `tab`, `pushClickedValue`; legacy `tabToOpen` |
| Badge endpoint | `GET /boerneloppen-theme/mobileApp/handleBadgeCount.json?type=minus|reset` |

Flow:

1. `onNewToken(token)` saves the token and, if a `vmcms` cookie exists, posts it
   to `saveFirebaseToken.json`.
2. `onMessageReceived` builds a notification with the title/body and the
   `tab`/`pushClickedValue` data fields.
3. Tapping the notification starts `MainActivity` with those extras.
4. `MainActivity.handleIntent` calls `setTabFromNotification(tab, value)`.
5. The selected tab calls `pushClicked(value)` in the page; the page opens the
   matching event or booking.
6. Opening notifications triggers `handleBadgeCount('reset')`; `pushClicked`
   triggers `handleBadgeCount('minus')`.

The website also exposes `pushActivate()` and `pushCheck()`:

- `pushActivate` asks the native side for an FCM token and uploads it.
- `pushCheck` reads the notification permission and answers with
  `javascript:pushStatus(true|false)`.

## Find goods and search agent

| Concern | Detail |
| --- | --- |
| Page | `/praktisk-info/soeg-efter-varer?referer=mit-boerneloppen` |
| Menu tag | `find` |
| Search-agent deep link | JS `gotoTab('find', '{"searchagent":...}')` |

The JS for a search-agent option calls `gotoTab('agent')`. The native `Tabs`
enum has no `agent` value, so `Tabs.fromString("agent")` throws. Unless another
handler intercepts it, this is a website/native contract mismatch; treat the
search-agent path as unverified and do not build it into the Rust client
without live testing.

## Profile and consent

| Concern | Detail |
| --- | --- |
| Page | `/mit-boerneloppen/profil` |
| Menu tag | `profile` |
| Push settings | Page calls `pushCheck`; native answers `pushStatus` |
| Facebook consent | `showFacebookConsentScreen` and `FacebookConsentUseCase`; preference keys `request_facebook_consent`, `facebook_consent_status` |

The profile page is HTML. The native Facebook consent flag is local state used
by the Facebook SDK integration, not a wire resource.

## Scanner, QR, and pictures

| Concern | Detail |
| --- | --- |
| Scanner feedback | `/customer/boerneloppen-theme/customer/scannerFeedback/{scanId}` |
| Trigger | JS `gotoTab('notifications', '{"scan":<id>}')` |
| Camera bridge | JS `cameraTriggered()` → native picture dialog / file chooser |
| File provider | `dk.boerneloppen.boerneloppen.fileprovider`, external and cache paths |

The CAMERA permission and the `FileProvider` exist for page-driven uploads.
There is no native QR parser or scanner screen in the app code inspected here;
the scanner is a website page.

## Read-only versus write operations

| Operation | Kind | Endpoint |
| --- | --- | --- |
| Fetch home, stand, sale, club, profile, notifications, FAQ, find pages | Read | `GET` website pages with `vmcms` |
| Get native login token | Read (session-bound) | `GET getLoginToken.json` |
| Check notification permission | Read | `pushCheck` → `pushStatus` JS |
| Check badge count | Read-ish | `handleBadgeCount.json` (mutates badge) |
| Login | Write | `POST /users/login` |
| Restore session from token | Write | `POST loginWithToken.json` |
| Register FCM token | Write | `POST saveFirebaseToken.json` |
| Logout | Write | `GET /users/logout` |
| Stand booking, sale edits, profile updates, photo uploads | Write | Website forms (payloads not mapped) |
| Badge decrement/reset | Write | `handleBadgeCount.json` |

For an initial Rust CLI, prefer the read-only page fetches plus the login-token
and badge endpoints. Do not automate booking, sale edits, or uploads until the
HTML forms and their validation are mapped in Phase 4.

## Unknown or unmapped

- The website's internal XHR endpoints for list data, loyalty points, stand
  availability, and sale items are not in `mobileapp.js` and were not mapped
  here. They are the main investigation target if HTML parsing is undesirable.
- The exact HTML form fields for booking, sale editing, profile updates, and
  photo upload are page-specific.
- The `gotoTab('agent')` mismatch needs live confirmation.
- The `handleBadgeCount.json` response shape is only partially known:
  `{status, badge}` from the website JavaScript; no native model exists.
- The app's WebView tabs are long-lived; refresh happens through the page's
  `tabActive` logic, not through native polling.
