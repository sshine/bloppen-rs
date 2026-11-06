# bloppen-rs

Børneloppen.dk API + CLI + MCP.

Reverse-engineering research and a Rust client for the Danish **Børneloppen**
Android app and website. This is an unofficial project and is not affiliated
with Børneloppen.

## Status

- **Phase 1–3 complete:** APK acquired, decompiled, and analysed. The app is a
  native Android shell around the `www.boerneloppen.dk` website, with two
  cookie-authenticated JSON endpoints.
- **Phase 4 in progress:** the `bloppen-probe` crate exercises the real login,
  cookie, token, and page flows with credentials from the keyring.
- **Phase 5–6 planned:** `bloppen-api` library, `bloppen-cli`, and an MCP
  server.

The detailed findings are in [`re/analysis/`](re/analysis/) and the project
requirements are in [`re/prd.md`](re/prd.md). Agent and contributor working
rules are in [`AGENTS.md`](AGENTS.md).

## Build

```sh
nix develop
just build
just test
```

The live probe needs credentials in the configured secretspec provider
(keyring by default):

```sh
just secrets-set
just secrets-check
just probe --token-login
```

## Layout

| Path | Purpose |
| --- | --- |
| `crates/bloppen-probe/` | Phase 4 live validation probe |
| `re/` | PRD, analysis, extraction and decompilation scripts |
| `nix/` | Dendritic flake-parts modules |
| `backlog/` | Backlog.md task records |

## Security

Never commit the APK, extracted splits, decompiled output, network captures, or
credentials. Live account credentials are stored in the configured secretspec
provider and injected only into the probe. Captured responses with personal
data belong under `secrets/`, which is gitignored.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
