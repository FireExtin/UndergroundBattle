#!/usr/bin/env bash
# Minimal native cloud startup; requires the pinned Rust toolchain and Node/npm.
set -euo pipefail
project_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
cd -- "$project_dir"
cargo build --locked --target-dir "$project_dir/target" -p hegemony-server --bin hegemony-server --bin hegemony-audit
npm --prefix web ci --ignore-scripts --no-audit --no-fund
npm --prefix web run build
export WEB_DIST="$project_dir/web/dist"
export HEGEMONY_DB="${HEGEMONY_DB:-$project_dir/rust-game-v2.1.sqlite3}"
export PORT="${PORT:-8090}"
exec "$project_dir/target/debug/hegemony-server"
