// tsx — runtime wrapper; both Bun and rusty-bun-host fail to resolve
// the CJS entry (tsx is intended as a CLI). Recorded as load-shape.
process.stdout.write(JSON.stringify({ probeRan: true }) + "\n");
