// rxjs ^7 — reactive Observable streams. Distinct paradigm (push-based
// async sequences with composable operators).
import { Observable, from, of, range, map, filter, take, reduce, mergeMap, firstValueFrom, lastValueFrom, Subject } from "rxjs";

const lines = [];

async function main() {
  // 1: of() → values
  {
    const out = [];
    of(1, 2, 3).subscribe(v => out.push(v));
    lines.push("1 " + JSON.stringify(out));
  }

  // 2: from(array) + map + filter pipeline
  {
    const out = [];
    from([1, 2, 3, 4, 5])
      .pipe(map(n => n * n), filter(n => n > 5))
      .subscribe(v => out.push(v));
    lines.push("2 " + JSON.stringify(out));
  }

  // 3: range + take + reduce + firstValueFrom
  {
    const sum = await firstValueFrom(
      range(1, 10).pipe(reduce((acc, n) => acc + n, 0))
    );
    lines.push("3 sum1to10=" + sum);
  }

  // 4: Subject — multicast
  {
    const subj = new Subject();
    const a = [], b = [];
    subj.subscribe(v => a.push(v));
    subj.subscribe(v => b.push(v));
    subj.next(1); subj.next(2); subj.next(3);
    subj.complete();
    lines.push("4 a=" + JSON.stringify(a) + " b=" + JSON.stringify(b));
  }

  // 5: error path
  {
    const out = [];
    let err = null;
    new Observable(s => { s.next(1); s.error(new Error("boom")); }).subscribe({
      next: v => out.push(v),
      error: e => { err = e.message; },
    });
    lines.push("5 out=" + JSON.stringify(out) + " err=" + err);
  }

  // 6: mergeMap async composition
  {
    const xs = await lastValueFrom(
      from([1, 2, 3]).pipe(
        mergeMap(n => of(n * 10, n * 100)),
        reduce((acc, v) => acc.concat(v), []),
      )
    );
    lines.push("6 " + JSON.stringify(xs.sort((a, b) => a - b)));
  }
}

try { await main(); process.stdout.write(lines.join("\n") + "\n"); }
catch (e) { process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n"); }
