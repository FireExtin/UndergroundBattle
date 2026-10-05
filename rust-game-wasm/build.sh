#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"
wasm_bindgen_bin="${WASM_BINDGEN:-wasm-bindgen}"
build_target_dir="${CARGO_TARGET_DIR:-target}"
build_args=()
output_dir="rust-game-wasm/pkg"
case "${1:-}" in
  "") ;;
  --society-fixtures) build_args=(--features society-fixtures); output_dir="rust-game-wasm/pkg-society-fixtures" ;;
  *) printf '%s\n' 'Usage: build.sh [--society-fixtures]' >&2; exit 2 ;;
esac
# Bash 3.2 on macOS treats an empty array as unset under nounset.
cargo build --locked --release -p hegemony-wasm --target wasm32-unknown-unknown ${build_args[@]+"${build_args[@]}"}
"$wasm_bindgen_bin" "$build_target_dir/wasm32-unknown-unknown/release/hegemony_wasm.wasm" --target web --out-dir "$output_dir" --out-name hegemony_wasm
printf '%s\n' '{"type":"module","private":true}' > "$output_dir/package.json"
