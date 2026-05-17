#!/usr/bin/env bash
# tag-append.sh — advisory append-only log for Tag-on-DAG entries.
#
# Usage: tag-append.sh <tag> <commit-hash> <recognition-string>
#
# Calls tag-validate.sh first; refuses to append if invalid. Reads the commit
# timestamp from git (ISO 8601). Appends one JSONL line to
# host/tools/tag-log.jsonl with (tag, commit, timestamp, pipeline, layer,
# handle, recognition).
#
# Structural collision detection at the log tier: if (pipeline, layer, handle)
# already exists for a different commit, refuses to append. Idempotent on
# identical (tag, commit) pair.
#
# Companion: host/tools/tag-grammar.md, host/tools/dag-coordinates.json,
# Doc 728.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TOOLS="$ROOT/host/tools"
VALIDATE="$TOOLS/tag-validate.sh"
LOG="${TAG_LOG_PATH:-$TOOLS/tag-log.jsonl}"

if [[ $# -ne 3 ]]; then
    echo "usage: $0 <tag> <commit-hash> <recognition-string>" >&2
    exit 2
fi

tag="$1"
commit="$2"
recognition="$3"

# Validate; capture the OK line so we can extract coordinates.
ok_line="$("$VALIDATE" "$tag")"
# ok_line: "OK pipeline=PXX layer=LY handle=HHH"
pipeline="$(awk -F'pipeline=| layer=' '{print $2}' <<< "$ok_line")"
layer="$(awk -F'layer=| handle=' '{print $2}' <<< "$ok_line")"
handle="${ok_line##*handle=}"

# Resolve commit timestamp via git
if ! timestamp="$(git -C "$ROOT" show -s --format=%cI "$commit" 2>/dev/null)"; then
    echo "INVALID: cannot resolve commit '$commit' in repo $ROOT" >&2
    exit 1
fi

# Initialize log if absent
[[ -f "$LOG" ]] || : > "$LOG"

# Idempotence + collision check
if [[ -s "$LOG" ]]; then
    # Identical (tag, commit)? -> idempotent no-op
    if jq -e --arg t "$tag" --arg c "$commit" \
            'select(.tag == $t and .commit == $c)' \
            "$LOG" >/dev/null 2>&1; then
        # already logged identically; nothing to do
        echo "OK (idempotent) tag=$tag commit=$commit"
        exit 0
    fi
    # Same coord triple, different commit? -> collision
    other="$(jq -r --arg p "$pipeline" --arg l "$layer" --arg h "$handle" \
                --arg c "$commit" \
                'select(.pipeline == $p and .layer == $l and .handle == $h and .commit != $c) | .commit' \
                "$LOG" | head -n1 || true)"
    if [[ -n "${other:-}" ]]; then
        echo "COLLISION: tag $tag at coord ($pipeline, $layer, $handle) already logged for commit $other" >&2
        exit 1
    fi
fi

# Build the JSON line via jq to get correct escaping
line="$(jq -cn \
    --arg tag "$tag" \
    --arg commit "$commit" \
    --arg timestamp "$timestamp" \
    --arg pipeline "$pipeline" \
    --arg layer "$layer" \
    --arg handle "$handle" \
    --arg recognition "$recognition" \
    '{tag:$tag, commit:$commit, timestamp:$timestamp, pipeline:$pipeline, layer:$layer, handle:$handle, recognition:$recognition}')"

printf '%s\n' "$line" >> "$LOG"
echo "OK appended tag=$tag commit=$commit"
