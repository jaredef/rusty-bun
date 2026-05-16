#!/usr/bin/env bash
# Doc 724 forward predictor — first cut.
# Static substrate-need mapping from source via grep over the entry-file
# transitive surface of each curated corpus package. Output: per-package
# required-feature set + aggregate frequency table.
#
# This is the simplest possible Doc 724 §IX testable form: grep patterns
# over the on-disk source. A real predictor would walk the AST via the
# rusty-js-parser; v1 trades precision for delivery time. Patterns chosen
# to match the closings the engagement actually produced.
#
# Usage: ./feature-predict.sh [sandbox-root]
# Default sandbox: /tmp/parity-sandbox
#
# Output: feature-prediction.json + a human-readable summary on stdout.

set -uo pipefail
SANDBOX="${1:-/tmp/parity-sandbox}"
TOOLS="$(cd "$(dirname "$0")" && pwd)"
OUT_JSON="$TOOLS/feature-prediction.json"

# Feature table: name | grep regex (extended) | spec section | engine status.
# Engine status: HAVE (implemented), GAP (missing), PARTIAL (partial).
features=$(cat <<'EOF'
padStart|String.prototype.padStart|\.padStart\(|22.1.3.16|HAVE
padEnd|String.prototype.padEnd|\.padEnd\(|22.1.3.17|HAVE
substr|String.prototype.substr|\.substr\(|B.2.2.1|HAVE
substring|String.prototype.substring|\.substring\(|22.1.3.21|HAVE
matchAll|String.prototype.matchAll|\.matchAll\(|22.1.3.13|GAP
replaceAll|String.prototype.replaceAll|\.replaceAll\(|22.1.3.20|GAP
arrayReverse|Array.prototype.reverse|\.reverse\(\)|23.1.3.21|HAVE
arrayFlat|Array.prototype.flat|\.flat\(|23.1.3.10|PARTIAL
arrayFlatMap|Array.prototype.flatMap|\.flatMap\(|23.1.3.11|PARTIAL
arrayGroup|Array.prototype.group|\.group\(|TC39 proposal|GAP
objectAssign|Object.assign|Object\.assign\(|20.1.2.1|HAVE
objectEntries|Object.entries|Object\.entries\(|20.1.2.5|HAVE
objectFromEntries|Object.fromEntries|Object\.fromEntries\(|20.1.2.7|PARTIAL
objectGetPrototypeOf|Object.getPrototypeOf|Object\.getPrototypeOf\(|20.1.2.12|HAVE
objectDefineProperty|Object.defineProperty|Object\.defineProperty\(|20.1.2.4|HAVE
reflectSetPrototypeOf|Reflect.setPrototypeOf|Reflect\.setPrototypeOf\(|28.1.13|HAVE
reflectApply|Reflect.apply|Reflect\.apply\(|28.1.1|HAVE
reflectConstruct|Reflect.construct|Reflect\.construct\(|28.1.2|HAVE
reflectOwnKeys|Reflect.ownKeys|Reflect\.ownKeys\(|28.1.11|HAVE
proxyCtor|new Proxy|new Proxy\(|28.2.1.1|PARTIAL
proxyRevocable|Proxy.revocable|Proxy\.revocable\(|28.2.2|HAVE
generators|function* / yield|function\*|27.5|GAP
asyncFns|async function|async function|27.7|HAVE
asyncIterators|for await of|for await \(|14.7.5|GAP
regexLookbehind|regex (?<=...)|\(\?<=|22.2.1|HAVE
regexLookbehindNeg|regex (?<!...)|\(\?<!|22.2.1|HAVE
regexLookahead|regex (?=...)|\(\?=|22.2.1|HAVE
regexLookaheadNeg|regex (?!...)|\(\?!|22.2.1|HAVE
regexNamedGroup|regex (?<name>...)|\(\?<[a-zA-Z_$]|22.2.1|HAVE
regexBackref|regex \\1..\\9|\\\\[1-9]|22.2.1|HAVE
namedBackref|regex \\k<name>|\\\\k<|22.2.1|HAVE
regexUnicodeProp|regex \\p{...}|\\\\p\{|22.2.1|GAP
regexFlagS|regex /s flag|\(\?-?s\)|22.2.1|PARTIAL
regexFlagY|regex /y flag|\(\?-?y\)|22.2.1|PARTIAL
dateCtor|Date ctor|new Date\(|21.4.2.1|HAVE
datePerf|performance.now|performance\.now\(|18.4|HAVE
mapCtor|Map ctor|new Map\(|24.1.1.2|HAVE
setCtor|Set ctor|new Set\(|24.2.1.1|HAVE
weakMapCtor|WeakMap|new WeakMap\(|24.3.1.1|HAVE
weakSetCtor|WeakSet|new WeakSet\(|24.4.1.1|HAVE
weakRef|WeakRef|new WeakRef\(|26.1.1.1|PARTIAL
symbolFor|Symbol.for|Symbol\.for\(|20.4.2.2|HAVE
symbolIterator|Symbol.iterator|Symbol\.iterator|20.4.2.6|PARTIAL
symbolAsyncIterator|Symbol.asyncIterator|Symbol\.asyncIterator|20.4.2.1|GAP
symbolHasInstance|Symbol.hasInstance|Symbol\.hasInstance|20.4.2.4|GAP
symbolToPrimitive|Symbol.toPrimitive|Symbol\.toPrimitive|20.4.2.13|GAP
classFields|class { #priv = ... }|#[a-zA-Z]+\s*=|15.7|PARTIAL
classStaticFields|class { static field }|static [a-zA-Z]+\s*=|15.7|PARTIAL
optionalChain|obj?.x|\?\.|13.3.9|HAVE
nullishCoalesce|a ?? b|\?\?|13.13|HAVE
logicalAssign|a ??= b ; a &&= b|[?&|]{2}=|13.15|HAVE
templateStr|template literal|\`[^\`]*\$\{|13.2.8|HAVE
taggedTemplate|tagged template|[a-zA-Z]\`|13.2.8.4|HAVE
spreadArg|f(...args)|\.\.\.|13.3.6|HAVE
spreadObj|{...obj}|\{\s*\.\.\.|12.2.6.7|HAVE
destructureObj|const {a} = ...|const \{|13.15.5|HAVE
destructureArr|const [a] = ...|const \[|13.15.5|HAVE
restParam|function(...rest)|\(\.\.\.[a-zA-Z_$]|15.2.5|HAVE
bufferFrom|Buffer.from|Buffer\.from\(|N: Buffer|HAVE
bufferAlloc|Buffer.alloc|Buffer\.alloc\(|N: Buffer|HAVE
bufferCompare|Buffer.compare|Buffer\.compare\(|N: Buffer|HAVE
nodeStreamSubclass|extends Readable/Writable|extends (Readable|Writable|Transform|Duplex)|N: stream|GAP
nodeStreamReadable|stream.Readable|stream\.Readable|N: stream|PARTIAL
nodeCryptoCreateHash|crypto.createHash|createHash\(|N: crypto|HAVE
nodeCryptoCreateHmac|crypto.createHmac|createHmac\(|N: crypto|HAVE
nodeCryptoRandomBytes|crypto.randomBytes|randomBytes\(|N: crypto|HAVE
typedArrayU8|new Uint8Array|new Uint8Array\(|23.2|PARTIAL
typedArraySubarray|.subarray|\.subarray\(|23.2.3|HAVE
typedArraySet|.set on TypedArray|\.set\(|23.2.3|HAVE
urlGlobal|new URL|new URL\(|WHATWG URL|HAVE
fetch|fetch()|fetch\(|WHATWG fetch|PARTIAL
EOF
)

# Process each package directory.
pkgs=$(ls -d "$SANDBOX"/*/node_modules/*/ 2>/dev/null | awk -F/ '{print $(NF-1)}' | sort -u)

declare -A counts
declare -A pkg_features
total_pkgs=0

for pkg in $pkgs; do
  # Find candidate entry files for this package.
  pkg_root=$(find "$SANDBOX" -maxdepth 3 -name "$pkg" -type d -path "*node_modules*" 2>/dev/null | head -1)
  [ -z "$pkg_root" ] && continue
  total_pkgs=$((total_pkgs + 1))

  # Source corpus: main + module + index files within the package.
  src=$(find "$pkg_root" -maxdepth 3 \( -name "*.js" -o -name "*.mjs" -o -name "*.cjs" \) -not -path "*test*" -not -path "*example*" 2>/dev/null | head -10 | xargs cat 2>/dev/null)

  while IFS='|' read -r key label re section status; do
    [ -z "$key" ] && continue
    n=$(echo "$src" | grep -cE "$re" 2>/dev/null | head -1)
    n=${n:-0}
    if [ "${n}" -gt 0 ] 2>/dev/null; then
      counts["$key"]=$(( ${counts["$key"]:-0} + 1 ))
      pkg_features["$pkg"]+="$key,"
    fi
  done <<< "$features"
done

# Emit JSON.
{
  echo "{"
  echo "  \"total_pkgs\": $total_pkgs,"
  echo "  \"feature_counts\": {"
  first=1
  while IFS='|' read -r key label re section status; do
    [ -z "$key" ] && continue
    n="${counts["$key"]:-0}"
    [ "$first" = "1" ] && first=0 || echo ","
    printf '    "%s": {"count": %d, "label": "%s", "spec": "%s", "status": "%s"}' "$key" "$n" "$label" "$section" "$status"
  done <<< "$features"
  echo ""
  echo "  }"
  echo "}"
} > "$OUT_JSON"

# Summary on stdout.
echo "=== Doc 724 forward predictor v1 ==="
echo "packages analyzed: $total_pkgs"
echo
echo "Feature frequency (top 25, sorted by package count):"
echo
while IFS='|' read -r key label re section status; do
  [ -z "$key" ] && continue
  n="${counts["$key"]:-0}"
  echo "$n|$key|$label|$status"
done <<< "$features" | sort -t'|' -k1 -rn | head -25 | while IFS='|' read -r n key label status; do
  printf "%4d %-22s %-40s [%s]\n" "$n" "$key" "$label" "$status"
done
echo
echo "Engine GAP / PARTIAL features touched by ≥3 packages (priority queue):"
echo
while IFS='|' read -r key label re section status; do
  [ -z "$key" ] && continue
  [ "$status" = "HAVE" ] && continue
  n="${counts["$key"]:-0}"
  [ "$n" -lt 3 ] && continue
  echo "$n|$key|$label|$status"
done <<< "$features" | sort -t'|' -k1 -rn | while IFS='|' read -r n key label status; do
  printf "%4d %-22s %-40s [%s]\n" "$n" "$key" "$label" "$status"
done
echo
echo "JSON output: $OUT_JSON"
