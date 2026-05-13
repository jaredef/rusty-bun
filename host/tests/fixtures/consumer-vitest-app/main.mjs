// vitest — load surfaces a destructuring-target edge in vitest's
// runner-or-expect chain (likely a complex left-hand destructure
// pattern beyond current preprocessor). Recorded E.33 vitest-
// destructure-target; deferred to shape-only.
process.stdout.write(JSON.stringify({ probeRan: true }) + "\n");
