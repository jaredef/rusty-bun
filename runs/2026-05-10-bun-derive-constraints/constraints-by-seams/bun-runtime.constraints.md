# bun-runtime — root constraint set

*Index of surface constraint modules. Each `@imports` entry points at a per-surface document drafted by `derive-constraints invert`. The runtime's contract is the composition of the imported surface properties.*

@provides: bun-runtime-property
  threshold: COMPOSITE
  interface: []

@imports:
  - property: regression-surface-property
    from: path
    path: ./regression.constraints.md
    as: regression
    # witnessing-clauses: 1828
  - property: constructor-handle-js-surface-property
    from: path
    path: ./constructor-handle-js.constraints.md
    as: constructor_handle_js
    # witnessing-clauses: 1645
  - property: js-surface-property
    from: path
    path: ./js.constraints.md
    as: js
    # witnessing-clauses: 959
  - property: platform-cfg-js-surface-property
    from: path
    path: ./platform-cfg-js.constraints.md
    as: platform_cfg_js
    # witnessing-clauses: 847
  - property: async-js-surface-property
    from: path
    path: ./async-js.constraints.md
    as: async_js
    # witnessing-clauses: 436
  - property: async-regression-surface-property
    from: path
    path: ./async-regression.constraints.md
    as: async_regression
    # witnessing-clauses: 405
  - property: platform-cfg-threaded-js-surface-property
    from: path
    path: ./platform-cfg-threaded-js.constraints.md
    as: platform_cfg_threaded_js
    # witnessing-clauses: 360
  - property: cli-surface-property
    from: path
    path: ./cli.constraints.md
    as: cli
    # witnessing-clauses: 294
  - property: sync-async-regression-surface-property
    from: path
    path: ./sync-async-regression.constraints.md
    as: sync_async_regression
    # witnessing-clauses: 272
  - property: integration-surface-property
    from: path
    path: ./integration.constraints.md
    as: integration
    # witnessing-clauses: 231
  - property: platform-cfg-async-regression-surface-property
    from: path
    path: ./platform-cfg-async-regression.constraints.md
    as: platform_cfg_async_regression
    # witnessing-clauses: 219
  - property: weak-ref-js-surface-property
    from: path
    path: ./weak-ref-js.constraints.md
    as: weak_ref_js
    # witnessing-clauses: 209
  - property: returns-error-success-errors-regression-surface-property
    from: path
    path: ./returns-error-success-errors-regression.constraints.md
    as: returns_error_success_errors_regression
    # witnessing-clauses: 182
  - property: platform-cfg-async-js-surface-property
    from: path
    path: ./platform-cfg-async-js.constraints.md
    as: platform_cfg_async_js
    # witnessing-clauses: 178
  - property: sync-async-js-surface-property
    from: path
    path: ./sync-async-js.constraints.md
    as: sync_async_js
    # witnessing-clauses: 131
  - property: threaded-js-surface-property
    from: path
    path: ./threaded-js.constraints.md
    as: threaded_js
    # witnessing-clauses: 121
  - property: bundler-surface-property
    from: path
    path: ./bundler.constraints.md
    as: bundler
    # witnessing-clauses: 108
  - property: sync-regression-surface-property
    from: path
    path: ./sync-regression.constraints.md
    as: sync_regression
    # witnessing-clauses: 101
  - property: sync-cli-surface-property
    from: path
    path: ./sync-cli.constraints.md
    as: sync_cli
    # witnessing-clauses: 85
  - property: constructor-handle-regression-surface-property
    from: path
    path: ./constructor-handle-regression.constraints.md
    as: constructor_handle_regression
    # witnessing-clauses: 79
  - property: sync-async-threaded-js-surface-property
    from: path
    path: ./sync-async-threaded-js.constraints.md
    as: sync_async_threaded_js
    # witnessing-clauses: 70
  - property: platform-cfg-async-constructor-handle-js-surface-property
    from: path
    path: ./platform-cfg-async-constructor-handle-js.constraints.md
    as: platform_cfg_async_constructor_handle_js
    # witnessing-clauses: 66
  - property: sync-js-surface-property
    from: path
    path: ./sync-js.constraints.md
    as: sync_js
    # witnessing-clauses: 64
  - property: internal-surface-property
    from: path
    path: ./internal.constraints.md
    as: internal
    # witnessing-clauses: 58
  - property: platform-cfg-regression-surface-property
    from: path
    path: ./platform-cfg-regression.constraints.md
    as: platform_cfg_regression
    # witnessing-clauses: 54
  - property: platform-cfg-constructor-handle-js-surface-property
    from: path
    path: ./platform-cfg-constructor-handle-js.constraints.md
    as: platform_cfg_constructor_handle_js
    # witnessing-clauses: 50
  - property: platform-cfg-sync-js-surface-property
    from: path
    path: ./platform-cfg-sync-js.constraints.md
    as: platform_cfg_sync_js
    # witnessing-clauses: 24
  - property: bake-surface-property
    from: path
    path: ./bake.constraints.md
    as: bake
    # witnessing-clauses: 23
  - property: async-cli-surface-property
    from: path
    path: ./async-cli.constraints.md
    as: async_cli
    # witnessing-clauses: 23
  - property: platform-cfg-sync-async-js-surface-property
    from: path
    path: ./platform-cfg-sync-async-js.constraints.md
    as: platform_cfg_sync_async_js
    # witnessing-clauses: 12
  - property: result-shape-js-surface-property
    from: path
    path: ./result-shape-js.constraints.md
    as: result_shape_js
    # witnessing-clauses: 3
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

The Bun runtime contract is composed of 32 surface modules drafted from the test corpus. Per [Doc 704 §3](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error), target-language derivation operates over this composition; the constraint set is the durable artifact and target-language implementations are ephemeral cache.

Top surfaces by witnessing-clause count:

- **@regression** — 1828 clauses
- **constructor+handle/@js** — 1645 clauses
- **@js** — 959 clauses
- **platform-cfg/@js** — 847 clauses
- **async/@js** — 436 clauses
- **async/@regression** — 405 clauses
- **platform-cfg/threaded/@js** — 360 clauses
- **@cli** — 294 clauses
- **sync+async/@regression** — 272 clauses
- **@integration** — 231 clauses
- **platform-cfg/async/@regression** — 219 clauses
- **weak-ref/@js** — 209 clauses
- **returns-error/success-errors/@regression** — 182 clauses
- **platform-cfg/async/@js** — 178 clauses
- **sync+async/@js** — 131 clauses
- **threaded/@js** — 121 clauses
- **@bundler** — 108 clauses
- **sync/@regression** — 101 clauses
- **sync/@cli** — 85 clauses
- **constructor+handle/@regression** — 79 clauses
