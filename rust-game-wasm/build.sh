#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"
wasm_bindgen_bin="${WASM_BINDGEN:-wasm-bindgen}"
build_target_dir="${CARGO_TARGET_DIR:-target}"
cargo build --locked --release -p hegemony-wasm --target wasm32-unknown-unknown
"$wasm_bindgen_bin" "$build_target_dir/wasm32-unknown-unknown/release/hegemony_wasm.wasm" --target web --out-dir rust-game-wasm/pkg --out-name hegemony_wasm
printf '%s\n' '{"type":"module","private":true}' > rust-game-wasm/pkg/package.json
