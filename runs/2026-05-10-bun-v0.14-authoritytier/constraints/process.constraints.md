# process — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: process-surface-property
  threshold: PROC1
  interface: [process.env, process, process.binding, process.execArgv, process.stderr, process.getuid, process.hasUncaughtExceptionCaptureCallback, process.stdout.json, process, process._exiting, process.argv, process.config.variables.clang, process.config.variables.host_arch, process.config.variables.target_arch, process.constrainedMemory, process.execve]

@imports: []

@pins: []

Surface drawn from 39 candidate properties across the Bun test corpus. Construction-style: 34; behavioral (high-cardinality): 5. Total witnessing constraint clauses: 85.

## PROC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.env** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:153` — process.env → `expect(process.env["LOL SMILE UTF16 😂"]).toBe("😂")`
- `test/js/node/process/process.test.js:155` — process.env → `expect(process.env["LOL SMILE UTF16 😂"]).toBe(undefined)`
- `test/js/node/process/process.test.js:158` — process.env → `expect(process.env["LOL SMILE latin1 <abc>"]).toBe("<abc>")`

## PROC2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process** — exposes values of the expected type or class. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:708` — process.${stub} → `expect(process[stub]()).toBeInstanceOf(Array)`
- `test/js/node/process/process.test.js:724` — process.${stub} → `expect(process[stub]).toBeInstanceOf(Set)`
- `test/js/node/process/process.test.js:731` — process.${stub} → `expect(process[stub]).toBeInstanceOf(Array)`

## PROC3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.binding** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/process-binding.test.ts:5` — process.binding > process.binding('constants') → `expect(constants).toBeDefined()`
- `test/js/node/process-binding.test.ts:15` — process.binding > process.binding('uv') → `expect(uv).toBeDefined()`
- `test/js/node/nodettywrap.test.ts:8` — process.binding('tty_wrap') → `expect(tty_wrap).toBeDefined()`

## PROC4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.execArgv** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/workers/worker.test.ts:187` — web worker > argv / execArgv options → `expect(process.execArgv).toEqual(original_execArgv)`
- `test/js/web/workers/worker.test.ts:389` — worker_threads > worker with argv/execArgv → `expect(process.execArgv).toEqual(original_execArgv)`
- `test/js/node/process/process.test.js:306` — process.execArgv → `expect(process.execArgv instanceof Array).toBe(true)`

## PROC5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.stderr** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/test-process-stdout-async-iterator.test.ts:7` — process.stdout and process.stderr have Symbol.asyncIterator for Node.js compatibility → `expect(typeof process.stderr[Symbol.asyncIterator]).toBe("function")`
- `test/js/node/net/node-net-allowHalfOpen.test.js:91` — allowHalfOpen: true should work on client-side → `expect(result.stderr).toBe("")`
- `test/js/node/net/node-net-allowHalfOpen.test.js:108` — allowHalfOpen: false should work on client-side → `expect(result.stderr).toBe("")`

## PROC6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getuid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:616` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getuid).toBeUndefined()`
- `test/js/node/process/process.test.js:617` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getuid).toBeUndefined()`

## PROC7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.hasUncaughtExceptionCaptureCallback** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:828` — process.hasUncaughtExceptionCaptureCallback → `expect(process.hasUncaughtExceptionCaptureCallback()).toBe(false)`
- `test/js/node/process/process.test.js:830` — process.hasUncaughtExceptionCaptureCallback → `expect(process.hasUncaughtExceptionCaptureCallback()).toBe(true)`

## PROC8
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.stdout.json** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/spawn/readablestream-helpers.test.ts:106` — ReadableStream conversion methods > Bun.spawn() process.stdout.json() should throw on inva… → `expect(result).toBeInstanceOf(Promise)`
- `test/js/bun/spawn/readablestream-helpers.test.ts:127` — ReadableStream conversion methods > Bun.spawn() process.stdout.json() should throw on inva… → `expect(result).toBeInstanceOf(Promise)`

## PROC9
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:700` — process.${stub} → `expect(process[stub]()).toBeUndefined()`

## PROC10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process._exiting** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:1135` — process._exiting → `expect(process._exiting).toBe(false)`

## PROC11
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.argv** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:355` — process.argv in testing → `expect(process.argv).toBeInstanceOf(Array)`

## PROC12
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.config.variables.clang** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:300` — process.config → `expect(process.config.variables.clang).toBeNumber()`

## PROC13
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.config.variables.host_arch** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:301` — process.config → `expect(process.config.variables.host_arch).toBeDefined()`

## PROC14
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.config.variables.target_arch** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:302` — process.config → `expect(process.config.variables.target_arch).toBeDefined()`

## PROC15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.constrainedMemory** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:762` — process.constrainedMemory() → `expect(process.constrainedMemory() >= 0).toBe(true)`

## PROC16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.execve** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process-execve.test.ts:6` — process.execve > is a function → `expect(typeof process.execve).toBe("function")`

## PROC17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getegid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:595` — process.getegid → `expect(typeof process.getegid()).toBe("number")`

## PROC18
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getegid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:612` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getegid).toBeUndefined()`

## PROC19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.geteuid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:598` — process.geteuid → `expect(typeof process.geteuid()).toBe("number")`

## PROC20
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.geteuid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:613` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.geteuid).toBeUndefined()`

## PROC21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getgid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:601` — process.getgid → `expect(typeof process.getgid()).toBe("number")`

## PROC22
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:614` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getgid).toBeUndefined()`

## PROC23
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgroups** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:604` — process.getgroups → `expect(process.getgroups()).toBeInstanceOf(Array)`

## PROC24
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgroups** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:615` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getgroups).toBeUndefined()`

## PROC25
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getuid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:608` — process.getuid → `expect(typeof process.getuid()).toBe("number")`

## PROC26
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.kill** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:653` — signal > process.kill(2) works → `expect(ret).toBe(true)`

## PROC27
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.revision** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/bun-test.test.ts:5` — Bun.version → `expect(process.revision).toBe(Bun.revision)`

## PROC28
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.stderr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process-stdio.test.ts:108` — process-stdio > process.stderr → `expect(process.stderr).toBeDefined()`

## PROC29
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.stdin** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process-stdio.test.ts:8` — process-stdio > process.stdin → `expect(process.stdin).toBeDefined()`

## PROC30
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.stdout** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/test-process-stdout-async-iterator.test.ts:6` — process.stdout and process.stderr have Symbol.asyncIterator for Node.js compatibility → `expect(typeof process.stdout[Symbol.asyncIterator]).toBe("function")`

## PROC31
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.stdout** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process-stdio.test.ts:102` — process-stdio > process.stdout → `expect(process.stdout).toBeDefined()`

## PROC32
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.stdout.bytes** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/spawn/readablestream-helpers.test.ts:177` — ReadableStream conversion methods > Bun.spawn() process.stdout.bytes() should convert stre… → `expect(result).toBeInstanceOf(Uint8Array)`

## PROC33
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.version.startsWith** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:216` — process.version starts with v → `expect(process.version.startsWith("v")).toBeTruthy()`

## PROC34
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.versions.bun** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/bun-test.test.ts:4` — Bun.version → `expect(process.versions.bun).toBe(Bun.version)`

## PROC35
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.exitCode** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/spawn/readablestream-helpers.test.ts:16` — ReadableStream conversion methods > Bun.spawn() process.stdout.text() should capture proce… → `expect(process.exitCode).toBe(0)`
- `test/js/bun/http/serve.test.ts:1737` — should be able to stop in the middle of a file response → `expect(process.exitCode || 0).toBe(0)`
- `test/js/bun/spawn/readablestream-helpers.test.ts:31` — ReadableStream conversion methods > Bun.spawn() process.stdout.text() should capture proce… → `expect(process.exitCode).toBe(0)`

## PROC36
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**process.binding** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/process-binding.test.ts:6` — process.binding > process.binding('constants') → `expect(constants).toHaveProperty("os")`
- `test/js/node/nodettywrap.test.ts:9` — process.binding('tty_wrap') → `expect(tty_wrap).toHaveProperty("TTY")`
- `test/js/node/process-binding.test.ts:7` — process.binding > process.binding('constants') → `expect(constants).toHaveProperty("crypto")`

## PROC37
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.stdout.text** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/14982/14982.test.ts:16` — issue 14982 > does not hang in commander → `expect(await process.stdout.text()).toBe("Test command\n")`
- `test/js/bun/spawn/spawn.test.ts:458` — spawn > pipe > should allow reading stdout > before exit → `expect(output.length).toBe(expected.length)`
- `test/js/bun/spawn/readablestream-helpers.test.ts:15` — ReadableStream conversion methods > Bun.spawn() process.stdout.text() should capture proce… → `expect(result).toBe("Hello from Bun spawn! 🚀\n")`

## PROC38
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.title** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:87` — process.title with UTF-16 characters → `expect(process.title).toBe("Hello, 世界! 🌍")`
- `test/js/node/process/process.test.js:91` — process.title with UTF-16 characters → `expect(process.title).toBe("🌍🌎🌏")`
- `test/js/node/process/process.test.js:95` — process.title with UTF-16 characters → `expect(process.title).toBe("Test 测试 тест")`

## PROC39
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.umask** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:261` — process.umask() → `expect(orig).toBe(0)`
- `test/js/node/process/process.test.js:265` — process.umask() → `expect(process.umask()).toBe(mask)`
- `test/js/node/process/process.test.js:266` — process.umask() → `expect(process.umask(undefined)).toBe(mask)`

