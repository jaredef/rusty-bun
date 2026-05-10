# os — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: os-surface-property
  threshold: OS1
  interface: [os.userInfo, os.setPriority, os.homedir, os.hostname, os.loadavg, os.release, os.uptime, os.version]

@imports: []

@pins: []

Surface drawn from 8 candidate properties across the Bun test corpus. Construction-style: 8; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 16.

## OS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.userInfo** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:119` — userInfo → `expect(info.username).toBe(process.env.USER)`
- `test/js/node/os/os.test.js:120` — userInfo → `expect(info.shell).toBe(process.env.SHELL || "unknown")`
- `test/js/node/os/os.test.js:124` — userInfo → `expect(info.username).toBe(process.env.USERNAME)`

## OS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.setPriority** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:31` — setPriority → `expect(os.setPriority(0, 10)).toBe(undefined)`
- `test/js/node/os/os.test.js:33` — setPriority → `expect(os.setPriority(0)).toBe(undefined)`
- `test/js/node/os/os.test.js:36` — setPriority → `expect(os.setPriority(0, 2)).toBe(undefined)`

## OS3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.homedir** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:61` — homedir → `expect(os.homedir() !== "unknown").toBe(true)`

## OS4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.hostname** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:88` — hostname → `expect(os.hostname() !== "unknown").toBe(true)`

## OS5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**os.loadavg** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:52` — loadavg → `expect(actual).toBeArrayOfSize(3)`

## OS6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.release** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:96` — release → `expect(os.release().length > 1).toBe(true)`

## OS7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.uptime** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:104` — uptime → `expect(os.uptime() > 0).toBe(true)`

## OS8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.version** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:108` — version → `expect(typeof os.version() === "string").toBe(true)`

