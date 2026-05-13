// Vite — load reached past parseEnv to an FFI-type-coercion edge
// (an internal callsite passes an object to a string-typed FFI). Shape-only.
process.stdout.write(JSON.stringify({ probeRan: true }) + "\n");
