# SQL — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sql-surface-property
  threshold: SQL1
  interface: [SQL, SQL.PostgresError, SQL.SQLiteError]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 250.

## SQL1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**SQL** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 220)

Witnessed by 220 constraint clauses across 4 test files. Antichain representatives:

- `test/js/sql/sqlite-sql.test.ts:13` — Connection & Initialization > common default connection strings > should parse common conn… → `expect(memory.options.adapter).toBe("sqlite")`
- `test/js/sql/sql-mysql.test.ts:813` — Connection ended error → `expect(await sql''.catch(x => x.code)).toBe("ERR_MYSQL_CONNECTION_CLOSED")`
- `test/js/sql/adapter-override.test.ts:11` — Adapter Override > postgres:// URL with adapter='sqlite' uses SQLite → `expect(sql.options.adapter).toBe("sqlite")`

## SQL2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**SQL.PostgresError** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:12039` — PostgreSQL tests > Misc > SQL Error Classes > PostgresError class > PostgresError should b… → `expect(typeof SQL.PostgresError).toBe("function")`
- `test/js/sql/sql.test.ts:12053` — PostgreSQL tests > Misc > SQL Error Classes > PostgresError class > PostgresError should e… → `expect(error.message).toBe("Postgres error")`
- `test/js/sql/sql.test.ts:12054` — PostgreSQL tests > Misc > SQL Error Classes > PostgresError class > PostgresError should e… → `expect(error.name).toBe("PostgresError")`

## SQL3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**SQL.SQLiteError** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:12123` — PostgreSQL tests > Misc > SQL Error Classes > SQLiteError class > SQLiteError should be a … → `expect(typeof SQL.SQLiteError).toBe("function")`
- `test/js/sql/sql.test.ts:12135` — PostgreSQL tests > Misc > SQL Error Classes > SQLiteError class > SQLiteError should exten… → `expect(error.message).toBe("SQLite error")`
- `test/js/sql/sql.test.ts:12136` — PostgreSQL tests > Misc > SQL Error Classes > SQLiteError class > SQLiteError should exten… → `expect(error.name).toBe("SQLiteError")`

