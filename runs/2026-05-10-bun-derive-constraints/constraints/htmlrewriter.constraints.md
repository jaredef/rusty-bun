# HTMLRewriter — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: htmlrewriter-surface-property
  threshold: HTML1
  interface: [HTMLRewriter]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 10.

## HTML1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**HTMLRewriter** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/js/workerd/html-rewriter.test.js:99` — HTMLRewriter > HTMLRewriter: async replacement → `expect(await res.text()).toBe("<div><span>replace</span></div>")`
- `test/js/workerd/html-rewriter.test.js:271` — HTMLRewriter > handles element specific mutations → `expect(await res.text()).toBe( [ "<p>", "<span>prepend html</span>", "&lt;span&gt;prepend&lt;/span&gt;", "test", "&lt;span&gt;append&lt;/span&gt;", "<span>append html</span>", "</p>", ].join(""), )`
- `test/js/workerd/html-rewriter.test.js:291` — HTMLRewriter > handles element specific mutations → `expect(await res.text()).toBe("<p>&lt;span&gt;replace&lt;/span&gt;</p>")`

