# cluster — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: cluster-surface-property
  threshold: CLUS1
  interface: [cluster.disconnect, cluster.setupPrimary, cluster.fork, cluster.isMaster, cluster.isPrimary, cluster.isWorker, cluster.on]

@imports: []

@pins: []

Surface drawn from 7 candidate properties across the Bun test corpus. Construction-style: 7; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 15.

## CLUS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**cluster.disconnect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/cluster_test.ts:10` — [node/cluster] has all node exports → `assertEquals(typeof cluster.disconnect, "function")`
- `tests/unit_node/cluster_test.ts:17` — [node/cluster] has all node exports → `assertEquals(typeof cluster.disconnect, "function")`
- `tests/unit_node/cluster_test.ts:32` — [node/cluster] has all node exports → `assertEquals(cluster.disconnect, clusterNamed.disconnect)`

## CLUS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**cluster.setupPrimary** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/cluster_test.ts:18` — [node/cluster] has all node exports → `assertEquals(typeof cluster.setupPrimary, "function")`
- `tests/unit_node/cluster_test.ts:19` — [node/cluster] has all node exports → `assertEquals(cluster.setupPrimary, cluster.setupMaster)`
- `tests/unit_node/cluster_test.ts:22` — [node/cluster] has all node exports → `assertEquals(cluster.setupPrimary, clusterNamed.setupPrimary)`

## CLUS3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**cluster.fork** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/cluster_test.ts:16` — [node/cluster] has all node exports → `assertEquals(typeof cluster.fork, "function")`
- `tests/unit_node/cluster_test.ts:30` — [node/cluster] has all node exports → `assertEquals(cluster.fork, clusterNamed.fork)`

## CLUS4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**cluster.isMaster** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/cluster_test.ts:8` — [node/cluster] has all node exports → `assertEquals(cluster.isMaster, true)`
- `tests/unit_node/cluster_test.ts:42` — [node/cluster] has all node exports → `assertEquals(cluster.isMaster, clusterNamed.isMaster)`

## CLUS5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**cluster.isPrimary** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/cluster_test.ts:7` — [node/cluster] has all node exports → `assertEquals(cluster.isPrimary, true)`
- `tests/unit_node/cluster_test.ts:40` — [node/cluster] has all node exports → `assertEquals(cluster.isPrimary, clusterNamed.isPrimary)`

## CLUS6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**cluster.isWorker** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/cluster_test.ts:9` — [node/cluster] has all node exports → `assertEquals(cluster.isWorker, false)`
- `tests/unit_node/cluster_test.ts:38` — [node/cluster] has all node exports → `assertEquals(cluster.isWorker, clusterNamed.isWorker)`

## CLUS7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**cluster.on** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/cluster_test.ts:11` — [node/cluster] has all node exports → `assertEquals(typeof cluster.on, "function")`

