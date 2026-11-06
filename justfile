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

# Print package, version, and SDK levels from the extracted base APK
apk-info:
    @aapt2 dump badging apk/extracted/base.apk | grep -E "^(package|minSdkVersion|targetSdkVersion|application-label)"

# Run the pinned Backlog.md CLI (see bin/backlog for why this is wrapped)
backlog *args:
    ./bin/backlog {{args}}
