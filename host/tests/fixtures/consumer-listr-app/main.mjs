// zen-observable ^0.10 — TC39 Observable proposal reference (distinct
// from rxjs).
import Observable from "zen-observable";

const lines = [];

const out = [];
new Observable(s => { s.next(1); s.next(2); s.next(3); s.complete(); }).subscribe(v => out.push(v));
lines.push("1 " + JSON.stringify(out));

const out2 = [];
let done = false;
new Observable(s => { s.next("a"); s.complete(); }).subscribe({
  next: v => out2.push(v),
  complete: () => { done = true; },
});
lines.push("2 done=" + done + " out=" + JSON.stringify(out2));

let err = null;
new Observable(s => { s.error(new Error("boom")); }).subscribe({
  error: e => { err = e.message; },
});
lines.push("3 err=" + err);

const out3 = [];
const o = Observable.of(10, 20, 30);
o.subscribe(v => out3.push(v));
lines.push("4 " + JSON.stringify(out3));

const out4 = [];
Observable.from([100, 200]).subscribe(v => out4.push(v));
lines.push("5 " + JSON.stringify(out4));

const out5 = [];
const sub = new Observable(s => {
  s.next(1);
  const id = setInterval(() => s.next(Date.now()), 100);
  return () => clearInterval(id);
}).subscribe(v => out5.push(v));
sub.unsubscribe();
lines.push("6 " + JSON.stringify(out5));

process.stdout.write(lines.join("\n") + "\n");
