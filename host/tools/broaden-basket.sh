#!/usr/bin/env bash
# Doc 724 §IX scale-up — install net-new packages into the parity sandbox
# matching the existing /tmp/parity-sandbox/<pkg>/node_modules/<pkg>/
# convention. Skips packages already present.
#
# Usage: ./broaden-basket.sh [N=80]   (default install batch size)
set -uo pipefail
N="${1:-80}"
SANDBOX="${PARITY_SANDBOX:-/tmp/parity-sandbox}"
TOOLS="$(cd "$(dirname "$0")" && pwd)"
LIST="$TOOLS/parity-top500.txt"

# Candidates: top500 ∖ already-installed, no scoped pkgs, skip known-binding.
candidates=$(
  grep -vE '^(#|$)' "$LIST" | sort -u |
  grep -vxFf <(ls "$SANDBOX" 2>/dev/null | sort -u) |
  grep -v '^@' |
  grep -vE '^(prisma-client|drizzle-orm|typeorm|mongodb|redis|ioredis|nanopg|xlsx|exceljs|sharp|fake-indexeddb|node-notifier|reflect-metadata|inversify|node-config|encoding-japanese)$' |
  head -"$N"
)

count=0
ok=0
fail=0
for pkg in $candidates; do
  count=$((count + 1))
  d="$SANDBOX/$pkg"
  mkdir -p "$d"
  ( cd "$d" && echo '{"name":"sb","dependencies":{"'"$pkg"'":"*"}}' > package.json && bun install --silent 2>/dev/null >/dev/null )
  if [ -d "$d/node_modules/$pkg" ]; then
    ok=$((ok + 1))
    printf "."
  else
    fail=$((fail + 1))
    printf "x"
    rm -rf "$d"
  fi
done
echo
echo "broadened: $ok installed, $fail failed (of $count candidates)"
