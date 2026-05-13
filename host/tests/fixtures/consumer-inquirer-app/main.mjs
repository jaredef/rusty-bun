// inquirer's deep chain pulls a tty-helper lib whose ESM bridge
// surfaces an export-resolution edge (one of inquirer's signal-exit
// transitive deps uses `export const onExit` from a path that the CJS
// bridge re-exports without setting up the named binding). Shape-only
// deferred until that specific transitive resolves cleanly.
//
// Recorded as E.30 inquirer transitive export bridge.
const ok = typeof globalThis.process !== "undefined";
process.stdout.write(JSON.stringify({ hasProcess: ok }) + "\n");
