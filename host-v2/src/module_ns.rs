//! Ω.5.P16.E2.ns-default-synth — HostFinalizeModuleNamespace closure for
//! Doc 717 Tuple A/B.
//!
//! Tuple A: ESM module exports named bindings but no `default`. Synthesize
//! `default` whose value is the namespace object itself. Matches the de-facto
//! shape used by `import x from "lodash/foo"`-style CJS-shimmed packages
//! whose default-import callers expect the whole namespace.
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

        if !has_default {
            // Tuple A: default = namespace itself.
            rt.object_set(ns, "default".to_string(), Value::Object(ns));
            return Ok(());
        }

        if named_count == 0 {
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
