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

        if has_default && named_count == 0 {
            // Tuple B: spread default's own enumerable string keys.
            if let Value::Object(def_id) = default_value {
                let pairs: Vec<(String, Value)> = rt.obj(def_id).properties
                    .iter()
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect();
                for (k, v) in pairs {
                    if k == "default" { continue; }
                    if rt.obj(ns).properties.contains_key(&k) { continue; }
                    rt.object_set(ns, k, v);
                }
            }
        }

        Ok(())
    })));
}
