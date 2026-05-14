//! node:crypto intrinsic stub — Tier-Ω.5.r.
//!
//! Exposes enough shape that `import crypto from "node:crypto"` /
//! `require("node:crypto")` succeeds and the common probe surface
//! (`createHash`, `randomUUID`, `randomBytes`) returns objects of the
//! right shape. The methods that perform real crypto throw TypeError
//! when called; the goal is import-time success + non-empty key set,
//! not runtime correctness.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

pub fn install(rt: &mut Runtime) {
    let crypto = new_object(rt);

    // createHash(alg) → Hash-like object with .update(data) and .digest(enc).
    register_method(rt, crypto, "createHash", |rt, args| {
        let alg = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => "<unknown>".to_string(),
        };
        let hash = new_object(rt);
        set_constant(rt, hash, "algorithm", Value::String(Rc::new(alg)));
        // update returns the hash for chaining (Node convention).
        register_method(rt, hash, "update", |rt, _args| {
            // Read the receiver as `this` and return it for chaining.
            let recv = rt.current_this();
            Ok(recv)
        });
        register_method(rt, hash, "digest", |_rt, _args| {
            Err(RuntimeError::TypeError(
                "node:crypto Hash.digest: not yet implemented (Tier-Ω.5.r stub)".into(),
            ))
        });
        Ok(Value::Object(hash))
    });

    register_method(rt, crypto, "createHmac", |rt, args| {
        let alg = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => "<unknown>".to_string(),
        };
        let hmac = new_object(rt);
        set_constant(rt, hmac, "algorithm", Value::String(Rc::new(alg)));
        register_method(rt, hmac, "update", |rt, _args| Ok(rt.current_this()));
        register_method(rt, hmac, "digest", |_rt, _args| {
            Err(RuntimeError::TypeError(
                "node:crypto Hmac.digest: not yet implemented (Tier-Ω.5.r stub)".into(),
            ))
        });
        Ok(Value::Object(hmac))
    });

    register_method(rt, crypto, "randomBytes", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:crypto randomBytes: not yet implemented (Tier-Ω.5.r stub)".into(),
        ))
    });

    register_method(rt, crypto, "randomUUID", |_rt, _args| {
        // Placeholder UUID — enough to satisfy shape probes that string-
        // check the result. Not cryptographically usable.
        Ok(Value::String(Rc::new(
            "00000000-0000-0000-0000-000000000000".into(),
        )))
    });

    register_method(rt, crypto, "createCipheriv", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:crypto createCipheriv: not yet implemented (Tier-Ω.5.r stub)".into(),
        ))
    });
    register_method(rt, crypto, "createDecipheriv", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:crypto createDecipheriv: not yet implemented (Tier-Ω.5.r stub)".into(),
        ))
    });
    register_method(rt, crypto, "pbkdf2Sync", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:crypto pbkdf2Sync: not yet implemented (Tier-Ω.5.r stub)".into(),
        ))
    });

    // Default export points at the namespace itself for CJS-interop
    // round-trip honesty.
    set_constant(rt, crypto, "default", Value::Object(crypto));

    rt.globals.insert("crypto".into(), Value::Object(crypto));
}
