# Database — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: database-surface-property
  threshold: DATA1
  interface: [Database.open, Database]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 35.

## DATA1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Database.open** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 19)

Witnessed by 19 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/sqlite/sqlite.test.js:148` — ${JSON.stringify(input)} -> ${query} → `expect(db.query("SELECT * FROM cats").all()).toStrictEqual([ { id: 1, name: "myname", age: 42, }, ])`
- `test/js/bun/sqlite/sqlite.test.js:155` — ${JSON.stringify(input)} -> ${query} → `expect(db.query('SELECT * FROM cats WHERE (name, age) = (${query})').all(input)).toStrictEqual([ { id: 1, name: "myname", age: 42 }, ])`
- `test/js/bun/sqlite/sqlite.test.js:158` — ${JSON.stringify(input)} -> ${query} → `expect(db.query('SELECT * FROM cats WHERE (name, age) = (${query})').get(input)).toStrictEqual({ id: 1, name: "myname", age: 42, })`

## DATA2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Database** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 16)

Witnessed by 16 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/14709.test.ts:20` — db.close(true) works after db.transaction() with actual work → `expect(db.query("SELECT COUNT(*) as count FROM test").get()).toEqual({ count: 3 })`
- `test/js/bun/sqlite/sqlite.test.js:912` — latin1 supplement chars → `expect(db.query("SELECT * FROM foo").all()).toEqual([ { id: 1, greeting: "Welcome to bun!", }, { id: 2, greeting: "Español", }, { id: 3, greeting: "¿Qué sucedió?", }, ])`
- `test/js/bun/sqlite/sqlite.test.js:930` — latin1 supplement chars → `expect(db.query("SELECT * FROM foo").all()).toEqual([ { id: 1, greeting: "Welcome to bun!", }, { id: 2, greeting: "Español", }, { id: 3, greeting: "¿Qué sucedió?", }, ])`

