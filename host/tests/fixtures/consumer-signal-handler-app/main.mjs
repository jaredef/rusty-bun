// Π2.6.d.c: process.on('SIGUSR1', fn) handler firing on
// process.kill(process.pid, 'SIGUSR1') via signalfd + reactor.
//
// Uses SIGUSR1 (rather than SIGINT) to avoid interfering with
// any parent test runner's signal handling.

let received = null;
process.on("SIGUSR1", (name) => {
  received = name;
});

// Give the signal pump time to install.
await new Promise(r => setTimeout(r, 10));

process.kill(process.pid, "SIGUSR1");

// Give the reactor + pump time to deliver.
await new Promise(r => setTimeout(r, 50));

process.stdout.write(JSON.stringify({
  received,
}) + "\n");
