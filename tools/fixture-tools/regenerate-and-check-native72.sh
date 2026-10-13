#!/usr/bin/env bash
set -euo pipefail
# Requires the locked project toolchain; existing nonempty evidence is never reused.
if [[ "$#" -ne 2 ]]; then
  printf '%s\n' 'Usage: regenerate-and-check-native72.sh source empty-output' >&2
  exit 2
fi
script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# One entrypoint validates paths/output and runs the existing storage guard first.
exec python3 "$script_directory/regenerate_native72_inputs.py" --build-and-check -- "$@"
