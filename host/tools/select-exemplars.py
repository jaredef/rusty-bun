#!/usr/bin/env python3
"""Sample an exemplar basket from a parity-results JSON.

Selection strategy: classify every entry by the same Doc 726 probe-shape
× outcome bins the post-sweep classifier uses, then take up to N entries
per bin so a quick sweep covers every shape × outcome cell. The output
is one package name per line, matching parity-top500.txt format, so the
existing parity-measure(-v2).sh harness can run against it unchanged.

Usage:
  select-exemplars.py <results.json> [N_per_bin=4] > exemplars.txt

Bins:
  - PASS reliable: 12 baseline canaries (catch regressions).
  - III.a OK/OK keyCount diff +/-1-2 / 3-10 / >10 / same-keyCount-typeof-diff.
  - III.c rb dyn-import load failed (sub-dep eval/parse gap).
  - III.c rb dyn-import resolve failed.
  - both-ERR (probe-side false-FAIL candidates).
  - bun ERR / rb OK (false-pass candidates per Doc 721 §VI.5).
  - rb ERR other (engine throw on import).
  - Skipped (install failures) -- 2 reps for sanity.

Deterministic ordering (sorted by pkg name within each bin) so the
exemplar file is stable across runs unless the source distribution
shifts.
"""
import json, re, sys, collections

if len(sys.argv) < 2:
    print(__doc__, file=sys.stderr); sys.exit(2)
src = sys.argv[1]
n_per_bin = int(sys.argv[2]) if len(sys.argv) > 2 else 4
n_pass = 12  # baseline canaries

content = open(src).read()
entries = re.findall(r'\{"pkg":"[^"]+",[^\n]*\}', content)

bins = collections.defaultdict(list)
for s in entries:
    pkg_m = re.search(r'"pkg":"([^"]+)"', s)
    if not pkg_m: continue
    pkg = pkg_m.group(1)
    status = re.search(r'"status":"([A-Z_]+)"', s).group(1)
    bun_m = re.search(r'"bun":"((?:[^"\\]|\\.)*)"', s)
    rb_m = re.search(r'"rb":"((?:[^"\\]|\\.)*)"', s)
    bun = bun_m.group(1) if bun_m else ''
    rb = rb_m.group(1) if rb_m else ''
    b_ok = '\\"status\\":\\"OK\\"' in bun
    r_ok = '\\"status\\":\\"OK\\"' in rb
    b_err = '\\"status\\":\\"ERR\\"' in bun
    r_err = '\\"status\\":\\"ERR\\"' in rb

    if status == 'PASS':
        bins['PASS'].append(pkg); continue
    if status == 'SKIP_INSTALL_FAILED':
        bins['SKIP'].append(pkg); continue
    if status == 'TIMEOUT':
        bins['TIMEOUT'].append(pkg); continue
    # status == FAIL
    if b_ok and 'resolve failed' in rb:
        bins['III.c resolve-failed'].append(pkg)
    elif b_ok and 'load failed' in rb:
        bins['III.c load-failed'].append(pkg)
    elif b_ok and r_err:
        bins['rb-ERR-other'].append(pkg)
    elif b_err and r_ok:
        bins['false-pass-candidate'].append(pkg)
    elif b_err and r_err:
        bins['both-ERR'].append(pkg)
    elif b_ok and r_ok:
        bkc = re.search(r'\\"keyCount\\":(\d+)', bun)
        rkc = re.search(r'\\"keyCount\\":(\d+)', rb)
        if bkc and rkc:
            d = int(rkc.group(1)) - int(bkc.group(1))
            if d == 0: bins['III.a typeof-diff'].append(pkg)
            elif abs(d) <= 2: bins['III.a keyCount-±1-2'].append(pkg)
            elif abs(d) <= 10: bins['III.a keyCount-3-10'].append(pkg)
            else: bins['III.a keyCount->10'].append(pkg)

selected = []
# PASS canaries -- a fixed prefix of stably-passing well-known packages,
# falling back to whatever PASS entries exist if the basket doesn't carry
# the canonical names.
canon = ['lodash', 'chalk', 'commander', 'debug', 'ms', 'minimist', 'semver',
        'glob', 'rimraf', 'uuid', 'pino', 'p-defer']
pass_set = set(bins['PASS'])
for c in canon:
    if c in pass_set and c not in selected:
        selected.append(c)
        if len(selected) >= n_pass: break
# Pad with the first N alphabetically if canon coverage is short.
for p in sorted(bins['PASS']):
    if len(selected) >= n_pass: break
    if p not in selected: selected.append(p)

# Per-residual-bin samples.
bin_order = [
    'III.a typeof-diff',
    'III.a keyCount-±1-2',
    'III.a keyCount-3-10',
    'III.a keyCount->10',
    'III.c load-failed',
    'III.c resolve-failed',
    'both-ERR',
    'false-pass-candidate',
    'rb-ERR-other',
    'TIMEOUT',
]
sample_caps = {'TIMEOUT': 2}
for b in bin_order:
    if b not in bins: continue
    cap = sample_caps.get(b, n_per_bin)
    for p in sorted(bins[b])[:cap]:
        if p not in selected: selected.append(p)

# 2 install-skip reps for sanity that the harness still handles them.
for p in sorted(bins.get('SKIP', []))[:2]:
    if p not in selected: selected.append(p)

# Output basket file (comment header + one pkg per line, matches the
# parity-top500.txt format the existing harness expects).
total_fail = sum(len(bins[k]) for k in bins if k not in ('PASS','SKIP','TIMEOUT'))
print(f"# Exemplar basket — sampled from {src}")
print(f"# {n_pass} PASS canaries + up to {n_per_bin} per residual bin")
print(f"# Source distribution: {len(bins['PASS'])} PASS, {total_fail} FAIL, "
      f"{len(bins.get('SKIP', []))} SKIP, {len(bins.get('TIMEOUT', []))} TIMEOUT")
print(f"# Selected {len(selected)} packages.")
print("#")
for p in selected:
    print(p)
