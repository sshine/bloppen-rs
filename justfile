# Default recipe: list available commands
default:
    @just --list

# Format Nix, Rust, Markdown, TOML, YAML, and shell
fmt:
    treefmt

# Check formatting without writing
fmt-check:
    treefmt --fail-on-change --no-cache

# Print package, version, and SDK levels from the extracted base APK
apk-info:
    @aapt2 dump badging apk/extracted/base.apk | grep -E "^(package|minSdkVersion|targetSdkVersion|application-label)"

# Run the pinned Backlog.md CLI (see bin/backlog for why this is wrapped)
backlog *args:
    ./bin/backlog {{args}}
