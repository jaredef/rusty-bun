#!/usr/bin/env bash
# Stub detector per Doc 716. Scans the host + pilot codebase for
# syntactic signatures of the three stub kinds defined in §II:
#   K1 throw-on-use
#   K2 no-op return
#   K3 hardcoded-sentinel
#
# Outputs candidates grouped by kind, with file:line:context for each.
# Operators triage candidates into the canonical catalogue (host/tools/
# stub-catalog.md) by adding a marker comment `// STUB:K<n> <node>`.
# Subsequent runs honor markers + only surface unmarked candidates as
# untriaged.
#
# Cross-references with substrate-rank.sh: each catalogued stub's
# substrate-node name should match a row in the ranker output. The
# (in-degree × consumer-exercise-count) product is the stub priority.

set -uo pipefail
ROOT="${1:-/home/jaredef/rusty-bun}"

# Sources to scan: host code + pilot crates. Skip target/ + node_modules/ + fixtures.
SOURCES=$(find "$ROOT" \
  \( -name "target" -o -name "node_modules" -o -name "fixtures" -o -name ".git" \) -prune \
  -o -type f \( -name "*.rs" -o -name "lib.rs" \) -print 2>/dev/null)

# K1 throw-on-use: explicit throw of an Error with "not implemented" /
# "not supported" / "not yet implemented" / "stub" / "NYI" markers.
# Also matches JS-side `throw new Error("...not implemented...")` inside
# the embedded JS strings in lib.rs.
echo "═══════════════════════════════════════════════════════════════"
echo "K1 — throw-on-use stubs"
echo "═══════════════════════════════════════════════════════════════"
echo
grep -nHE 'throw\s+new\s+Error\(["'"'"'`][^"'"'"'`]*(not implemented|not supported|not yet|NYI|stub)' \
  $SOURCES 2>/dev/null |
  awk -F: '{ printf "  %s:%s\n    %s\n", $1, $2, substr($0, index($0, $3)) }' |
  sed "s|$ROOT/||g" |
  head -80

# K2 no-op return: function bodies that are literally `(){}` arrow-form
# or `function() {}` declaration-form, returning undefined implicitly.
# Includes `cb()` / `() => {}` callbacks. Filters to those near a
# function definition or property assignment.
echo
echo "═══════════════════════════════════════════════════════════════"
echo "K2 — no-op return stubs"
echo "═══════════════════════════════════════════════════════════════"
echo
# Scope K2 to lib.rs only — Rust match-arms produce too many false
# positives elsewhere. Within lib.rs, restrict to JS-context lines
# (property assignment with empty body or arrow-returns-sentinel).
grep -nE '^[[:space:]]+(globalThis\..*=|[a-zA-Z_]+:)\s*function\s*\([^)]*\)\s*\{\s*\}|^[[:space:]]+[a-zA-Z_]+\s*\([^)]*\)\s*\{\s*\}\s*,?$|^[[:space:]]+[a-zA-Z_]+:\s*\([^)]*\)\s*=>\s*\{\s*\},?$|^[[:space:]]+[a-zA-Z_]+:\s*\([^)]*\)\s*=>\s*(null|false|undefined)\s*,?$' \
  "$ROOT/host/src/lib.rs" 2>/dev/null |
  awk -F: '{ ln=$1; $1=""; printf "  host/src/lib.rs:%s\n    %s\n", ln, substr($0, 2) }' |
  head -60

# K3 hardcoded-sentinel: returns a fixed-non-trivial value that's
# plausibly correct for the common-case consumer. Harder to detect
# syntactically without context — surface candidates via `return "..."`
# of multi-char strings near apparently-stubbed methods.
echo
echo "═══════════════════════════════════════════════════════════════"
echo "K3 — hardcoded-sentinel stubs (heuristic; high false-positive)"
echo "═══════════════════════════════════════════════════════════════"
echo
grep -nHE 'return\s+"[a-zA-Z]+";?$|=>\s*"[a-zA-Z][a-zA-Z]+"\s*[,;]?$' \
  $SOURCES 2>/dev/null |
  awk -F: '{ printf "  %s:%s\n    %s\n", $1, $2, substr($0, index($0, $3)) }' |
  sed "s|$ROOT/||g" |
  head -40

# Explicit marker comments (operator-curated catalogue entries).
echo
echo "═══════════════════════════════════════════════════════════════"
echo "Curated markers — //STUB:K[1-3] <node>"
echo "═══════════════════════════════════════════════════════════════"
echo
grep -nHE '//\s*STUB:K[1-3]' $SOURCES 2>/dev/null |
  awk -F: '{ printf "  %s:%s\n    %s\n", $1, $2, substr($0, index($0, $3)) }' |
  sed "s|$ROOT/||g"

# Summary counts (does not double-count files containing multiple stubs).
echo
echo "═══════════════════════════════════════════════════════════════"
echo "Summary"
echo "═══════════════════════════════════════════════════════════════"
k1=$(grep -hE 'throw\s+new\s+Error\(["'"'"'`][^"'"'"'`]*(not implemented|not supported|not yet|NYI|stub)' $SOURCES 2>/dev/null | wc -l)
k2=$(grep -hE '(=\s*function\s*\([^)]*\)\s*\{\s*\}|:\s*function\s*\([^)]*\)\s*\{\s*\}|=>\s*\{\s*\})' $SOURCES 2>/dev/null | grep -v '// ' | wc -l)
k3=$(grep -hE 'return\s+"[a-zA-Z]+";?$' $SOURCES 2>/dev/null | wc -l)
markers=$(grep -hE '//\s*STUB:K[1-3]' $SOURCES 2>/dev/null | wc -l)
echo "  K1 throw-on-use candidates: $k1"
echo "  K2 no-op-return candidates: $k2"
echo "  K3 hardcoded-sentinel candidates (heuristic): $k3"
echo "  Curated //STUB:K markers:     $markers"
