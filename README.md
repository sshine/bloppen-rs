# bloppen

An async Rust client for the Danish [Børneloppen][boerneloppen] children's
flea-market site and the JSON endpoints used by its Android app.

It ships three things:

- **`bloppen-api`** — a typed library that keeps a Børneloppen web session,
  searches the public catalogue, fetches pages, and manages a stand's products.
- **`bloppen`** — a command-line client for the same operations.
- **`bloppen-mcp`** — a Model Context Protocol server that exposes the site to
  an LLM client over stdio.

Børneloppen is a Danish chain of children's flea-market supermarkets. This
project is unofficial and not affiliated with Børneloppen.

## What it can do

- Sign in with a normal account and persist a cookie session across runs.
- Search the public product catalogue with pagination.
- Fetch any public or authenticated page, including the account area.
- Add, update, show, hide, move, and delete products on a booked stand.
- Save stand discounts, toggle product notes, and print price tags.
- Upload product images.
- Activate or deactivate a stand's webshop.
- Reauthenticate automatically when the session expires.

Buying, payment and booking flows, and account administration are out of
scope. The client only performs the actions a stand holder can perform on
their own stand.

## Getting started

```rust
use bloppen_api::{Client, Credentials, SearchQuery};

let client = Client::builder()
    .credentials(Credentials::new("you@example.com", "password"))
    .session_file("/home/you/.local/share/bloppen/session.json")
    .build()?;

client.sign_in().await?;

let results = client.search(&SearchQuery::new("lego")).await?;
for product in &results.data {
    println!("{} — {}", product.name, product.price);
}
```

The session file is optional, but with it every later run reuses the stored
cookies and login token instead of starting from the login form.

## Bookings and products

A **booking** is one stand rental period; products are the items inside it.
Product operations therefore take a `booking_id`, while operations that work
on an existing item take a `product_id`. A product is visible in the webshop
when it is *active* on an *open* booking and its price tag has been printed;
new products are hidden by default.

See [`re/analysis/17-bookings-vs-products.md`](re/analysis/17-bookings-vs-products.md)
for the model behind these calls.

## Search

`Client::search` calls the public `searches/search.json` endpoint. No session
is required. Results are typed and paginated:

```rust
use bloppen_api::SearchQuery;

let query = SearchQuery::new("lego").page(2);
let results = client.search(&query).await?;

println!("{} products in {} pages", results.total, results.pages);
for product in &results.data {
    println!(
        "{:<40} {:>10}  store {}  stand {}",
        product.name,
        product.price,
        product.store.as_deref().unwrap_or("-"),
        product.stand.as_deref().unwrap_or("-"),
    );
}
```

`SearchQuery` also accepts a `sku` and a `store_id`, matching the site's own
search filters. Besides name and price, a `Product` carries size, store,
stand, sold price and marker, and image URLs.

## Pages

The account area is server-rendered HTML rather than JSON.
`Client::page(path)` returns the status, final URL after redirects, title, and
body. Paths under `/mit-boerneloppen/` use the stored session; public pages
work without one.

```rust
let page = client.page("/mit-boerneloppen/forside").await?;
println!("{} {} — {} bytes", page.status, page.final_url, page.bytes());
```

## Sessions and expiry

The primary credential is the `vmcms` cookie. It is issued with a seven-day
lifetime, and logging in again does **not** refresh it. Expiry is also not
reported as a `401`: a private page returns `200` after redirecting to
`/users/login`, and `getLoginToken.json` returns `200` with
`status: "error"`.

The client handles this in one place:

1. It reads the cookie expiry from `Expires` or `Max-Age`.
2. It signs in again when expiry is within a five-minute safety margin.
3. It treats a login redirect or `status: "error"` as `AuthExpired`.
4. It reauthenticates once and retries the request.
5. It falls back to the stored login token when no password is available.
6. It clears cookies, token, and the session file on logout.

The session file is written atomically with mode `0600`.

## Write operations

The library mirrors the AJAX calls the stand editor makes, including the
site's JSON-in-a-form-field encoding and bulk `product_ids[]` fields. Writes
are available on `Client` and through `Client::booking(booking_id)`:

```rust
use bloppen_api::ProductDraft;

let draft = ProductDraft::new("Lego bricks", "39,00")
    .with_category(12)
    .with_description("About 200 pieces");

let response = client
    .booking(123)
    .save_products(Some(12), None, &[draft])
    .await?;

println!("{:?}", response.new_product_ids);
```

The booking API covers `save_products`, `set_products_active`,
`delete_products`, `save_discount`, `unprint_products`, `set_shop_active`, and
`products_markup`. Stand-wide operations such as moving products between
categories, toggling a note, printing price tags, and uploading an image live
on `Client`.

## CLI

The `bloppen` binary is a thin front end over the library. Inside this
repository, use `just cli` in place of `bloppen` so the live credentials are
injected by secretspec.

```sh
bloppen auth login
bloppen auth status
bloppen search lego --limit 5
bloppen search lego --page 2 --json
bloppen page /mit-boerneloppen/forside
bloppen page /mit-boerneloppen/klub --raw
bloppen auth token              # redacted unless --show
bloppen auth logout

bloppen product list --booking-id 123 --raw
bloppen product add --booking-id 123 --name Lego --price 39,00 --category-id 12
bloppen product active --booking-id 123 --product-id 456
bloppen product inactive --booking-id 123 --product-id 456
bloppen product move --category-id 99 --product-id 456
bloppen product discount --booking-id 123 --percent 25 --from 2026-11-08 --to 2026-11-10
bloppen product note --product-id 456
bloppen product print --product-id 456
bloppen product unprint --booking-id 123 --product-id 456
bloppen product upload --product-id 456 --file photo.jpg
bloppen product delete --booking-id 123 --product-id 456 --yes
bloppen shop activate --booking-id 123
```

New products are created hidden; pass `--active` to `product add` to show one
immediately. `--json` switches supported commands to machine-readable output.
The session is stored at the XDG data path (`~/.local/share/bloppen/session.json`
on Linux) and can be overridden with `--session-file` or
`BORNELOPPEN_SESSION_FILE`.

### Write safety

Product-id operations only accept ids that this client created, tracked in a
sibling `created-products.json`. That keeps a mistyped id from changing an
unrelated listing. `--allow-existing-products` is the explicit override.
`product delete` additionally requires `--yes`.

## MCP server

`bloppen-mcp` serves JSON-RPC over stdio. It starts with three read-only tools:

| Tool | Purpose |
| --- | --- |
| `bloppen_session_status` | Session state, `vmcms` expiry, token presence |
| `bloppen_search_products` | Public product search with pagination and a limit |
| `bloppen_fetch_page` | HTML page fetch; private pages use the stored session |

With `--allow-writes` it adds `bloppen_save_products`,
`bloppen_set_products_active`, `bloppen_move_products`,
`bloppen_save_discount`, `bloppen_toggle_product_note`,
`bloppen_print_price_tags`, `bloppen_unprint_products`, and
`bloppen_set_shop_active`.

With `--allow-writes --allow-deletes` it also serves
`bloppen_delete_products`, which requires `confirm: true`. Passing
`--allow-deletes` alone is refused. Product-id tools apply the same
created-product guard as the CLI unless `--allow-existing-products` is given,
and the server caps listing rows with `--max-items` and page bodies with
`--max-bytes`.

Inspect the tool schemas without credentials:

```sh
just mcp-tools
```

Check the live session and exit without serving:

```sh
just mcp --check
```

An MCP client configuration, run from this repository:

```json
{
  "mcpServers": {
    "bloppen": {
      "command": "nix",
      "args": ["develop", "--command", "just", "mcp", "--allow-writes"]
    }
  }
}
```

## Credentials

Live commands read `BORNELOPPEN_EMAIL` and `BORNELOPPEN_PASSWORD` from the
environment. In this repository, `just` injects them through the configured
[secretspec] provider, which defaults to the system keyring:

```sh
just secrets-set     # prompts for both values
just secrets-check   # reports what is missing without printing values
```

Never pass credentials on a command line or store them in a tracked file. The
library keeps them in [`Secret`], which redacts itself in `Debug` and
`Display`, and never prints passwords, cookie values, or token values.

## Development

Enter the development shell, then run the checks:

```sh
nix develop
just ci
```

`just ci` runs formatting, clippy, tests, docs, the lock check, and
`cargo package` for every workspace member. The library's tarball is compiled
on its own; the CLI, MCP, and probe manifests are checked with `--no-verify`
because they depend on the unpublished `bloppen-api` path.

Useful recipes:

```sh
just test          # unit, wiremock, CLI, and MCP tests
just live-test     # real site, one test at a time, through secretspec
just showcase      # read-only CLI smoke test against the real site
just mcp-tools     # print the default read-only MCP schemas
just cli auth status
```

The mock suite covers login, reauthentication, search, pages, session
persistence, every write endpoint's path and encoding, multipart upload, and
the created-product guard. The MCP suite covers both tool tiers, a real client
over a pipe, and transport stream discipline. The live suite is opt-in and
covers sign-in, session status, public search, authenticated pages, token
re-entry, logout, and a safe write round-trip.

## Layout

| Path | Purpose |
| --- | --- |
| `crates/bloppen-api/` | HTTP client, session persistence, reauth, search, pages, writes |
| `crates/bloppen-cli/` | `bloppen` command-line client |
| `crates/bloppen-mcp/` | MCP server over stdio |
| `crates/bloppen-probe/` | Live validation probe used during reverse engineering |
| `re/` | PRD, analysis notes, extraction and decompilation scripts |
| `nix/` | Dendritic flake-parts modules |
| `backlog/` | Backlog.md task records |

The reverse-engineering record is in [`re/analysis/`](re/analysis/), the
product requirements in [`re/prd.md`](re/prd.md), and the working rules in
[`AGENTS.md`](AGENTS.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

[boerneloppen]: https://www.boerneloppen.dk
[secretspec]: https://github.com/sshine/secretspec
[`secret`]: https://docs.rs/bloppen-api/latest/bloppen_api/struct.Secret.html
