#!/usr/bin/env bash
# tag-validate.sh — validate a Tag-on-DAG string against the manifest.
#
# Form (per host/tools/tag-grammar.md §1, Doc 728):
#   Ω.5.<pipeline>.<layer>.<handle>[.<seq>]
#
# - Leading "Ω.5." is accepted as the unicode form, the literal four-char
#   ASCII fallback, or omitted entirely (unprefixed coordinate).
# - <pipeline> ∈ manifest.pipelines[].id (P01..P16)
# - <layer> ∈ manifest.above_engine_layers[].id ∪ manifest.engine_layers[].id
# - <handle> matches [a-z][a-z0-9-]* (kebab-case, starts with letter)
# - <seq> optional, numeric; presence is a smell per grammar §2.
#
# Exit 0 on valid; echoes "OK pipeline=<P> layer=<L> handle=<H>" to stdout.
# Exit 1 on invalid; echoes "INVALID: <reason>" to stderr.
#
# Companion: host/tools/dag-coordinates.json (manifest_version 1).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST_PATH="${MANIFEST_PATH:-$ROOT/host/tools/dag-coordinates.json}"

die() { echo "INVALID: $1" >&2; exit 1; }

validate_one() {
    local tag="$1"
    [[ -n "$tag" ]] || die "empty tag"

    # Strip leading Ω.5. (unicode) or literal Ω.5. — both render identically
    # in this source byte-for-byte. Bash sees the bytes; we strip the prefix
    # if present, accept unprefixed otherwise.
    local body="$tag"
    if [[ "$body" == "Ω.5."* ]]; then
        body="${body#Ω.5.}"
    fi

    # Split on dots: pipeline.layer.handle[.seq]
    IFS='.' read -r -a parts <<< "$body"
    local n="${#parts[@]}"
    if (( n < 3 || n > 4 )); then
        die "expected pipeline.layer.handle[.seq], got $n segment(s) in '$body'"
    fi

    local pipeline="${parts[0]}"
    local layer="${parts[1]}"
    local handle="${parts[2]}"
    local seq="${parts[3]:-}"

    # Validate pipeline against manifest
    if ! jq -e --arg p "$pipeline" '.pipelines | map(.id) | index($p)' \
            "$MANIFEST_PATH" >/dev/null; then
        die "pipeline id '$pipeline' not in manifest"
    fi

    # Validate layer against union of above_engine_layers + engine_layers
    if ! jq -e --arg l "$layer" \
            '((.above_engine_layers + .engine_layers) | map(.id) | index($l))' \
            "$MANIFEST_PATH" >/dev/null; then
        die "layer id '$layer' not in manifest"
    fi

    # Validate handle kebab-case
    if ! [[ "$handle" =~ ^[a-z][a-z0-9-]*$ ]]; then
        die "handle '$handle' must match [a-z][a-z0-9-]*"
    fi

    # Validate optional seq
    if [[ -n "$seq" ]]; then
        if ! [[ "$seq" =~ ^[0-9]+$ ]]; then
            die "seq '$seq' must be numeric"
        fi
        echo "WARN: seq disambiguator is a smell per grammar §2 — re-evaluate handle granularity" >&2
    fi

    echo "OK pipeline=$pipeline layer=$layer handle=$handle"
}

self_test() {
    local fail=0
    local good=(
        "Ω.5.P03.L3.callee-shape-probe"
        "Ω.5.P04.L5.bigint-arith"
        "Ω.5.P05.L1.dotjson-bare"
        "Ω.5.P02.L0.fn-expr-early-return"
        "Ω.5.P08.L3.typedarray-of-static"
        "P06.E3.module-ns-default"
    )
    local bad=(
        "Ω.5.P99.L5.bogus-pipeline"
        "Ω.5.P03.L9.bogus-layer"
        "Ω.5.P03.L3.BadHandle"
    )
    local t
    for t in "${good[@]}"; do
        if ! "$0" "$t" >/dev/null 2>&1; then
            echo "self-test: expected VALID, got INVALID for: $t" >&2
            fail=1
        fi
    done
    for t in "${bad[@]}"; do
        if "$0" "$t" >/dev/null 2>&1; then
            echo "self-test: expected INVALID, got VALID for: $t" >&2
            fail=1
        fi
    done
    if (( fail == 0 )); then
        echo "self-test: PASS (${#good[@]} good, ${#bad[@]} bad)"
    else
        echo "self-test: FAIL" >&2
        exit 1
    fi
}

if [[ "${1:-}" == "--self-test" ]]; then
    self_test
    exit 0
fi

if [[ $# -ne 1 ]]; then
    echo "usage: $0 <tag-string> | --self-test" >&2
    exit 2
fi

validate_one "$1"
