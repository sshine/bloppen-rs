# Default recipe: list available commands
default:
    @just --list

# Format Nix, Rust, Markdown, TOML, YAML, and shell
fmt:
    treefmt

# Check formatting without writing
fmt-check:
    treefmt --fail-on-change --no-cache

# Run the workspace tests
test:
    cargo test --workspace --all-features

# Run clippy over the workspace
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build the workspace
build:
    cargo build --workspace --all-features

# Generate documentation
doc *args='':
    cargo doc --no-deps --workspace --all-features {{args}}

# Verify Cargo.lock is in step with the manifests
lock-check:
    cargo metadata --locked --format-version 1 > /dev/null

# Verify every member packages cleanly
package-check:
    cargo package --locked --workspace --allow-dirty

# Run CI checks locally
ci: fmt-check lint test build doc lock-check package-check
    @echo "All CI checks passed!"

secretspec_reason := "bloppen live API validation"

# Prompt for the live credentials and store them in the configured provider
[doc("Store BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD in the keyring provider")]
secrets-set:
    secretspec set BORNELOPPEN_EMAIL --reason {{quote(secretspec_reason)}}
    secretspec set BORNELOPPEN_PASSWORD --reason {{quote(secretspec_reason)}}

# Report which live credentials are missing from the configured provider
secrets-check:
    secretspec check --no-prompt --reason {{quote(secretspec_reason)}}

# Run the live probe with credentials injected by secretspec
probe *args='':
    secretspec run --reason {{quote(secretspec_reason)}} -- \
        cargo run --quiet -p bloppen-probe -- {{args}}

# Run the CLI with credentials injected by secretspec
cli *args='':
    secretspec run --reason {{quote(secretspec_reason)}} -- \
        cargo run --quiet -p bloppen-cli -- {{args}}

# Run the read-only CLI showcase against the live site
showcase:
    secretspec run --reason {{quote(secretspec_reason)}} -- ./scripts/showcase.sh

# Run the MCP server over stdio with credentials from secretspec
mcp *args='':
    secretspec run --reason {{quote(secretspec_reason)}} -- \
        cargo run --quiet -p bloppen-mcp -- {{args}}

# Print the MCP tool schemas, which needs no credentials
mcp-tools *args='':
    cargo run --quiet -p bloppen-mcp -- --list-tools {{args}}

# Run the live API tests with credentials injected by secretspec
[doc("Run the live API tests against the real site, one at a time")]
live-test *args='':
    secretspec run --reason {{quote(secretspec_reason)}} -- \
        cargo test -p bloppen-api --test live -- --ignored --nocapture --test-threads 1 {{args}}

# Print package, version, and SDK levels from the extracted base APK
apk-info:
    @aapt2 dump badging apk/extracted/base.apk | grep -E "^(package|minSdkVersion|targetSdkVersion|application-label)"

# Run the pinned Backlog.md CLI (see bin/backlog for why this is wrapped)
backlog *args:
    ./bin/backlog {{args}}
