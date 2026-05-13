#!/usr/bin/env bash
# Substrate-usage ranker per Doc 715 §VII shift 2 + §X.a + §XI.f.
set -uo pipefail
FIX="${1:-/home/jaredef/rusty-bun/host/tests/fixtures}"

WIRED='node:fs fs node:path path node:http http node:crypto crypto node:buffer buffer node:url url node:os os node:process process node:dns dns node:dns/promises dns/promises node:events events node:util util node:util/types util/types node:stream stream node:stream/promises stream/promises node:querystring querystring node:assert assert node:assert/strict assert/strict node:child_process child_process node:net net node:tty tty node:zlib zlib node:diagnostics_channel diagnostics_channel node:https https node:perf_hooks perf_hooks node:async_hooks async_hooks node:timers timers node:timers/promises timers/promises node:console console node:fs/promises fs/promises node:stream/web stream/web node:test test node:worker_threads worker_threads node:http2 http2 node:vm vm node:string_decoder string_decoder node:readline readline node:readline/promises readline/promises node:module module node:cluster cluster node:tls tls node:v8 v8 node:constants constants'

# Builtin names without "node:" prefix
BUILTIN_NAMES=$(echo $WIRED | tr ' ' '\n' | grep -v "^node:")

find "$FIX" -path "*/node_modules/*" \( -name "*.js" -o -name "*.mjs" \) -print0 2>/dev/null |
  xargs -0 -P 8 -n 200 grep -hE "require\(['\"]([^./][^'\"]*)['\"]\)|from ['\"]([^./][^'\"]*)['\"]" 2>/dev/null |
  perl -ne '
    while (/(?:require\(|from\s+)['\''"]([^'\''"]+)['\''"]/g) {
      my $s = $1;
      next if $s =~ m{^\.{1,2}/};
      next if $s =~ m{^/};
      if ($s =~ m{^node:[^/]+(?:/[^/]+)?}) { print "$&\n"; }
      elsif ($s =~ m{^(@[^/]+/[^/]+)}) { print "$1\n"; }
      elsif ($s =~ m{^([^/]+)}) { print "$1\n"; }
    }
  ' |
  sort | uniq -c | sort -rn | awk -v wired="$WIRED" -v bn="$BUILTIN_NAMES" '
    BEGIN {
      n = split(wired, w, " ")
      for (i = 1; i <= n; i++) wired_set[w[i]] = 1
      m = split(bn, bs, "\n")
      for (i = 1; i <= m; i++) if (bs[i] != "") builtin_set[bs[i]] = 1
      print "=== Substrate nodes — ranked by usage =================="
      printf "%-8s %-30s %s\n", "COUNT", "NAME", "STATE"
      print "------- ------------------------------ -------"
    }
    {
      count = $1
      name = $2
      is_builtin = 0
      if (name ~ /^node:/) is_builtin = 1
      else if (builtin_set[name]) is_builtin = 1
      if (is_builtin) {
        state = (wired_set[name] ? "[WIRED]" : "[OPEN]")
        printf "%-8s %-30s %s\n", count, name, state
        builtins[name] = count
      } else {
        if (npm_count < 30) { npm_names[++npm_count] = name; npm_counts[npm_count] = count }
      }
    }
    END {
      print ""
      print "=== Top npm packages by usage ==========================="
      printf "%-8s %s\n", "COUNT", "NAME"
      print "------- ------------------------------"
      for (i = 1; i <= npm_count; i++) printf "%-8s %s\n", npm_counts[i], npm_names[i]
    }
  '
