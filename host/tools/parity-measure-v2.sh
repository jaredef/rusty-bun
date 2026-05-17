#!/usr/bin/env bash
# Parity sweep targeting host-v2's rusty-bun-host-v2 (the rusty-js engine
# substrate work — Ω.5.P* tags). Sister script to parity-measure.sh, which
# targets the rquickjs-based host/ binary. Both use the same probe + sandbox.
#
# Usage: ./parity-measure-v2.sh [list.txt] [out.json]
# Defaults: list=parity-top500.txt, out=parity-results-v2.json
set -uo pipefail
TOOLS="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$TOOLS/../.." && pwd)"
LIST="${1:-$TOOLS/parity-top500.txt}"
OUT="${2:-$TOOLS/parity-results-v2.json}"
exec env RB_BIN="$ROOT/target/release/rusty-bun-host-v2" \
  "$TOOLS/parity-measure.sh" "$LIST" "$OUT"
