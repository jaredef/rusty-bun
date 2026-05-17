#!/usr/bin/env bash
# Fast parity sweep against the exemplar basket (host/tools/parity-
# exemplars.txt). Targets host-v2 via parity-measure-v2.sh. Use for
# fast iteration during substrate work; rerun host/tools/select-
# exemplars.py against the latest full sweep to refresh the basket
# when the residual distribution shifts.
#
# Usage: ./parity-fast-v2.sh [out.json]
# Default: out=host/tools/parity-results-exemplars.json
set -uo pipefail
TOOLS="$(cd "$(dirname "$0")" && pwd)"
OUT="${1:-$TOOLS/parity-results-exemplars.json}"
exec "$TOOLS/parity-measure-v2.sh" "$TOOLS/parity-exemplars.txt" "$OUT"
