#!/usr/bin/env bash
# Expanded route-2 value probe — covers shape-passers beyond the
# original 24-package sample.
set -u
RB=/media/jaredef/T7/rusty-bun-target/release/rusty-bun-host-v2
SANDBOX="${PARITY_SANDBOX:-/tmp/parity-sandbox}"
declare -A probes

# === Original 24 (carry forward) ===
probes[nanoid]='import { nanoid } from "nanoid"; const id = nanoid(); console.log(typeof id==="string" && id.length===21 ? "OK" : "BAD");'
probes[uuid]='import { v4 } from "uuid"; const id = v4(); console.log(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(id) ? "OK" : "BAD");'
probes[picocolors]='import pc from "picocolors"; const r = pc.red("hi"); console.log(r.includes("hi") ? "OK" : "BAD");'
probes[minimatch]='import { minimatch } from "minimatch"; console.log(minimatch("foo.js", "*.js")===true ? "OK" : "BAD");'
probes[bignumber.js]='import BigNumber from "bignumber.js"; console.log(new BigNumber("1.1").plus("2.2").toString()==="3.3" ? "OK" : "BAD");'
probes[change-case]='import { camelCase } from "change-case"; console.log(camelCase("hello world")==="helloWorld" ? "OK" : "BAD");'
probes[dequal]='import { dequal } from "dequal"; console.log(dequal({a:1},{a:1})===true ? "OK" : "BAD");'
probes[deepmerge]='import deepmerge from "deepmerge"; const r = deepmerge({a:1},{b:2}); console.log((r.a===1 && r.b===2) ? "OK" : "BAD");'
probes[colorette]='import { red } from "colorette"; console.log(red("x").includes("x") ? "OK" : "BAD");'
probes[mitt]='import mitt from "mitt"; const e = mitt(); let h=null; e.on("a", v=>h=v); e.emit("a", 42); console.log(h===42 ? "OK" : "BAD");'
probes[kleur]='import kleur from "kleur"; console.log(kleur.red("x").includes("x") ? "OK" : "BAD");'
probes[ansi-colors]='import c from "ansi-colors"; console.log(c.red("x").includes("x") ? "OK" : "BAD");'
probes[micromatch]='import mm from "micromatch"; console.log(mm(["a.js","b.txt"], "*.js").length===1 ? "OK" : "BAD");'
probes[picomatch]='import pm from "picomatch"; console.log(pm("*.js")("foo.js")===true ? "OK" : "BAD");'
probes[pluralize]='import pluralize from "pluralize"; console.log(pluralize("car",2)==="cars" ? "OK" : "BAD");'
probes[remeda]='import { sum } from "remeda"; console.log(sum([1,2,3])===6 ? "OK" : "BAD");'
probes[defu]='import defu from "defu"; const r = defu({a:1},{b:2}); console.log((r.a===1 && r.b===2) ? "OK" : "BAD");'
probes[p-defer]='import pDefer from "p-defer"; const d = pDefer(); console.log((typeof d.promise==="object" && typeof d.resolve==="function") ? "OK" : "BAD");'
probes[pathe]='import { join } from "pathe"; console.log(join("a","b","c")==="a/b/c" ? "OK" : "BAD");'
probes[ulid]='import { ulid } from "ulid"; const r = ulid(); console.log(typeof r==="string" && r.length===26 ? "OK" : "BAD");'
probes[ufo]='import { joinURL } from "ufo"; console.log(joinURL("http://a","b","c").includes("/b/c") ? "OK" : "BAD");'
probes[object-hash]='import oh from "object-hash"; const r = oh({a:1}); console.log(typeof r==="string" && r.length>0 ? "OK" : "BAD");'
probes[rfdc]='import rfdc from "rfdc"; const c = rfdc(); const r = c({a:[1,2]}); console.log((r.a[0]===1 && r.a[1]===2) ? "OK" : "BAD");'
probes[commander]='import { Command } from "commander"; const c = new Command(); console.log(typeof c.command==="function" ? "OK" : "BAD");'

# === Expansion (newly added) ===
probes[acorn]='import * as acorn from "acorn"; const ast = acorn.parse("1+2", {ecmaVersion: 2020}); console.log(ast.type==="Program" ? "OK" : "BAD");'
probes[acorn-walk]='import * as walk from "acorn-walk"; console.log(typeof walk.simple==="function" ? "OK" : "BAD");'
probes[astring]='import { generate } from "astring"; const code = generate({type:"Literal", value:42, raw:"42"}); console.log(typeof code==="string" && code.includes("42") ? "OK" : "BAD");'
probes[bcryptjs]='import bcrypt from "bcryptjs"; console.log(typeof bcrypt.hash==="function" && typeof bcrypt.compare==="function" ? "OK" : "BAD");'
probes[camelcase]='import camelCase from "camelcase"; console.log(camelCase("hello world")==="helloWorld" ? "OK" : "BAD");'
probes[csv-parse]='import { parse } from "csv-parse/sync"; const r = parse("a,b\n1,2"); console.log(Array.isArray(r) && r.length===2 ? "OK" : "BAD");'
probes[csv-parser]='import csv from "csv-parser"; console.log(typeof csv==="function" ? "OK" : "BAD");'
probes[date-fns]='import { format } from "date-fns/format"; const d = new Date(2026, 4, 15); console.log(format(d, "yyyy-MM-dd")==="2026-05-15" ? "OK" : "BAD");'
probes[dayjs]='import dayjs from "dayjs"; const d = dayjs("2026-05-15"); console.log(d.format("YYYY-MM-DD")==="2026-05-15" ? "OK" : "BAD");'
probes[decimal.js]='import Decimal from "decimal.js"; console.log(new Decimal("0.1").plus("0.2").toString()==="0.3" ? "OK" : "BAD");'
probes[destr]='import destr from "destr"; console.log(destr("123")===123 && destr("true")===true ? "OK" : "BAD");'
probes[escodegen]='import { generate } from "escodegen"; const code = generate({type:"Literal", value:42}); console.log(code==="42" ? "OK" : "BAD: "+code);'
probes[esutils]='import esutils from "esutils"; console.log(typeof esutils.code==="object" ? "OK" : "BAD");'
probes[fast-deep-equal]='import equal from "fast-deep-equal"; console.log(equal({a:1},{a:1})===true ? "OK" : "BAD");'
probes[fast-equals]='import { deepEqual } from "fast-equals"; console.log(deepEqual({a:1},{a:1})===true ? "OK" : "BAD");'
probes[hookable]='import { createHooks } from "hookable"; const h = createHooks(); console.log(typeof h.hook==="function" ? "OK" : "BAD");'
probes[immer]='import { produce } from "immer"; const r = produce({a:1}, d=>{d.a=2}); console.log(r.a===2 ? "OK" : "BAD");'
probes[into-stream]='import intoStream from "into-stream"; console.log(typeof intoStream==="function" ? "OK" : "BAD");'
probes[just-curry-it]='import curry from "just-curry-it"; const add = curry((a,b)=>a+b); console.log(add(1)(2)===3 ? "OK" : "BAD");'
probes[loglevel]='import log from "loglevel"; log.setLevel("silent"); console.log(typeof log.info==="function" ? "OK" : "BAD");'
probes[magic-string]='import MagicString from "magic-string"; const s = new MagicString("hello"); s.append(" world"); console.log(s.toString()==="hello world" ? "OK" : "BAD");'
probes[markdown-it]='import md from "markdown-it"; const m = md(); console.log(m.render("# Hi").includes("<h1>") ? "OK" : "BAD");'
probes[marked]='import { marked } from "marked"; console.log(marked.parse("# Hi").includes("<h1>") ? "OK" : "BAD");'
probes[merge-options]='import merge from "merge-options"; const r = merge({a:1},{b:2}); console.log((r.a===1 && r.b===2) ? "OK" : "BAD");'
probes[micromark]='import { micromark } from "micromark"; console.log(micromark("# Hi").includes("<h1>") ? "OK" : "BAD");'
probes[minimist]='import parseArgs from "minimist"; const a = parseArgs(["--foo","bar"]); console.log(a.foo==="bar" ? "OK" : "BAD");'
probes[moment]='import moment from "moment"; console.log(moment("2026-05-15").format("YYYY-MM-DD")==="2026-05-15" ? "OK" : "BAD");'
probes[nanostores]='import { atom } from "nanostores"; const a = atom(1); console.log(a.get()===1 ? "OK" : "BAD");'
probes[ndjson]='import * as ndjson from "ndjson"; console.log(typeof ndjson.parse==="function" || typeof ndjson.default?.parse==="function" ? "OK" : "BAD");'
probes[p-finally]='import pFinally from "p-finally"; console.log(typeof pFinally==="function" ? "OK" : "BAD");'
probes[p-limit]='import pLimit from "p-limit"; const l = pLimit(1); console.log(typeof l==="function" ? "OK" : "BAD");'
probes[p-queue]='import PQueue from "p-queue"; const q = new PQueue({concurrency:1}); console.log(typeof q.add==="function" ? "OK" : "BAD");'
probes[p-retry]='import pRetry from "p-retry"; console.log(typeof pRetry==="function" ? "OK" : "BAD");'
probes[p-throttle]='import pThrottle from "p-throttle"; const t = pThrottle({limit:1, interval:1}); console.log(typeof t==="function" ? "OK" : "BAD");'
probes[p-timeout]='import pTimeout from "p-timeout"; console.log(typeof pTimeout==="function" ? "OK" : "BAD");'
probes[pako]='import pako from "pako"; const c = pako.deflate("hello"); console.log(c.length > 0 ? "OK" : "BAD");'
probes[rxjs]='import { of } from "rxjs"; const o = of(1,2,3); console.log(typeof o.subscribe==="function" ? "OK" : "BAD");'
probes[scrypt-js]='import { scrypt } from "scrypt-js"; console.log(typeof scrypt==="function" ? "OK" : "BAD");'
probes[split2]='import split2 from "split2"; const s = split2(); console.log(typeof s.on==="function" ? "OK" : "BAD");'
probes[superstruct]='import { string, validate } from "superstruct"; const [err,v]=validate("hi", string()); console.log(v==="hi" ? "OK" : "BAD");'
probes[ts-pattern]='import { match } from "ts-pattern"; const r = match(1).with(1,()=>"one").otherwise(()=>"other"); console.log(r==="one" ? "OK" : "BAD");'
probes[tweetnacl]='import nacl from "tweetnacl"; const kp = nacl.box.keyPair(); console.log(kp.publicKey.length===32 ? "OK" : "BAD");'
probes[unified]='import { unified } from "unified"; const u = unified(); console.log(typeof u.use==="function" ? "OK" : "BAD");'
probes[upath]='import upath from "upath"; console.log(upath.normalize("a//b")==="a/b" ? "OK" : "BAD");'
probes[valibot]='import { string, parse } from "valibot"; console.log(parse(string(), "hi")==="hi" ? "OK" : "BAD");'
probes[zod]='import { z } from "zod"; const s = z.string(); console.log(s.parse("hi")==="hi" ? "OK" : "BAD");'
probes[fast-glob]='import fg from "fast-glob"; console.log(typeof fg.sync==="function" ? "OK" : "BAD");'
probes[ts-pattern]='import { match } from "ts-pattern"; const r = match(1).with(1,()=>"one").otherwise(()=>"other"); console.log(r==="one" ? "OK" : "BAD");'

# Run
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
echo "=== EXPANDED ROUTE-2 ==="
echo "n=$n  pass=$ok  bad=$bad  err=$err"; printf "ERR: %s\n" "${errp[@]}"; printf "BAD: %s\n" "${badp[@]}"
