//! Module evaluation tests per Doc 717 §IX + design spec §VI–§VII.
//!
//! Verifies that:
//! - evaluate_module returns a Module Namespace Object
//! - Named exports populate the namespace
//! - HostFinalizeModuleNamespace runs between Link and Evaluate
//! - The Tuple A closure (default = namespace) works through the hook

use rusty_js_runtime::{HostHook, Runtime, Value};

#[test]
fn evaluate_empty_module() {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    let ns = rt.evaluate_module("", "test").expect("evaluate failed");
    let obj = ns.borrow();
    assert!(obj.properties.is_empty());
}

#[test]
fn named_export_populates_namespace() {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    let src = r#"
        const greeting = 'hello';
        export { greeting };
    "#;
    let ns = rt.evaluate_module(src, "test").expect("evaluate failed");
    let obj = ns.borrow();
    if let Some(d) = obj.properties.get("greeting") {
        if let Value::String(s) = &d.value {
            assert_eq!(s.as_str(), "hello");
        } else { panic!("expected string, got {:?}", d.value); }
    } else { panic!("no `greeting` in namespace"); }
}

#[test]
fn multiple_named_exports() {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    let src = r#"
        const a = 1;
        const b = 2;
        const c = 3;
        export { a, b, c };
    "#;
    let ns = rt.evaluate_module(src, "test").expect("evaluate failed");
    let obj = ns.borrow();
    for (name, expected) in &[("a", 1.0), ("b", 2.0), ("c", 3.0)] {
        match obj.properties.get(*name) {
            Some(d) => {
                if let Value::Number(n) = &d.value { assert_eq!(*n, *expected); }
                else { panic!("{} not a number", name); }
            }
            None => panic!("{} missing from namespace", name),
        }
    }
}

#[test]
fn rename_export() {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    let src = r#"
        const internal = 42;
        export { internal as exposed };
    "#;
    let ns = rt.evaluate_module(src, "test").expect("evaluate failed");
    let obj = ns.borrow();
    assert!(obj.properties.contains_key("exposed"));
    assert!(!obj.properties.contains_key("internal"));
}

// ─────────── Doc 717 Tuple-A closure via host hook ───────────

#[test]
fn host_hook_synthesizes_default_as_namespace() {
    // The Tuple A closure (Doc 717): a module without `export default` gets
    // a synthetic `default` key pointing to the namespace itself.
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.install_host_hook(HostHook::FinalizeModuleNamespace(Box::new(|_rt, _ast, ns| {
        let has_default = ns.borrow().properties.contains_key("default");
        if !has_default {
            // Synthesize default = namespace-as-object. For v1 we attach an
            // ordinary marker; in a fuller implementation this would be
            // the namespace object itself (a self-reference).
            ns.borrow_mut().set_own("default".into(), rusty_js_runtime::Value::String(
                std::rc::Rc::new("<synthesized-default>".to_string())
            ));
        }
        Ok(())
    })));
    let src = "const x = 1; export { x };";
    let ns = rt.evaluate_module(src, "test").expect("evaluate failed");
    let obj = ns.borrow();
    assert!(obj.properties.contains_key("x"));
    assert!(obj.properties.contains_key("default"),
        "Tuple-A closure: default synthesized by host hook");
}

#[test]
fn host_hook_does_not_run_without_install() {
    // Verify default isn't magically present without the hook.
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    let src = "const x = 1; export { x };";
    let ns = rt.evaluate_module(src, "test").expect("evaluate failed");
    let obj = ns.borrow();
    assert!(obj.properties.contains_key("x"));
    assert!(!obj.properties.contains_key("default"));
}

// ─────────── Doc 717 Tuple-B closure via host hook ───────────

#[test]
fn host_hook_synthesizes_named_exports_from_default() {
    // Tuple B (Doc 717): when default is set to an object with own
    // properties, host hook synthesizes named exports from those props.
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    // In this v1 test, we manually inject a "default" key with a known
    // object and verify the hook spreads its properties onto the namespace.
    rt.install_host_hook(HostHook::FinalizeModuleNamespace(Box::new(|_rt, _ast, ns| {
        // For demo: if there's a known sentinel binding `__default_obj_props`,
        // pretend it's the default and spread.
        let keys_to_spread: Vec<(String, rusty_js_runtime::Value)> = {
            let b = ns.borrow();
            if let Some(d) = b.properties.get("__default_obj_props") {
                if let rusty_js_runtime::Value::Object(o) = &d.value {
                    o.borrow().properties.iter()
                        .map(|(k, dd)| (k.clone(), dd.value.clone()))
                        .collect()
                } else { Vec::new() }
            } else { Vec::new() }
        };
        let mut ns_mut = ns.borrow_mut();
        for (k, v) in keys_to_spread {
            if !ns_mut.properties.contains_key(&k) {
                ns_mut.set_own(k, v);
            }
        }
        Ok(())
    })));
    let src = r#"
        const __default_obj_props = { Ls: 1, en: 2, extend: 3 };
        export { __default_obj_props };
    "#;
    let ns = rt.evaluate_module(src, "test").expect("evaluate failed");
    let obj = ns.borrow();
    // After the hook, Ls / en / extend should be top-level exports.
    assert!(obj.properties.contains_key("Ls"),
        "Tuple-B: 'Ls' spread from default's own props by host hook");
    assert!(obj.properties.contains_key("en"));
    assert!(obj.properties.contains_key("extend"));
}
