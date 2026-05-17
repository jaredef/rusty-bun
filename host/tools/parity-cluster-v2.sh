#!/usr/bin/env bash
# Cluster-targeted parity sweep. Extract one residual cluster from the
# most recent canonical reading and run host-v2 against just that basket.
# Cheap iteration loop for substrate moves that target a single cluster:
# measure per-cluster yield without paying the full 25-min canonical
# sweep cost.
#
# Usage:
#   parity-cluster-v2.sh <cluster-name> [results.json] [out.json]
#
# Defaults:
#   results.json = host/tools/parity-results-top500-postp49.json (newest canonical)
#   out.json     = host/tools/parity-results-cluster-<cluster-name>.json
#
# Cluster names: see select-cluster.py --help. Typical:
#   kc-pm-1-2  kc-pm-3-10  kc-gt-10  typeof-diff
#   dyn-import compile-error  both-err  bun-err-rb-ok  rb-err-other
#
# After landing a substrate move, run this against the cluster you targeted
# to confirm yield before paying for a full sweep.

set -uo pipefail

if [ $# -lt 1 ]; then
  echo "usage: $0 <cluster-name> [results.json] [out.json]" >&2
  exit 2
fi

CLUSTER="$1"
TOOLS="$(cd "$(dirname "$0")" && pwd)"
RESULTS="${2:-$TOOLS/parity-results-top500-postp49.json}"
OUT="${3:-$TOOLS/parity-results-cluster-${CLUSTER}.json}"

if [ ! -f "$RESULTS" ]; then
  echo "results file not found: $RESULTS" >&2
  exit 1
fi

BASKET="$(mktemp -t cluster-basket-XXXXXX.txt)"
trap 'rm -f "$BASKET"' EXIT

python3 "$TOOLS/select-cluster.py" "$RESULTS" "$CLUSTER" > "$BASKET" || exit 1

N=$(wc -l < "$BASKET")
echo "Cluster '$CLUSTER': $N packages from $(basename "$RESULTS")" >&2
echo "Basket: $BASKET" >&2
echo "Out:    $OUT" >&2

exec "$TOOLS/parity-measure-v2.sh" "$BASKET" "$OUT"
