#!/usr/bin/env python3
"""Extract a residual cluster from a parity-results JSON.

Companion to select-exemplars.py. Where exemplars *samples* across every
shape × outcome bin (cheap coverage), this script *enumerates* every
package in a single named cluster (targeted iteration). Use for substrate
moves that target one residual cluster — run a sweep against just that
cluster instead of the full canon to measure the move's per-cluster
yield without spending the full 25-min canonical sweep cost.

Usage:
  select-cluster.py <results.json> <cluster-name> > basket.txt

Cluster names (matches the post-sweep classifier in EXT 9/10):
  kc-pm-1-2       — III.a OK/OK keyCount Δ±1-2 (the EXT 9 long tail)
  kc-pm-3-10      — III.a OK/OK keyCount Δ 3-10
  kc-gt-10        — III.a OK/OK keyCount Δ>10
  typeof-diff     — III.a OK/OK same keyCount, type-of-property diff
  dyn-import      — III.c rb dyn-import load failed (general)
  compile-error   — III.c rb dyn-import CompileError sub-bucket
  both-err        — both engines ERR (probe-side false-FAIL candidates)
  bun-err-rb-ok   — Bun ERR but rb OK (Doc 726 §VI.5 false-pass candidates)
  rb-err-other    — rb ERR with no dynamic-import signature

The output is one package name per line, matching parity-top500.txt /
parity-exemplars.txt format. Pipe into parity-measure(-v2).sh or use
parity-cluster-v2.sh which wraps that step.
"""
import json
import sys


def parse(s):
    try:
        return json.loads(s)
    except Exception:
        return None


def classify(entry):
    if entry.get("status") != "FAIL":
        return None
    b = parse(entry.get("bun", ""))
    r = parse(entry.get("rb", ""))
    if not b or not r:
        return "other"
    bs, rs = b.get("status"), r.get("status")
    if bs == "ERR" and rs == "ERR":
        return "both-err"
    if bs == "ERR" and rs == "OK":
        return "bun-err-rb-ok"
    if bs == "OK" and rs == "ERR":
        msg = r.get("message", "")
        if "dynamic import" in msg and "CompileError" in msg:
            return "compile-error"
        if "dynamic import" in msg:
            return "dyn-import"
        return "rb-err-other"
    if bs == "OK" and rs == "OK":
        bk, rk = b.get("keyCount", 0), r.get("keyCount", 0)
        bsh, rsh = b.get("shape", {}), r.get("shape", {})
        if bk == rk:
            diffs = sum(1 for k in bsh if k in rsh and bsh[k] != rsh[k])
            return "typeof-diff" if diffs > 0 else "same-kc-value-diff"
        d = abs(bk - rk)
        if d <= 2:
            return "kc-pm-1-2"
        if d <= 10:
            return "kc-pm-3-10"
        return "kc-gt-10"
    return "other"


def main():
    if len(sys.argv) != 3:
        sys.stderr.write(__doc__)
        sys.exit(2)
    path, want = sys.argv[1], sys.argv[2]

    entries = []
    with open(path) as f:
        for line in f:
            line = line.strip().rstrip(",")
            if not line.startswith("{"):
                continue
            o = parse(line)
            if o:
                entries.append(o)

    # Dedupe by pkg, prefer entry with both bun and rb populated.
    by_pkg = {}
    for e in entries:
        p = e.get("pkg")
        if not p:
            continue
        if p not in by_pkg or (e.get("bun") and not by_pkg[p].get("bun")):
            by_pkg[p] = e

    matched = sorted(p for p, e in by_pkg.items() if classify(e) == want)
    if not matched:
        sys.stderr.write(
            f"warning: no packages match cluster {want!r} in {path}\n"
        )
        sys.exit(1)
    for p in matched:
        print(p)


if __name__ == "__main__":
    main()
