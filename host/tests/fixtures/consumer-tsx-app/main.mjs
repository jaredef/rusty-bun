// tsx — CLI runtime wrapper. The package's entry tries to install a
// Node loader hook at import time, which fails outside a CLI context.
// Both Bun and rusty-bun-host raise an error on import. Verify shape
// parity: both throw, both report an error name.
let errored = false;
try {
  await import("tsx");
} catch (e) {
  errored = true;
}
process.stdout.write(JSON.stringify({ errored }) + "\n");
