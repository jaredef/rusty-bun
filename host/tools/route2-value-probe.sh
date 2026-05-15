#!/usr/bin/env bash
set -u
RB=/media/jaredef/T7/rusty-bun-target/debug/rusty-bun-host-v2
SANDBOX=/tmp/parity-sandbox
declare -A probes
probes[nanoid]='import { nanoid } from "nanoid"; const id = nanoid(); console.log(typeof id, id?.length===21 ? "OK" : "BAD len="+id?.length);'
probes[uuid]='import { v4 } from "uuid"; const id = v4(); console.log(typeof id, /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(id) ? "OK" : "BAD: "+id);'
probes[picocolors]='import pc from "picocolors"; const r = pc.red("hi"); console.log(typeof r, r.includes("hi") ? "OK" : "BAD: "+r);'
probes[minimatch]='import { minimatch } from "minimatch"; const m = minimatch("foo.js", "*.js"); console.log(typeof m, m===true ? "OK" : "BAD: "+m);'
probes[bignumber.js]='import BigNumber from "bignumber.js"; const r = new BigNumber("1.1").plus("2.2").toString(); console.log(typeof r, r==="3.3" ? "OK" : "BAD: "+r);'
probes[change-case]='import { camelCase } from "change-case"; const r = camelCase("hello world"); console.log(typeof r, r==="helloWorld" ? "OK" : "BAD: "+r);'
probes[dequal]='import { dequal } from "dequal"; const r = dequal({a:1},{a:1}); console.log(typeof r, r===true ? "OK" : "BAD: "+r);'
probes[deepmerge]='import deepmerge from "deepmerge"; const r = deepmerge({a:1},{b:2}); console.log(typeof r, (r.a===1 && r.b===2) ? "OK" : "BAD: "+JSON.stringify(r));'
probes[colorette]='import { red } from "colorette"; const r = red("x"); console.log(typeof r, r.includes("x") ? "OK" : "BAD: "+r);'
probes[mitt]='import mitt from "mitt"; const e = mitt(); let hit=null; e.on("a", v=>hit=v); e.emit("a", 42); console.log(hit===42 ? "OK" : "BAD: "+hit);'
probes[kleur]='import kleur from "kleur"; const r = kleur.red("x"); console.log(r.includes("x") ? "OK" : "BAD: "+r);'
probes[ansi-colors]='import c from "ansi-colors"; const r = c.red("x"); console.log(r.includes("x") ? "OK" : "BAD: "+r);'
probes[micromatch]='import mm from "micromatch"; const r = mm(["a.js","b.txt"], "*.js"); console.log(Array.isArray(r) && r.length===1 ? "OK" : "BAD: "+JSON.stringify(r));'
probes[picomatch]='import pm from "picomatch"; const m = pm("*.js"); console.log(m("foo.js")===true ? "OK" : "BAD");'
probes[pluralize]='import pluralize from "pluralize"; const r = pluralize("car",2); console.log(r==="cars" ? "OK" : "BAD: "+r);'
probes[remeda]='import { sum } from "remeda"; const r = sum([1,2,3]); console.log(r===6 ? "OK" : "BAD: "+r);'
probes[defu]='import defu from "defu"; const r = defu({a:1},{b:2}); console.log((r.a===1 && r.b===2) ? "OK" : "BAD: "+JSON.stringify(r));'
probes[p-defer]='import pDefer from "p-defer"; const d = pDefer(); console.log((typeof d.promise==="object" && typeof d.resolve==="function") ? "OK" : "BAD");'
probes[pathe]='import { join } from "pathe"; const r = join("a","b","c"); console.log(r==="a/b/c" ? "OK" : "BAD: "+r);'
probes[ulid]='import { ulid } from "ulid"; const r = ulid(); console.log(typeof r==="string" && r.length===26 ? "OK" : "BAD: "+r);'
probes[ufo]='import { joinURL } from "ufo"; const r = joinURL("http://a","b","c"); console.log(r.includes("/b/c") ? "OK" : "BAD: "+r);'
probes[object-hash]='import oh from "object-hash"; const r = oh({a:1}); console.log(typeof r==="string" && r.length>0 ? "OK" : "BAD: "+r);'
probes[rfdc]='import rfdc from "rfdc"; const clone = rfdc(); const r = clone({a:[1,2]}); console.log((r.a[0]===1 && r.a[1]===2) ? "OK" : "BAD: "+JSON.stringify(r));'
probes[commander]='import { Command } from "commander"; const c = new Command(); console.log(typeof c.command==="function" ? "OK" : "BAD");'

# Build with bare specifiers — use a tiny shim importing from each pkg dir.
n=0; ok=0; bad=0; err=0
declare -a okp badp errp
for pkg in "${!probes[@]}"; do
  d="$SANDBOX/$pkg"
  [ -d "$d" ] || continue
  n=$((n+1))
  printf '%s\n' "${probes[$pkg]}" > "$d/probe-val.mjs"
  out=$(cd "$d" && timeout 8 "$RB" ./probe-val.mjs 2>&1)
  first=$(echo "$out" | head -1)
  if [[ "$first" == *"OK"* ]]; then ok=$((ok+1)); okp+=("$pkg")
  elif [[ "$first" == *"BAD"* ]]; then bad=$((bad+1)); badp+=("$pkg :: $first")
  else err=$((err+1)); errp+=("$pkg :: $(echo "$out" | tail -1)")
  fi
done
echo "=== ROUTE-2 RESULTS ==="
echo "n=$n  pass=$ok  bad=$bad  err=$err"
echo "-- PASS --"; printf '  %s\n' "${okp[@]}"
echo "-- BAD --"; printf '  %s\n' "${badp[@]}"
echo "-- ERR --"; printf '  %s\n' "${errp[@]}"
