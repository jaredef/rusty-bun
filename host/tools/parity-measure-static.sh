#!/usr/bin/env bash
# Doc 715 §VII shift 2 + Doc 716 operational metric + Tier-Ω.4.f.
# Variant of parity-measure.sh that codegens per-package STATIC import
# probes. Required for host-v2 because rusty-js-runtime does not yet
# implement dynamic import with bare specifier (Ω.5.CCCCCCC stub).
#
# For each package in the list:
#   1. bun install into an isolated sandbox dir (reused with parity-measure.sh)
#   2. Codegen probe-static.mjs with `import * as M from "<pkg>"` literal
#   3. Run under Bun, capture stdout + exit code
#   4. Run under $RB, capture stdout + exit code
#   5. Compare byte-for-byte (matching parity-measure.sh semantics)
#
# Empty stdout + nonzero exit → synthetic ERR JSON ("LoadFailed").
# This keeps host-v2's substrate-honest "throw at module-eval" path
# legible to downstream jq queries.
#
# Override target host via RB_BIN= (default $ROOT/target/release/rusty-bun-host-v2).
# Override package list via positional arg (default parity-top100.txt).
# Override output via second positional arg (default parity-results-static.json).
#
# Usage: ./parity-measure-static.sh [list.txt] [out.json]
set -uo pipefail
TOOLS="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$TOOLS/../.." && pwd)"
LIST="${1:-$TOOLS/parity-top100.txt}"
OUT="${2:-$TOOLS/parity-results-static.json}"
SANDBOX="${PARITY_SANDBOX:-/tmp/parity-sandbox}"
RB="${RB_BIN:-$ROOT/target/release/rusty-bun-host-v2}"

mkdir -p "$SANDBOX"

if [ ! -x "$RB" ]; then
  echo "Binary not found: $RB"
  echo "Build first: cargo build --release --bin $(basename "$RB")"
  exit 1
fi

PKGS=$(grep -vE '^\s*(#|$)' "$LIST" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')

# Emit a synthetic FAIL JSON for the empty-stdout case. Matches the
# shape of an OK probe well enough for jq to distinguish status="ERR".
synth_err() {
  local pkg="$1" reason="$2"
  jq -cn --arg pkg "$pkg" --arg reason "$reason" \
    '{status:"ERR",pkg:$pkg,error:"LoadFailed",message:$reason}'
}

# Quote a string for safe JS string-literal embedding. The package
# list has no scoped packages today, but we still need to defend
# against any future entries containing backslashes or quotes.
js_quote() {
  printf '%s' "$1" | jq -Rs .
}

echo "[" > "$OUT"
first=1
n_pass=0
n_fail=0
n_skip=0
total=0
mismatch_log=()

for pkg in $PKGS; do
  total=$((total + 1))
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
    status="SKIP_INSTALL_FAILED"
    n_skip=$((n_skip + 1))
    if [ $first -eq 0 ]; then echo "," >> "$OUT"; fi
    pkg_json=$(js_quote "$pkg")
    status_json=$(js_quote "$status")
    echo "  {\"pkg\":$pkg_json,\"status\":$status_json,\"bun\":\"\",\"rb\":\"\"}" >> "$OUT"
    first=0
    continue
  fi

  # Codegen static-import probe. The package name is embedded as a
  # JS string literal (jq -Rs handles escaping). On import failure
  # the runtime throws at module-eval and never reaches the write,
  # so stdout is empty + exit is nonzero — handled below.
  qpkg=$(js_quote "$pkg")
  probe="$d/parity-probe-static.mjs"
  cat > "$probe" <<EOF
import * as M from $qpkg;
const keys = Object.keys(M).sort();
const shape = {};
for (const k of keys) { shape[k] = typeof M[k]; }
process.stdout.write(JSON.stringify({status:"OK",pkg:$qpkg,keyCount:keys.length,shape}) + "\n");
EOF

  bun_out=$(cd "$d" && bun parity-probe-static.mjs 2>/dev/null)
  bun_rc=$?
  rb_out=$(cd "$d" && "$RB" parity-probe-static.mjs 2>/dev/null)
  rb_rc=$?

  # Empty stdout + nonzero exit → synthetic LoadFailed JSON. This
  # surfaces the substrate-honest failure path without conflating
  # it with successful "Object.keys is empty" loads.
  if [ -z "$bun_out" ] && [ $bun_rc -ne 0 ]; then
    bun_out=$(synth_err "$pkg" "static import threw at eval (exit $bun_rc)")
  fi
  if [ -z "$rb_out" ] && [ $rb_rc -ne 0 ]; then
    rb_out=$(synth_err "$pkg" "static import threw at eval (exit $rb_rc)")
  fi

  if [ "$bun_out" = "$rb_out" ] && [ -n "$bun_out" ]; then
    # Treat byte-equal ERR-vs-ERR as still a FAIL semantically, since
    # parity-measure.sh's prior semantics required PASS to be a
    # successful load. The "OK" prefix gate matches that convention.
    case "$bun_out" in
      *'"status":"OK"'*) status="PASS"; n_pass=$((n_pass + 1)) ;;
      *) status="FAIL"; n_fail=$((n_fail + 1)); mismatch_log+=("$pkg: both-ERR ${bun_out:0:80}") ;;
    esac
  else
    status="FAIL"
    n_fail=$((n_fail + 1))
    mismatch_log+=("$pkg: bun=${bun_out:0:80} rb=${rb_out:0:80}")
  fi

  if [ $first -eq 0 ]; then echo "," >> "$OUT"; fi
  bun_json=$(printf '%s' "$bun_out" | jq -Rs .)
  rb_json=$(printf '%s' "$rb_out" | jq -Rs .)
  pkg_json=$(js_quote "$pkg")
  status_json=$(js_quote "$status")
  echo "  {\"pkg\":$pkg_json,\"status\":$status_json,\"bun\":$bun_json,\"rb\":$rb_json}" >> "$OUT"
  first=0
done

echo "]" >> "$OUT"

echo
echo "==============================================================="
echo "Parity measurement summary (static-probe variant)"
echo "Host: $RB"
echo "==============================================================="
echo "Total:    $total"
echo "Pass:     $n_pass"
echo "Fail:     $n_fail"
echo "Skip:     $n_skip"
if [ $total -gt 0 ]; then
  pct=$(echo "scale=1; $n_pass * 100 / $total" | bc)
  echo "Parity:   $pct% ($n_pass / $total)"
fi
echo "Results JSON: $OUT"
if [ ${#mismatch_log[@]} -gt 0 ]; then
  echo
  echo "Mismatches (first 10):"
  for ((i=0; i<${#mismatch_log[@]} && i<10; i++)); do
    echo "  ${mismatch_log[$i]}"
  done
fi
