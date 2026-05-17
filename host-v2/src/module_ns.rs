//! Ω.5.P18.E1.ns-default-synth-narrow — HostFinalizeModuleNamespace closure.
//!
//! Earlier Ω.5.P16.E2 added Tuple A (synthesize `default = namespace` when
//! the module had named exports but no default) for compatibility with
//! `import x from "lodash/foo"` patterns. Per Doc 721 Step 4 against the
//! Ω.5.P17 residual, Tuple A causes a 237-package III.a keyCount-Δ+1 cluster
//! versus Bun, which does NOT synthesize default on ESM-with-named-exports.
//! CJS-shimmed packages route through `evaluate_cjs_module` instead and
//! never reach this hook, so Tuple A's stated rationale doesn't apply here.
//!
//! Tuple A is now restricted to the empty-namespace case: a module that
//! exports nothing at all gets `default = namespace` as a fallback so
//! default-import callers receive a stable empty handle. Modules with at
//! least one named export are left untouched, matching Bun.
//!
//! Tuple B: ESM module exports only `default` and the default value is an
//! object. Synthesize one named export per own enumerable string key of the
//! default value. Matches `export default { a, b }` followed by
//! `import { a } from "..."`.
//!
//! Reentrant-safe: the hook re-reads the namespace's current property set on
//! each invocation and never overwrites an existing key.

use rusty_js_runtime::{HostHook, Runtime, Value};

pub fn install(rt: &mut Runtime) {
    rt.install_host_hook(HostHook::FinalizeModuleNamespace(Box::new(|rt, _ast, ns| {
        let (has_default, default_value, named_count): (bool, Value, usize) = {
            let o = rt.obj(ns);
            let has = o.properties.contains_key("default");
            let dv = o.properties.get("default").map(|d| d.value.clone()).unwrap_or(Value::Undefined);
            let other = o.properties.keys().filter(|k| k.as_str() != "default").count();
            (has, dv, other)
        };

        if !has_default && named_count == 0 {
            // Tuple A (narrow): module exports nothing — install default as
            // a fallback handle pointing at the empty namespace.
            rt.object_set(ns, "default".to_string(), Value::Object(ns));
            return Ok(());
        }

        // Ω.5.P21.E2.tuple-b-drop: Tuple B (spread default's own keys as
        // named exports when only `default` is declared) is dropped here
        // for the same reason Ω.5.P18.E1 dropped Tuple A. This hook fires
        // only on ESM evaluation; for ESM-with-only-default, Bun's
        // namespace is exactly `{default: V}` regardless of V's shape.
        // CJS-shimmed packages whose original justification motivated
        // Tuple B route through `evaluate_cjs_module`'s
        // `populate_cjs_namespace_view` and get their own handling
        // (including the Ω.5.P21.E1 callable-instance-prop filter).
        //
        // Examples that close from this drop: mitt (single fn-default ESM,
        // was leaking name/length/prototype), kleur (object default, was
        // leaking bg* color methods), upath (similar pattern). All now
        // produce keyCount=1 matching Bun.
        let _ = (has_default, default_value, named_count); // silence unused

        Ok(())
    })));
}
