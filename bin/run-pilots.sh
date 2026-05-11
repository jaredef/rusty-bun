#!/usr/bin/env bash
# rusty-bun pilot runner — runs every pilot's verifier + consumer-regression
# suite through the workspace and emits a structured summary.
#
# Per Doc 581 trajectory §II Tier-D #10. Used for the live-state spot-check
# the resume protocol asks for at session start, plus standalone status
# reports.

set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

# --slow includes seed §A8.17 inner-loop-budget tests (bigint/EC/RSA suites
# marked #[ignore] for the inner loop). Run with --slow at engagement-closure
# milestones (Tier-Π phase closure, host-iteration completion).
SLOW_ARGS=""
for arg in "$@"; do
    if [ "$arg" = "--slow" ]; then
        SLOW_ARGS="-- --include-ignored"
    fi
done

OUT=$(cargo test --release --workspace $SLOW_ARGS 2>&1)
EXIT=$?

# Aggregate the per-test-suite "test result:" lines.
SUMMARY=$(echo "$OUT" | grep -E "^test result:" | awk '
    /ok\./   { ok_passed+=$4; ok_failed+=$6; ok_ignored+=$8; ok_suites++ }
    /FAILED/ { fail_passed+=$4; fail_failed+=$6; fail_ignored+=$8; fail_suites++ }
    END {
        printf "Suites OK:     %d\n", ok_suites
        printf "Suites FAIL:   %d\n", fail_suites
        printf "Tests passed:  %d\n", ok_passed + fail_passed
        printf "Tests failed:  %d\n", ok_failed + fail_failed
        printf "Tests ignored: %d\n", ok_ignored + fail_ignored
    }
')

echo "================================================================"
echo "  rusty-bun pilot runner — workspace status"
echo "================================================================"
echo
echo "$SUMMARY"
echo
if [ "$EXIT" != "0" ]; then
    echo "✗ Workspace test run had non-zero exit code: $EXIT"
    echo
    echo "Failures:"
    echo "$OUT" | grep -A2 "FAILED" | head -20
    exit 1
else
    echo "✓ All workspace tests pass."
fi
