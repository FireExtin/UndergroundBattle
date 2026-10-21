#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"
if [[ "${UNDERGROUNDBATTLE_BUILD_STORAGE_READY:-}" != "1" ]]; then
  exec node "$project_root/tools/build-storage.mjs" --output-dir "$project_root/rust-game-wasm/pkg" -- bash "$project_root/rust-game-wasm/verify.sh" "$@"
fi
fixture_file="$(mktemp "$TMPDIR/hegemony-wasm-fixture.XXXXXX.json")"
xq24_directory="$(mktemp -d "$TMPDIR/hegemony-xq24.XXXXXX")"
trap 'rm -f "$fixture_file"; rm -rf "$fixture_file.cases" "$xq24_directory"' EXIT
bash rust-game-wasm/build.sh
cargo run --locked -p hegemony-wasm --example native_fixtures -- "$fixture_file"
node rust-game-wasm/tests/compare.mjs "$fixture_file"
XQ24_CONTAINER_EVIDENCE_FILE="$xq24_directory/invalid.json" \
  XQ24_TRACE_DIR="$xq24_directory/room" \
  cargo test --locked -p hegemony-server --lib -- \
    xq24_entry_frame_container_restore_rejects_response_bypass \
    xq24_continuous_duel_teams_room_pause_restore_journal_and_receipt_inputs
node tools/cards/check-xq24-containers.mjs rust-game-wasm/pkg/hegemony_wasm.js "" "$xq24_directory"
