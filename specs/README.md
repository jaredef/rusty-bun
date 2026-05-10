# Spec-source extracts

Manually-curated invariant extracts from external specifications, consumed by the `derive-constraints` pipeline alongside test-source corpora. Each `*.spec.md` file under this directory is parsed by the `extract::spec` extractor (see `derive-constraints/src/extract/spec.rs`) and emitted as `TestFile` entries with `language: spec` and `kind: SpecInvariant` clauses, which flow through cluster / invert / seams / couple identically to test-derived clauses.

## Why

Per the [TextEncoder pilot AUDIT](../pilots/textencoder/AUDIT.md), most surface elements at the WHATWG-spec'd interface boundary are *gap-A* — the test corpus has no positive operational witness. The auto-emitted constraint document is a *floor* on what the surface guarantees; the spec is the *ceiling*. Both are required as derivation inputs.

## Format

```markdown
# <Surface name> — <spec authority>

[surface] <subject head>           <!-- optional: default subject for un-attributable clauses -->
[spec] <spec URL>                  <!-- recorded for human readers -->

## <invariant group name>
- <clause text — leading identifier path becomes the subject>
- <another clause>

## <next group>
- <clause>
```

- Each `## ` heading opens a new invariant group (modeled as a `TestCase`).
- Each `- ` bullet is a single invariant clause (modeled as a `ConstraintClause` with `kind: SpecInvariant`).
- The clause's subject is heuristically extracted: longest leading identifier path that starts with a capital letter or matches `[surface]`. For prose-heavy clauses, falls back to the document's `[surface]` annotation.
- Spec verb-class is heuristically derived from the clause text (`returns` → equivalence; `throws` → error; `is defined`/`exists` → existence; `contains`/`includes` → containment; etc.). Unrecognized → generic-assertion.

## Pipeline integration

```bash
derive-constraints pipeline \
    --test-corpus /path/to/tests \
    --impl-source /path/to/impl \
    --baseline   /path/to/baseline-crates \
    --specs      ./specs \
    --out        ./runs/<run-id>
```

The `--specs` flag is optional; when provided, spec extracts are scanned alongside the test corpus and merged into a single `scan.json`. Cluster-phase canonicalization treats spec clauses as additional witnesses for the same `(subject, verb_class)` properties — when test corpus and spec extract both witness the same property, the antichain representatives include both source kinds.

## Cross-corroboration

When a property has antichain representatives from BOTH a test source AND a spec source, the property is *cross-corroborated*. Cross-corroborated properties are stronger constraints than test-only or spec-only ones because the spec normatively says "must" and the test source empirically says "does." Look for these in the cluster.json output as the first-tier candidates for any rederive pilot.

## Current corpus

| File | Surface | Authority | Clauses |
|---|---|---|---|
| `text-encoder.spec.md` | TextEncoder | WHATWG Encoding §9 | 18 |
| `text-decoder.spec.md` | TextDecoder | WHATWG Encoding §10 | 25 |
| `url-search-params.spec.md` | URLSearchParams | WHATWG URL §5.2 | 29 |

Add new extracts in this directory; they'll be picked up automatically by any `pipeline --specs ./specs` invocation.
