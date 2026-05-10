# Math — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: math-surface-property
  threshold: MATH1
  interface: [Math.abs]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 11.

## MATH1
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Math.abs** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/websocket/websocket.test.js:866` — instances should be finalized when GC'd → `expect(Math.abs(current_websocket_count - initial_websocket_count)).toBeLessThanOrEqual(50)`
- `test/js/bun/webview/webview-chrome.test.ts:754` — chrome: scrollTo with block: start aligns top → `expect(Math.abs(top)).toBeLessThan(2)`
- `test/js/bun/test/fake-timers/fake-timers.test.ts:224` — Date.now() mocking > Date.now() before and after vi.useFakeTimers() should be roughly equa… → `expect(diff).toBeLessThan(100)`

