# readline — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: readline-surface-property
  threshold: READ1
  interface: [readline.cursorTo, readline.clearLine, readline.createInterface, readline.moveCursor]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 33.

## READ1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.cursorTo** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:236` — readline.cursorTo() > should not throw on undefined or null as stream → `assert.strictEqual(readline.cursorTo(null), true)`
- `test/js/node/readline/readline.node.test.ts:237` — readline.cursorTo() > should not throw on undefined or null as stream → `assert.strictEqual(readline.cursorTo(), true)`
- `test/js/node/readline/readline.node.test.ts:238` — readline.cursorTo() > should not throw on undefined or null as stream → `assert.strictEqual(readline.cursorTo(null, 1, 1, mustCall()), true)`

## READ2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.clearLine** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:123` — readline.clearLine() > should clear to the left of cursor when given -1 as direction → `assert.strictEqual(readline.clearLine(writable, -1), true)`
- `test/js/node/readline/readline.node.test.ts:128` — readline.clearLine() > should clear to the right of cursor when given 1 as direction → `assert.strictEqual(readline.clearLine(writable, 1), true)`
- `test/js/node/readline/readline.node.test.ts:133` — readline.clearLine() > should clear whole line when given 0 as direction → `assert.strictEqual(readline.clearLine(writable, 0), true)`

## READ3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.createInterface** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:1936` — readline.createInterface() > should respond to home and end sequences for common pttys  → `assert.strictEqual(rl.cursor, 3)`
- `test/js/node/readline/readline.node.test.ts:1959` — readline.createInterface() > should respond to home and end sequences for common pttys  → `assert.strictEqual(rl.cursor, 0)`
- `test/js/node/readline/readline.node.test.ts:1961` — readline.createInterface() > should respond to home and end sequences for common pttys  → `assert.strictEqual(rl.cursor, 3)`

## READ4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.moveCursor** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:189` — readline.moveCursor() > shouldn't write when moveCursor(0, 0) is called → `assert.strictEqual(readline.moveCursor(writable, set[0], set[1]), true)`
- `test/js/node/readline/readline.node.test.ts:192` — readline.moveCursor() > shouldn't write when moveCursor(0, 0) is called → `assert.strictEqual(readline.moveCursor(writable, set[0], set[1], mustCall()), true)`
- `test/js/node/readline/readline.node.test.ts:211` — readline.moveCursor() > should not throw on null or undefined stream → `assert.strictEqual(readline.moveCursor(null, 1, 1), true)`

