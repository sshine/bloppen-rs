# Write operations and MCP tiers

**Date:** 2026-11-07
**Scope:** stand and product writes; buying is deliberately excluded

## Endpoint catalogue

The website's stand editor is a jQuery/Vue page. Its AJAX calls use
`X-Requested-With: XMLHttpRequest` and the stand page as the referer; the client
sets both on write requests.

| Action | Method and path | Payload |
| --- | --- | --- |
| Product list markup | `GET /customer/boerneloppen-theme/products/getProductsForBooth` | `booking_id` query |
| Add/update products | `POST .../products/saveBooking.json` | form `json` = `{booking_id, category_id, move_to_category_id, products: [...]}` |
| Show/hide | `POST .../products/active.json` or `inactive.json` | `booking_id`, repeated `product_ids[]` |
| Delete | `POST .../products/delete.json` | `booking_id`, repeated `product_ids[]` |
| Unprint | `POST .../products/unPrint.json` | `booking_id`, repeated `product_ids[]` |
| Move to category | `POST .../products/moveToCategory.json` | repeated `product_ids[]`, `category_id` |
| Move to booking | `POST .../products/move.json` | repeated `product_ids[]`, `move_from_booking`, `move_to_booking` |
| Info seen | `POST .../products/infoSeen.json` | `booking_id` |
| Note toggle | `GET .../products/noteActiveToggle/{id}.json` | none |
| Discount | `POST .../products/saveDiscount.json` | form `json` = `{booking_id, percent, from, to}` |
| Shop active | `POST .../products/aktiverShop` or `deaktiverShop` | `booking_id` |
| Print tags | `GET .../products/printPriceTags` | `product_ids` (comma-separated), `page`, `max`; 64 per page |
| Upload image | `POST .../products/uploadImage` | multipart `image`, `product_id` or `temp_id`, optional `image_index` |

### Product row shape

The editor serializes every input in a changed product row. The important
fields are:

```text
product_id, temp_id, image, image_2,
newImage, newImage2, deleteImage, deleteImage2, _delete,
name, price, description,
boerneloppen_theme_category_id, top_category_id, sub_category_id,
boerneloppen_theme_specification_id,
active, big_item, order
```

Hidden flags travel as `"0"`/`"1"` strings; checkboxes as booleans. `ProductDraft`
mirrors that.

## Library

`bloppen-api` exposes a `BookingApi` from `Client::booking(id)`:

- `products_markup`, `save_products`, `set_products_active`, `delete_products`,
  `unprint_products`, `move_products_to_category`, `move_products`,
  `mark_info_seen`, `toggle_product_note`, `save_discount`, `set_shop_active`,
  `print_price_tags`, `upload_product_image`.

The operations that are not scoped to a booking (`move_products_to_category`,
`toggle_product_note`, `print_price_tags`, `upload_product_image`) are also on
`Client` directly.

`ProductDraft::new` creates a **hidden** product (`active: false`). The CLI adds
`--active` to show one immediately; the MCP tool defaults `active` to false.

Every write calls `ensure_authenticated` first. It uses the same expiry-aware
and reactive reauth path as reads.

## CLI

```sh
just cli product list --booking-id 123 --raw
just cli product add --booking-id 123 --name Lego --price 39,00 --category-id 12
just cli product active --booking-id 123 --product-id 456
just cli product inactive --booking-id 123 --product-id 456
just cli product move --category-id 99 --product-id 456
just cli product discount --booking-id 123 --percent 25 --from 2026-11-08 --to 2026-11-10
just cli product note --product-id 456
just cli product print --product-id 456
just cli product unprint --booking-id 123 --product-id 456
just cli product upload --product-id 456 --file photo.jpg
just cli product delete --booking-id 123 --product-id 456 --yes
just cli shop activate --booking-id 123
just cli shop deactivate --booking-id 123
```

Deleting requires `--yes`; the check happens before any request.

## MCP tiers

The server is read-only by default. The tiers are composed, not filtered, so a
tool that is not served cannot be reached by name.

| Flags | Tools |
| --- | --- |
| none | `bloppen_session_status`, `bloppen_search_products`, `bloppen_fetch_page` |
| `--allow-writes` | adds save products, set active, move, discount, note toggle, print, unprint, shop active |
| `--allow-writes --allow-deletes` | adds `bloppen_delete_products` |

`bloppen_delete_products` requires `confirm: true` and has
`destructive_hint: true`. `--allow-deletes` without `--allow-writes` is refused
rather than silently ignored.

Buying, booking extensions, payment selection, and stand cancellation are not
served. They involve real money and were not validated.

## Tests

- Eight wiremock tests assert the write paths, form encoding, the `json` field,
  repeated `product_ids[]`, the multipart upload, and response parsing.
- CLI tests cover the product/shop surface and the `--yes` guard.
- MCP tests cover the tier counts, write tools over a real client, the delete
  confirmation, and a failed write becoming a tool error.
- The live suite signs in and calls `mark_info_seen` on the first booking on the
  stand page. That booking is a past one, and the site answered
  `200 {"status":"error","message":"Du har ikke adgang!"}`. The client surfaced
  the error correctly. No successful live write was performed because the
  account has no current booking at validation time.

## Remaining work

- Live-test a successful write against an active booking.
- Map booking extension, cleanup, cancellation, and payment selection if they
  are ever wanted. Buying is explicitly out of scope.
- Discover category and specification ids from the page so `product add` can
  work without the caller supplying ids.
- Consider an MCP image-upload tool; the CLI already supports upload.
