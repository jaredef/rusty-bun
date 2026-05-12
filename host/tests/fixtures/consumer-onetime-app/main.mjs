// onetime ^8 — ensure a fn runs only once.
import onetime from "onetime";

const lines = [];

let calls = 0;
const fn = onetime(() => { calls++; return calls; });
const a = fn();
const b = fn();
const c = fn();
lines.push("1 calls=" + calls + " a=" + a + " b=" + b + " c=" + c);

let calls2 = 0;
const fn2 = onetime(x => { calls2++; return x * 2; });
lines.push("2 first=" + fn2(5) + " second=" + fn2(99) + " calls=" + calls2);

let err = null;
const fn3 = onetime(() => "ok", { throw: true });
fn3();
try { fn3(); } catch (e) { err = e.message; }
lines.push("3 throws=" + (err !== null));

const fn4 = onetime(() => 42);
lines.push("4 callCount=" + onetime.callCount(fn4));
fn4();
lines.push("5 after=" + onetime.callCount(fn4));

lines.push("6 isFn=" + (typeof onetime === "function"));

process.stdout.write(lines.join("\n") + "\n");
