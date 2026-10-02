#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"
fixture_file="$(mktemp /tmp/hegemony-wasm-fixture.XXXXXX.json)"
trap 'rm -f "$fixture_file"' EXIT
bash rust-game-wasm/build.sh
cargo run --locked -p hegemony-wasm --example native_fixtures -- "$fixture_file"
node rust-game-wasm/tests/compare.mjs "$fixture_file"
