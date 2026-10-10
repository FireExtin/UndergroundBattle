#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"
if [[ "${UNDERGROUNDBATTLE_BUILD_STORAGE_READY:-}" != "1" ]]; then
  exec node "$project_root/tools/build-storage.mjs" --output-dir "$project_root/rust-game-wasm/pkg" -- bash "$project_root/rust-game-wasm/verify.sh" "$@"
fi
fixture_file="$(mktemp "$TMPDIR/hegemony-wasm-fixture.XXXXXX.json")"
trap 'rm -f "$fixture_file"; rm -rf "$fixture_file.cases"' EXIT
bash rust-game-wasm/build.sh
cargo run --locked -p hegemony-wasm --example native_fixtures -- "$fixture_file"
node rust-game-wasm/tests/compare.mjs "$fixture_file"
