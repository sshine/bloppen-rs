# Bookings, stands, products, and what is actually for sale

## Booking

A **booking** is one stand rental period at a Børneloppen store. It is not an
item and not "putting something up for sale".

The stand page has one `.booth` element per booking, with:

```text
data-booking-id="..."     the API's booking_id
data-booth="..."          the physical stand/booth number
data-commission="..."
data-discount="..."
data-booking-seen-info="..."
bookingFrom / bookingTo   the rental period
```

The booking also carries the store, webshop state, and the product rows that
belong to it. The API calls it a booking throughout:

- `saveBooking.json`
- `active.json` / `inactive.json` / `delete.json` / `unPrint.json`
- `infoSeen.json`
- `saveDiscount.json`
- `aktiverShop` / `deaktiverShop`

The Android app's UI talks about a **stand** or **booth**; the API and the
website's JavaScript talk about a **booking**. `BookingApi` in this crate follows
the API term: it is a handle to one stand rental period, not to one item.

## Product

A **product** is an item within a booking:

```text
product_id, temp_id
name, price, description
boerneloppen_theme_category_id, boerneloppen_theme_specification_id
active, big_item
image, image_2
```

The stand page calls the printable product labels **prismærker**. The search
endpoint calls the same things products and returns `name`, `price`, `size`,
`store`, `stand`, and image URLs.

## When is an item actually for sale?

A product is visible in the flea market when it is active in an open stand and
its price tag has been printed (and the webshop is active for web sales). The
relevant operations are:

| Operation | Endpoint |
| --- | --- |
| Show/hide a product | `active.json` / `inactive.json` |
| Print price tags | `printPriceTags` |
| Mark tags not printed | `unPrint.json` |
| Activate the stand's webshop | `aktiverShop` / `deaktiverShop` |

Adding a product does not print a tag and does not activate it for sale by
itself. The CLI and MCP server create new products **hidden** by default; pass
`--active` (CLI) or set `active: true` (MCP) to show one. Deleting a product
removes it; printing is the step that makes the physical label appear.

## Safety policy in this repository

The user's constraint is: **do not modify pre-existing items**.

The client therefore records product ids returned by its own successful
`save_products` call:

- CLI: `created-products.json` next to the session file.
- MCP: an in-process set.

Product-id operations (`active`, `inactive`, `delete`, `move`, `note`, `print`,
`unprint`, `upload` with a product id) refuse ids the client did not create. The
override is `--allow-existing-products` for the CLI and the MCP server.

`product add` creates new products and is allowed without the override. Stand
settings (discount, info-seen, shop activation) are not product-id operations,
but they still change the stand and are only served in the MCP write tier.

## What the past-booking error means

The live suite tried `infoSeen` on the first booking on the stand page. It was a
past rental period, and the site answered:

```json
{"status":"error","message":"Du har ikke adgang!"}
```

That is the site refusing a write to a stand that is not current; it is not an
authentication failure and not an item-level issue. A successful live write
needs a current booking.
