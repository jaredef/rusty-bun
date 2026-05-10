# Glob — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: glob-surface-property
  threshold: GLOB1
  interface: [Glob, Glob]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 1308.

## GLOB1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Glob** — exhibits the property captured in the witnessing test. (behavioral; cardinality 1222)

Witnessed by 1222 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/glob/match.test.ts:88` — Glob.match > no early globstar lock-in → `expect(new Glob('**/*abc*').match('a/abc')).toBeTrue()`
- `test/js/bun/glob/match.test.ts:89` — Glob.match > no early globstar lock-in → `expect(new Glob('**/*.js').match('a/b.c/c.js')).toBeTrue()`
- `test/js/bun/glob/match.test.ts:90` — Glob.match > no early globstar lock-in → `expect(new Glob("/**/*a").match("/a/a")).toBeTrue()`

## GLOB2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Glob** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 86)

Witnessed by 86 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/glob/match.test.ts:400` — Glob.match > ported from micromatch / glob-match / globlin tests > basic → `expect(new Glob("abc").match("abc")).toBe(true)`
- `test/js/bun/glob/match.test.ts:401` — Glob.match > ported from micromatch / glob-match / globlin tests > basic → `expect(new Glob("*").match("abc")).toBe(true)`
- `test/js/bun/glob/match.test.ts:402` — Glob.match > ported from micromatch / glob-match / globlin tests > basic → `expect(new Glob("*").match("")).toBe(true)`

