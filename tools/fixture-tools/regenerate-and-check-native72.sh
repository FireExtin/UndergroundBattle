#!/usr/bin/env bash
set -euo pipefail
# Usage: bash regenerate-and-check-native72.sh /absolute/candidate/source /absolute/output
# Requires locked Rust dependencies, Rust 1.90, wasm32-unknown-unknown,
# wasm-bindgen 0.2.104, Python 3, Node, and the candidate's npm dependencies.
JZ35_SOURCE_ROOT=$(realpath "$1")
mkdir -p "$2"
JZ35_REVIEW_ROOT=$(realpath "$2")
export JZ35_SOURCE_ROOT JZ35_REVIEW_ROOT
export CARGO_BUILD_JOBS=1
cd "$JZ35_SOURCE_ROOT"
# Avoid stale shared-cache binaries when reviewing source extracted from Git archives.
find rust-game/src rust-game-wasm/src -type f -exec touch {} +
cargo build --locked --release --target wasm32-unknown-unknown -p hegemony-wasm
wasm-bindgen "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/hegemony_wasm.wasm" --target web --out-dir rust-game-wasm/pkg
python3 "$(dirname "$(realpath "$0")")/regenerate_native72_inputs.py"
node rust-game-wasm/tests/jz35_compare.mjs "$JZ35_REVIEW_ROOT/evidence"
for card in jc061 jc087 jc094 wm056 wm023 xq25 wm038; do
  node "rust-game-wasm/tests/${card}_compare.mjs" "$JZ35_REVIEW_ROOT/evidence/ui-refresh/$card"
done
cd web
npm test -- src/game/JZ35Integration.test.jsx src/game/JZ35Face.test.jsx src/game/CurrentLifecycleChoices.test.jsx src/game/SevenCardIntegration.test.jsx --maxWorkers=1
npm test -- --maxWorkers=1
npm run build
