# bloppen-rs

Børneloppen.dk API + CLI + MCP.

An unofficial Rust client for the Danish **Børneloppen** children's flea-market
site and the Android app that wraps it. Not affiliated with Børneloppen.

## Status

- **Phase 1–4 complete:** the APK was acquired, decompiled, analysed, and the
  real login, cookie, token, and page flows were validated against the live
  site.
- **Phase 5–6 delivered:** a typed API library, a CLI, and a read-only MCP
  server.
- **Phase 7+:** more domain endpoints, write operations, and packaging remain.

The reverse-engineering record is in [`re/analysis/`](re/analysis/), the PRD in
[`re/prd.md`](re/prd.md), and the working rules in [`AGENTS.md`](AGENTS.md).

## Build

```sh
nix develop
just ci
```

`just ci` runs formatting, clippy, tests, docs, the lock check, and
`cargo package` for every workspace member.

## Credentials

Live commands use the account email/password from the configured secretspec
provider. The default provider here is the system keyring:

```sh
just secrets-set     # prompts for BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD
just secrets-check   # reports what is missing without printing values
```

`just cli`, `just showcase`, `just live-test`, `just probe`, and `just mcp` wrap
the process in `secretspec run`, so no secret is read from argv or a tracked
file.

## CLI

```sh
just cli auth login
just cli auth status
just cli search lego --limit 5
just cli search lego --page 2 --json
just cli page /mit-boerneloppen/forside
just cli page /mit-boerneloppen/klub --raw
just cli auth token            # redacted
just cli auth logout
```

The session is stored at the XDG data path (`~/.local/share/bloppen/session.json`
on Linux) and can be overridden with `--session-file` or
`BORNELOPPEN_SESSION_FILE`. `--json` is available on `auth`, `search`, and
`page` where it makes sense.

`just showcase` runs the read-only login/status/search/page/token sequence
against the live site.

## MCP server

The server speaks JSON-RPC over stdio and serves three read-only tools:

| Tool | Purpose |
| --- | --- |
| `bloppen_session_status` | Session state, `vmcms` expiry, token presence |
| `bloppen_search_products` | Public product search with pagination and a limit |
| `bloppen_fetch_page` | HTML page fetch; private pages use the stored session |

Inspect the schemas without credentials:

```sh
just mcp-tools
```

Check the live session and exit:

```sh
just mcp --check
```

Example MCP client configuration, run from this repository:

```json
{
  "mcpServers": {
    "bloppen": {
      "command": "nix",
      "args": ["develop", "--command", "just", "mcp"]
    }
  }
}
```

## Library

```rust
use bloppen_api::{Client, Credentials, SearchQuery};

let client = Client::builder()
    .credentials(Credentials::new(email, password))
    .session_file(path)
    .build()?;

client.sign_in().await?;

let results = client.search(&SearchQuery::new("lego")).await?;
for product in &results.data {
    println!("{} — {}", product.name, product.price);
}
```

Exports include `Client`, `ClientBuilder`, `Credentials`, `Secret`,
`SessionStatus`, `SearchQuery`, `SearchResults`, `Product`, `Page`, `Error`,
`AuthError`, and `Result`.

## Session and expiry

The `vmcms` cookie is the primary credential. The live validation established
that:

- it is set with a seven-day `Max-Age`;
- logging in does **not** refresh it;
- expiry is not a 401. A private page returns `200` and redirects to
  `/users/login`, while `getLoginToken.json` returns `200` with
  `status:"error"`.

The client therefore:

1. reads the cookie expiry (from `Expires` or `Max-Age`);
2. reauthenticates when it is within a five-minute safety margin;
3. treats a login redirect or `status:"error"` as `AuthExpired`;
4. reauthenticates once and retries the request;
5. falls back to the stored login token when no password is available;
6. clears cookies, token, and the session file on logout.

The session file is written atomically with mode `0600`.

## Tests

```sh
just test          # unit, wiremock, CLI, and MCP tests
just live-test     # real site, one test at a time, through secretspec
just showcase      # read-only CLI smoke test against the real site
just ci            # full local CI
```

The live suite covers sign-in, session status, public search, authenticated
page access, token re-entry, and logout.

## Layout

| Path | Purpose |
| --- | --- |
| `crates/bloppen-api/` | HTTP client, session persistence, reauth, search, pages |
| `crates/bloppen-cli/` | `bloppen` command-line client |
| `crates/bloppen-mcp/` | Read-only MCP server |
| `crates/bloppen-probe/` | Phase 4 live validation probe |
| `re/` | PRD, analysis, extraction and decompilation scripts |
| `nix/` | Dendritic flake-parts modules |
| `backlog/` | Backlog.md task records |

## Security

Never commit the APK, extracted splits, decompiled output, network captures, or
credentials. Live account credentials live in the configured secretspec
provider, and session captures belong under `secrets/` (gitignored). The
library never prints passwords, cookie values, or token values.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
