# bun-runtime — root constraint set

*Index of surface constraint modules. Each `@imports` entry points at a per-surface document drafted by `derive-constraints invert`. The runtime's contract is the composition of the imported surface properties.*

@provides: bun-runtime-property
  threshold: COMPOSITE
  interface: []

@imports:
  - property: constructor-handle-js-surface-property
    from: path
    path: ./constructor-handle-js.constraints.md
    as: constructor_handle_js
    # witnessing-clauses: 1574
  - property: regression-surface-property
    from: path
    path: ./regression.constraints.md
    as: regression
    # witnessing-clauses: 1546
  - property: js-surface-property
    from: path
    path: ./js.constraints.md
    as: js
    # witnessing-clauses: 852
  - property: platform-cfg-js-surface-property
    from: path
    path: ./platform-cfg-js.constraints.md
    as: platform_cfg_js
    # witnessing-clauses: 643
  - property: cli-surface-property
    from: path
    path: ./cli.constraints.md
    as: cli
    # witnessing-clauses: 624
  - property: platform-cfg-threaded-js-surface-property
    from: path
    path: ./platform-cfg-threaded-js.constraints.md
    as: platform_cfg_threaded_js
    # witnessing-clauses: 351
  - property: platform-cfg-async-js-surface-property
    from: path
    path: ./platform-cfg-async-js.constraints.md
    as: platform_cfg_async_js
    # witnessing-clauses: 307
  - property: sync-async-regression-surface-property
    from: path
    path: ./sync-async-regression.constraints.md
    as: sync_async_regression
    # witnessing-clauses: 268
  - property: integration-surface-property
    from: path
    path: ./integration.constraints.md
    as: integration
    # witnessing-clauses: 231
  - property: weak-ref-js-surface-property
    from: path
    path: ./weak-ref-js.constraints.md
    as: weak_ref_js
    # witnessing-clauses: 210
  - property: platform-cfg-async-regression-surface-property
    from: path
    path: ./platform-cfg-async-regression.constraints.md
    as: platform_cfg_async_regression
    # witnessing-clauses: 209
  - property: sync-async-js-surface-property
    from: path
    path: ./sync-async-js.constraints.md
    as: sync_async_js
    # witnessing-clauses: 192
  - property: returns-error-success-errors-regression-surface-property
    from: path
    path: ./returns-error-success-errors-regression.constraints.md
    as: returns_error_success_errors_regression
    # witnessing-clauses: 177
  - property: async-js-surface-property
    from: path
    path: ./async-js.constraints.md
    as: async_js
    # witnessing-clauses: 167
  - property: sync-js-surface-property
    from: path
    path: ./sync-js.constraints.md
    as: sync_js
    # witnessing-clauses: 138
  - property: threaded-js-surface-property
    from: path
    path: ./threaded-js.constraints.md
    as: threaded_js
    # witnessing-clauses: 121
  - property: bundler-surface-property
    from: path
    path: ./bundler.constraints.md
    as: bundler
    # witnessing-clauses: 114
  - property: sync-cli-surface-property
    from: path
    path: ./sync-cli.constraints.md
    as: sync_cli
    # witnessing-clauses: 85
  - property: sync-regression-surface-property
    from: path
    path: ./sync-regression.constraints.md
    as: sync_regression
    # witnessing-clauses: 76
  - property: constructor-handle-regression-surface-property
    from: path
    path: ./constructor-handle-regression.constraints.md
    as: constructor_handle_regression
    # witnessing-clauses: 59
  - property: internal-surface-property
    from: path
    path: ./internal.constraints.md
    as: internal
    # witnessing-clauses: 58
  - property: platform-cfg-constructor-handle-js-surface-property
    from: path
    path: ./platform-cfg-constructor-handle-js.constraints.md
    as: platform_cfg_constructor_handle_js
    # witnessing-clauses: 54
  - property: async-cli-surface-property
    from: path
    path: ./async-cli.constraints.md
    as: async_cli
    # witnessing-clauses: 23
  - property: platform-cfg-regression-surface-property
    from: path
    path: ./platform-cfg-regression.constraints.md
    as: platform_cfg_regression
    # witnessing-clauses: 23
  - property: platform-cfg-sync-js-surface-property
    from: path
    path: ./platform-cfg-sync-js.constraints.md
    as: platform_cfg_sync_js
    # witnessing-clauses: 22
  - property: platform-cfg-async-constructor-handle-js-surface-property
    from: path
    path: ./platform-cfg-async-constructor-handle-js.constraints.md
    as: platform_cfg_async_constructor_handle_js
    # witnessing-clauses: 19
  - property: sync-bundler-surface-property
    from: path
    path: ./sync-bundler.constraints.md
    as: sync_bundler
    # witnessing-clauses: 19
  - property: async-returns-error-js-surface-property
    from: path
    path: ./async-returns-error-js.constraints.md
    as: async_returns_error_js
    # witnessing-clauses: 13
  - property: result-shape-js-surface-property
    from: path
    path: ./result-shape-js.constraints.md
    as: result_shape_js
    # witnessing-clauses: 4
  - property: platform-cfg-sync-async-threaded-js-surface-property
    from: path
    path: ./platform-cfg-sync-async-threaded-js.constraints.md
    as: platform_cfg_sync_async_threaded_js
    # witnessing-clauses: 2

@pins: []

## COMPOSITE
type: bridge
authority: derived
scope: system
status: active
depends-on: []

The Bun runtime contract is composed of 30 surface modules drafted from the test corpus. Per [Doc 704 §3](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error), target-language derivation operates over this composition; the constraint set is the durable artifact and target-language implementations are ephemeral cache.

Top surfaces by witnessing-clause count:

- **constructor+handle/@js** — 1574 clauses
- **@regression** — 1546 clauses
- **@js** — 852 clauses
- **platform-cfg/@js** — 643 clauses
- **@cli** — 624 clauses
- **platform-cfg/threaded/@js** — 351 clauses
- **platform-cfg/async/@js** — 307 clauses
- **sync+async/@regression** — 268 clauses
- **@integration** — 231 clauses
- **weak-ref/@js** — 210 clauses
- **platform-cfg/async/@regression** — 209 clauses
- **sync+async/@js** — 192 clauses
- **returns-error/success-errors/@regression** — 177 clauses
- **async/@js** — 167 clauses
- **sync/@js** — 138 clauses
- **threaded/@js** — 121 clauses
- **@bundler** — 114 clauses
- **sync/@cli** — 85 clauses
- **sync/@regression** — 76 clauses
- **constructor+handle/@regression** — 59 clauses
