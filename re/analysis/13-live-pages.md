# Live authenticated-page and endpoint survey

**Date:** 2026-11-06
**Build:** Børneloppen 1.0.4 (versionCode 23)
**Session:** developer account via secretspec/keyring

## Page sweep

The probe requested every tab and menu page with the authenticated cookie jar.
All returned `200 OK` and `text/html; charset=UTF-8`.

| Page | Bytes | Title |
| --- | ---: | --- |
| home | 188,834 | `Mit Børneloppen` |
| stand | 248,330 | `Mit Børneloppen | Stande og prismærker` |
| sale | 397,788 | `Mit Børneloppen | Udbetaling` |
| club | 236,733 | `Mit Børneloppen | Loppeklub` |
| profile | 41,227 | `Mit Børneloppen | Indstillinger` |
| notifications | 86,275 | `Mit Børneloppen | Notifikationer` |
| faq | 44,171 | `Oftest stillede spørgsmål - Lær mere om Børneloppen` |
| find | 42,596 | `Søg efter varer - Find brugt børneudstyr, Børnetøj og Legetøj` |
| boost | 26,894 | `Mit Børneloppen | Boost dit salg` |

Two corrections to the Phase 3 domain map:

- The `sale` tab is the **payout** page (`Udbetaling`), not a list of sale
  items.
- The `stand` page is about stands and price tags (`Stande og prismærker`).

## A real JSON endpoint: product search

The `find` page uses:

```text
GET /boerneloppen-theme/searches/search.json
    ?query=lego&sku=&boerneloppen_theme_store_id=
```

Observed with and without an authenticated cookie jar:

| Field | Value |
| --- | --- |
| HTTP status | 200 |
| Content-Type | `application/json` |
| Body size | 34,906 bytes |
| Top-level keys | `data`, `pages`, `total` |
| `data` | array of 100 objects |
| `data[0]` keys | `discount`, `full_image_url`, `image_url`, `name`, `price`, `price_sold`, `size`, `sold`, `stand`, `store` |
| Authentication | not required |

This is the first confirmed read-only JSON domain endpoint. It supports the
"find goods" feature and gives the Rust client a non-HTML path for product
search. The response is paginated (`pages`, `total`); the exact page/size
parameters still need mapping.

## Internal AJAX endpoints found in page JavaScript

The pages contain Vue/jQuery code with endpoint constants. These are useful for
Phase 5 but were not all exercised, because several need POST bodies or
page-specific identifiers.

### Read-only candidates

| Endpoint | Method | Observed | Notes |
| --- | --- | --- | --- |
| `/boerneloppen-theme/searches/search.json` | GET | 200 JSON | Product search; works anonymously |
| `/customer/boerneloppen-theme/dashboard/getSalesHistory?today=1` | GET | 500 without expected parameters/headers | Used by the home sales modal |
| `/customer/boerneloppen-theme/dashboard/getSalesHistory?today=0` | GET | 500 without expected parameters/headers | Used by the home totals modal |
| `/customer/boerneloppen-theme/products/getProductsForPayout?booking_id=...` | GET | not probed without a booking id | Returns HTML markup into a container |
| `/boerneloppen-theme/bookings/getPaymentMethodsMarkup` | GET | 200 HTML | Markup fragment |
| `/customer/boerneloppen-theme/products/getMoveOptions.json` | AJAX | 500 without parameters | Needs a product/stand context |
| `/customer/boerneloppen-theme/products/getAvailablePrices.json` | AJAX | 500 without parameters | Needs a booking context |
| `/boerneloppen-theme/bookings/getClubGiftsMarkup` | AJAX | 500 without parameters | Markup fragment |

The 500s are not proof that an endpoint is broken; they mean the request lacked
the parameters or request shape the page uses. Each should be mapped from its
call site in Phase 5 before use.

### Write endpoints (not called)

These are deliberately excluded from the probe:

```text
/boerneloppen-theme/bookings/cleanUpBooth.json
/boerneloppen-theme/bookings/extendBooking.json
/customer/boerneloppen-theme/customer/buyClubRaffleTicket.json
/customer/boerneloppen-theme/customer/buyMailPrice.json
/customer/boerneloppen-theme/customer/subscribeNewsletter.json
/customer/boerneloppen-theme/products/active.json
/customer/boerneloppen-theme/products/delete.json
/customer/boerneloppen-theme/products/infoSeen.json
/customer/boerneloppen-theme/products/move.json
/customer/boerneloppen-theme/products/moveToCategory.json
/customer/boerneloppen-theme/products/noteActiveToggle/{id}.json
/customer/boerneloppen-theme/products/saveBooking.json
/customer/boerneloppen-theme/products/saveDiscount.json
/customer/boerneloppen-theme/products/unPrint.json
/customer/boerneloppen-theme/products/uploadImage
/gdpr/widgets/save.json
```

Page links also reference payout, receipt, delete, print-price-tag, shop
activate/deactivate, and scanner-feedback paths. Those are page-specific and
were not exercised.

## Decision for the Rust client

- **Product search:** use `search.json`; it is public, JSON, and already
  returns the fields a CLI needs.
- **Account pages:** start with HTML or the page's AJAX fragments. There is no
  single JSON API for loyalty, stands, sales, or notifications.
- **Writes:** do not automate them until their payloads and validation are
  mapped with the developer's own account. `BORNELOPPEN_TEST_WRITES` remains
  the opt-in gate.

## Sensitive output

The `--save-pages` option wrote the nine HTML pages to `secrets/pages/` for
local inspection. That directory is gitignored and may contain account data,
stand data, and product data; do not copy excerpts into tracked files.

## Reproduction

```sh
just probe --token-login --save-pages secrets/pages
just probe --no-pages --json-probes
```

The probe prints status, content type, sizes, titles, and JSON field names only;
it never prints page bodies.
