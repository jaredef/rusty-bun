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

/// Ω.5.P43.E1: true if `url` is a `file://` URL ending in `.js` whose
/// enclosing package.json does not declare `"type":"module"`. That's the
/// "module field ESM" shape (.js loaded as ESM because of the package's
/// `module` field or `exports.module` condition, not because the package
/// is whole-tree ESM via `type:module`). Bun's namespace synth treats
/// this case as "needs default", matching the @opentelemetry/core /
/// @xstate/fsm / minified-rollup-TypeScript pattern.
fn is_js_under_non_type_module_package(url: &str) -> bool {
    let path_str = match url.strip_prefix("file://") { Some(p) => p, None => return false };
    let path = std::path::Path::new(path_str);
    if path.extension().and_then(|s| s.to_str()) != Some("js") { return false; }
    let mut cur = path.parent();
    while let Some(d) = cur {
        let candidate = d.join("package.json");
        if candidate.is_file() {
            if let Ok(text) = std::fs::read_to_string(&candidate) {
                // Cheap text scan for "type":"module" — full JSON parse
                // would pull a dep. False positives on commented-out
                // "type" keys are tolerable for v1.
                if text.contains("\"type\"") && text.contains("\"module\"") {
                    // Heuristic: if both tokens appear AND the package
                    // declares type:module, treat as pure-ESM (return
                    // false). Otherwise it's module-field-ESM (return
                    // true). We don't precisely parse — Bun's heuristic
                    // is similarly loose.
                    let lower = text.replace(char::is_whitespace, "");
                    if lower.contains("\"type\":\"module\"") {
                        return false;
                    }
                }
                return true;  // found package.json, no type:module → module-field-ESM
            }
        }
        cur = d.parent();
    }
    true  // no package.json anywhere → treat as module-field-ESM (rare edge)
}

pub fn install(rt: &mut Runtime) {
    rt.install_host_hook(HostHook::FinalizeModuleNamespace(Box::new(|rt, _ast, ns, url| {
        let (has_default, default_value, named_count): (bool, Value, usize) = {
            let o = rt.obj(ns);
            let has = o.properties.contains_key("default");
            let dv = o.properties.get("default").map(|d| d.value.clone()).unwrap_or(Value::Undefined);
            let other = o.properties.keys().filter(|k| k.as_str() != "default").count();
            (has, dv, other)
        };

        // Ω.5.P43.E1.tuple-a-by-url-shape: re-widen Tuple A for .js
        // modules loaded through a `module` field / `exports.module` /
        // module-field walk. Bun synthesizes `default = namespace` when
        // an ESM-shaped .js module under a non-type:module package is
        // loaded; we now do too. Pure ESM (type:module .js, .mjs files,
        // empty namespaces) continues to follow P18.E1's behavior:
        // empty → fallback default, otherwise no synth.
        //
        // The URL discriminator: .mjs always pure-ESM. .js needs to
        // check the parent package.json's `type` field; absence (or
        // anything other than "module") means we're in module-field
        // territory and Bun synthesizes default.
        let is_module_field_esm = is_js_under_non_type_module_package(url);

        if !has_default && named_count == 0 {
            // Tuple A (narrow): module exports nothing — install default
            // as a fallback handle pointing at the empty namespace.
            rt.object_set(ns, "default".to_string(), Value::Object(ns));
            return Ok(());
        }
        if !has_default && is_module_field_esm {
            // Tuple A (wide, P43.E1): module has named exports but no
            // explicit default; Bun synthesizes default = namespace for
            // the module-field-ESM case. Matches the @opentelemetry/core
            // / @xstate/fsm / many-TS-compiled-packages shape.
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
