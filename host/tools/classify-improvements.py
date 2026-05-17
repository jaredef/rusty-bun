#!/usr/bin/env python3
"""Post-hoc classifier: surface FAIL entries that are actually rusty-js
improvements over Bun (seed §A8.24's "telos has two reads" recognition).

For each FAIL where both engines produce {"status":"OK", ...}, examine
the key-set diff and classify against a fixed catalogue of Bun-laxness
signatures. Each match is a candidate `IMPROVES_OVER_BUN` reclassification.

The classifier does NOT auto-promote — it just surfaces the candidates
for keeper review. The decision of whether to keep spec-correctness or
match Bun for ecosystem compatibility per package belongs to the keeper.

Usage:
  classify-improvements.py <results.json>

Signatures recognized at v1:

  __esModule-leak (Bun-laxness)
    Bun's namespace includes `__esModule` from a non-enumerable CJS
    `Object.defineProperty(exports, '__esModule', { value: true })`.
    ECMA §6.2.5.4 defaults enumerable to false; we filter, Bun doesn't.
    Pattern: bun_keys - rb_keys == {'__esModule'}.

  null-prototype-extras (Bun-laxness)
    Bun surfaces inherited Function.prototype properties via a CJS
    wrapper. We don't. Pattern: bun_keys - rb_keys ⊆
    {'apply', 'bind', 'call', 'caller', 'arguments', 'constructor'}.

Add new signatures as they surface from chain-walks.
"""
import json, re, sys
from collections import defaultdict

if len(sys.argv) < 2:
    print(__doc__, file=sys.stderr); sys.exit(2)

content = open(sys.argv[1]).read()
entries = re.findall(r'\{"pkg":"[^"]+",[^\n]*\}', content)

SIGS = []

def sig_esmodule_leak(bun_keys, rb_keys, bkc, rkc, **_):
    return (bun_keys - rb_keys) == {'__esModule'} and not (rb_keys - bun_keys)

def sig_fn_proto_extras(bun_keys, rb_keys, bkc, rkc, **_):
    diff = bun_keys - rb_keys
    if not diff or (rb_keys - bun_keys): return False
    fn_proto = {'apply', 'bind', 'call', 'caller', 'arguments', 'constructor', 'toString'}
    return diff.issubset(fn_proto)

SIGS = [
    ('__esModule-leak', sig_esmodule_leak),
    ('fn-prototype-extras', sig_fn_proto_extras),
]

candidates = defaultdict(list)
total_fail = 0
for s in entries:
    if '"status":"FAIL"' not in s: continue
    total_fail += 1
    pkg_m = re.search(r'"pkg":"([^"]+)"', s)
    if not pkg_m: continue
    pkg = pkg_m.group(1)
    bun_m = re.search(r'"bun":"((?:[^"\\]|\\.)*)"', s)
    rb_m = re.search(r'"rb":"((?:[^"\\]|\\.)*)"', s)
    if not bun_m or not rb_m: continue
    bun_raw = bun_m.group(1).encode().decode('unicode_escape')
    rb_raw = rb_m.group(1).encode().decode('unicode_escape')
    try:
        bun = json.loads(bun_raw)
        rb = json.loads(rb_raw)
    except Exception:
        continue
    if bun.get('status') != 'OK' or rb.get('status') != 'OK':
        continue
    bks = set(bun.get('shape', {}))
    rks = set(rb.get('shape', {}))
    bkc = bun.get('keyCount', 0)
    rkc = rb.get('keyCount', 0)
    for name, fn in SIGS:
        if fn(bks, rks, bkc, rkc, bun=bun, rb=rb):
            candidates[name].append(pkg)

print(f"=== IMPROVES_OVER_BUN candidates (of {total_fail} FAILs) ===\n")
total = 0
for name, pkgs in sorted(candidates.items()):
    print(f"{name}: {len(pkgs)} packages")
    for p in sorted(pkgs)[:10]:
        print(f"  {p}")
    if len(pkgs) > 10:
        print(f"  ... and {len(pkgs)-10} more")
    print()
    total += len(pkgs)

print(f"Total improvement candidates: {total}")
print(f"  ({100*total/total_fail:.1f}% of current FAILs)" if total_fail else "")
