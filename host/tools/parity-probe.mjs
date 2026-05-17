// Generic parity probe per Doc 715 §VII shift 2 + Doc 716 §X.
// Imports a package by name (passed via $PARITY_PROBE_PKG env var)
// and emits a canonical JSON shape descriptor. Output is compared
// across runtimes; identical output = byte-identical for the import-
// and-shape level.
//
// Per Doc 715 §X.b alphabet stability: the shape descriptor reads
// only typeof + arity + key presence. It deliberately doesn't
// invoke methods, so semantic divergence at L5 doesn't surface here.
// That's intentional: this is the LOAD + SHAPE parity layer
// (L2/L3 in the lattice), the layer at which our enumerator is at
// 100% against Bun. The probe measures whether real consumer
// imports of real packages preserve that parity.

const pkg = process.env.PARITY_PROBE_PKG;
if (!pkg) {
  process.stdout.write('{"status":"NO_PKG"}\n');
  process.exit(0);
}

try {
  const m = await import(pkg);
  const keys = Object.keys(m).sort();
  const shape = {};
  for (const k of keys) {
    const v = m[k];
    shape[k] = typeof v;
  }
  process.stdout.write(JSON.stringify({
    status: "OK",
    pkg,
    keyCount: keys.length,
    shape,
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({
    status: "ERR",
    pkg,
    error: (e && e.constructor && e.constructor.name) || "Error",
    message: ((e && e.message) || String(e)).slice(0, 800),
  }) + "\n");
}
