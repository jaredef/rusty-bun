# Markdown — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: markdown-surface-property
  threshold: MARK1
  interface: [Markdown.html, Markdown.render, Markdown.html, Markdown.render, Markdown.react]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 5. Total witnessing constraint clauses: 110.

## MARK1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Markdown.html** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 45)

Witnessed by 45 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/md/md-heading-ids.test.ts:12` — headingIds option > basic heading gets an id attribute → `expect(result).toBe('<h2 id="hello-world">Hello World</h2>\n')`
- `test/js/bun/md/md-edge-cases.test.ts:14` — fuzzer-like edge cases > empty string produces empty output across all APIs → `expect(Markdown.html("")).toBe("")`
- `test/js/bun/md/md-heading-ids.test.ts:19` — headingIds option > heading levels 1-6 all get ids → `expect(result).toBe('<h${i} id="test">Test</h${i}>\n')`

## MARK2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Markdown.render** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 34)

Witnessed by 34 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/md/md-render-callback.test.ts:14` — Bun.markdown.render > returns a string → `expect(typeof result).toBe("string")`
- `test/js/bun/md/md-edge-cases.test.ts:15` — fuzzer-like edge cases > empty string produces empty output across all APIs → `expect(Markdown.render("", {})).toBe("")`
- `test/js/bun/md/md-render-callback.test.ts:19` — Bun.markdown.render > without callbacks, children pass through unchanged → `expect(result).toBe("Hello world")`

## MARK3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Markdown.html** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/md/md-heading-ids.test.ts:36` — headingIds option > duplicate headings get deduplicated with -N suffix → `expect(result).toContain('<h2 id="foo">Foo</h2>')`
- `test/js/bun/md/md-edge-cases.test.ts:32` — fuzzer-like edge cases > null bytes are replaced with U+FFFD → `expect(html).toContain("\uFFFD")`
- `test/js/bun/md/md-heading-ids.test.ts:37` — headingIds option > duplicate headings get deduplicated with -N suffix → `expect(result).toContain('<h2 id="foo-1">Foo</h2>')`

## MARK4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Markdown.render** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/md/md-render-callback.test.ts:348` — Bun.markdown.render > parser options work alongside callbacks → `expect(result).toContain("[www.example.com]")`
- `test/js/bun/md/md-edge-cases.test.ts:261` — fuzzer-like edge cases > all options work with render() → `expect(result).toContain("[H1:")`
- `test/js/bun/md/md-render-callback.test.ts:371` — Bun.markdown.render > table callbacks → `expect(result).toContain("<table>")`

## MARK5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Markdown.react** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/md/md-react.test.ts:29` — Bun.markdown.react > returns a Fragment element → `expect(result.$$typeof).toBe(REACT_TRANSITIONAL_SYMBOL)`
- `test/js/bun/md/md-react.test.ts:30` — Bun.markdown.react > returns a Fragment element → `expect(result.type).toBe(REACT_FRAGMENT_SYMBOL)`
- `test/js/bun/md/md-react.test.ts:141` — Bun.markdown.react > default $$typeof is react.transitional.element → `expect(result.$$typeof).toBe(REACT_TRANSITIONAL_SYMBOL)`

