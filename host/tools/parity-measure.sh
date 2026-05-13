#!/usr/bin/env bash
# Doc 715 §VII shift 2 + Doc 716 operational metric.
# Measures rusty-bun-host's parity against Bun at the load-and-shape
# layer across a curated list of npm packages.
#
# For each package in the list:
#   1. bun install into an isolated tempdir
#   2. Run parity-probe.mjs under Bun → record output
#   3. Run parity-probe.mjs under rusty-bun-host via cargo test path → record output
#   4. Compare byte-for-byte
#
# Output: per-package status + aggregate parity-percentage.
#
# Usage: ./parity-measure.sh [list.txt] [out.json]
# Defaults: list=parity-top100.txt, out=parity-results.json

set -uo pipefail
TOOLS="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$TOOLS/../.." && pwd)"
LIST="${1:-$TOOLS/parity-top100.txt}"
OUT="${2:-$TOOLS/parity-results.json}"

# Per-package isolated install root. Reusable across runs to skip
# already-installed packages — bun add is idempotent.
SANDBOX="${PARITY_SANDBOX:-/tmp/parity-sandbox}"
mkdir -p "$SANDBOX"

# rusty-bun-host binary
RB="$ROOT/target/release/rusty-bun-host"
if [ ! -x "$RB" ]; then
  echo "Build rusty-bun-host first: cargo build --release --bin rusty-bun-host"
  exit 1
fi

# Read package list (skip comments + blank lines).
PKGS=$(grep -vE '^\s*(#|$)' "$LIST" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')

# Header
echo "[" > "$OUT"
first=1

n_pass=0
n_fail=0
n_skip=0
total=0
mismatch_log=()

for pkg in $PKGS; do
  total=$((total + 1))
  # Per-package sandbox dir (replace / with -- for scoped pkgs).
  safe="${pkg//\//--}"
  d="$SANDBOX/$safe"
  if [ ! -d "$d/node_modules/$pkg" ]; then
    mkdir -p "$d"
    (
      cd "$d"
      [ -f package.json ] || echo '{"name":"sb","version":"0.0.0"}' > package.json
      bun add "$pkg" > /dev/null 2>&1
    )
  fi

  if [ ! -d "$d/node_modules/$pkg" ]; then
    # Install failed (e.g., 404).
    status="SKIP_INSTALL_FAILED"
    n_skip=$((n_skip + 1))
    if [ $first -eq 0 ]; then echo "," >> "$OUT"; fi
    printf '  {"pkg":%q,"status":%q}\n' "$pkg" "$status" >> "$OUT"
    first=0
    continue
  fi

  # Copy the probe into the sandbox so the resolver can find the
  # package via the sandbox's node_modules. The probe references the
  # package by bare specifier; resolution walks up from the probe's
  # directory.
  cp "$TOOLS/parity-probe.mjs" "$d/parity-probe.mjs"
  bun_out=$(cd "$d" && PARITY_PROBE_PKG="$pkg" bun parity-probe.mjs 2>/dev/null)
  rb_out=$(cd "$d" && PARITY_PROBE_PKG="$pkg" "$RB" parity-probe.mjs 2>/dev/null)

  # Compare
  if [ "$bun_out" = "$rb_out" ] && [ -n "$bun_out" ]; then
    status="PASS"
    n_pass=$((n_pass + 1))
  else
    status="FAIL"
    n_fail=$((n_fail + 1))
    mismatch_log+=("$pkg: bun=${bun_out:0:80} rb=${rb_out:0:80}")
  fi

  if [ $first -eq 0 ]; then echo "," >> "$OUT"; fi
  bun_json=$(printf '%s' "$bun_out" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')
  rb_json=$(printf '%s' "$rb_out" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')
  pkg_json=$(printf '%s' "$pkg" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')
  status_json=$(printf '%s' "$status" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')
  echo "  {\"pkg\":$pkg_json,\"status\":$status_json,\"bun\":$bun_json,\"rb\":$rb_json}" >> "$OUT"
  first=0
done

echo "]" >> "$OUT"

# Summary
echo
echo "═══════════════════════════════════════════════════════════════"
echo "Parity measurement summary"
echo "═══════════════════════════════════════════════════════════════"
echo "Total:    $total"
echo "Pass:     $n_pass"
echo "Fail:     $n_fail"
echo "Skip:     $n_skip"
echo
if [ $total -gt 0 ]; then
  pct=$(echo "scale=1; $n_pass * 100 / $total" | bc)
  echo "Parity:   $pct% ($n_pass / $total)"
fi
echo
echo "Results JSON: $OUT"

if [ ${#mismatch_log[@]} -gt 0 ]; then
  echo
  echo "Mismatches (first 10):"
  for ((i=0; i<${#mismatch_log[@]} && i<10; i++)); do
    echo "  ${mismatch_log[$i]}"
  done
fi
