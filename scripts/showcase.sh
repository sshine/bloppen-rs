#!/usr/bin/env bash
# Run the read-only CLI commands against the live site.
#
# `just showcase` wraps this in `secretspec run`, so BORNELOPPEN_EMAIL and
# BORNELOPPEN_PASSWORD are present without touching the environment directly.
set -uo pipefail

run() {
  local label=$1
  shift
  echo "=== $label ==="
  "$@" 2>&1 || echo "(failed)"
  echo
}

cli=(cargo run --quiet -p bloppen-cli --)

run "auth login" "${cli[@]}" auth login
run "auth status" "${cli[@]}" auth status
run "search lego" "${cli[@]}" search lego --limit 3
run "home page" "${cli[@]}" page /mit-boerneloppen/forside
run "login token" "${cli[@]}" auth token
