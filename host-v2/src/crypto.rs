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

    // Tier-Ω.5.hhh: real (non-secure) entropy. Replaces the throw-stub
    // randomBytes and the zeros-stub randomUUID. Source is a wall-clock-
    // seeded xorshift state shared per Runtime via a thread-local. Not
    // cryptographically secure (Pin-Art deferred), but produces valid
    // varying bytes so ulid / nanoid / uuid generate distinct values.
    register_method(rt, crypto, "randomBytes", |rt, args| {
        let n = match args.first() {
            Some(Value::Number(n)) => *n as usize,
            _ => 0,
        };
        let mut bytes_vec = Vec::with_capacity(n);
        for _ in 0..n { bytes_vec.push(next_random_byte()); }
        // Return a Uint8Array-like object so .length / indexing work.
        let arr_obj = rusty_js_runtime::value::Object::new_array();
        let id = rt.alloc_object(arr_obj);
        for (i, b) in bytes_vec.iter().enumerate() {
            rt.object_set(id, i.to_string(), Value::Number(*b as f64));
        }
        rt.object_set(id, "length".into(), Value::Number(n as f64));
        Ok(Value::Object(id))
    });

    register_method(rt, crypto, "randomUUID", |_rt, _args| {
        // v4 UUID: 16 random bytes with version/variant bits set.
        let mut b = [0u8; 16];
        for i in 0..16 { b[i] = next_random_byte(); }
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        let s = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
        );
        Ok(Value::String(Rc::new(s)))
    });

    // Tier-Ω.5.hhh: crypto.webcrypto subobject — getRandomValues fills
    // the input typed-array in place per Web Crypto API. nanoid imports
    // `{ webcrypto as crypto }` and calls `crypto.getRandomValues(pool)`.
    let webcrypto = new_object(rt);
    register_method(rt, webcrypto, "getRandomValues", |rt, args| {
        let id = match args.first() {
            Some(Value::Object(id)) => *id,
            _ => return Err(RuntimeError::TypeError("crypto.getRandomValues: argument must be a typed array".into())),
        };
        let length = match rt.object_get(id, &"length".to_string()) {
            Value::Number(n) => n as usize,
            _ => 0,
        };
        for i in 0..length {
            rt.object_set(id, i.to_string(), Value::Number(next_random_byte() as f64));
        }
        Ok(Value::Object(id))
    });
    register_method(rt, webcrypto, "randomUUID", |_rt, _args| {
        let mut b = [0u8; 16];
        for i in 0..16 { b[i] = next_random_byte(); }
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        let s = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
        );
        Ok(Value::String(Rc::new(s)))
    });
    rt.object_set(crypto, "webcrypto".into(), Value::Object(webcrypto));
    // Tier-Ω.5.hhh: also expose getRandomValues directly on the crypto
    // namespace so `globalThis.crypto.getRandomValues` works (web style).
    // ulid + many browser-targeting libs read it this way.
    register_method(rt, crypto, "getRandomValues", |rt, args| {
        let id = match args.first() {
            Some(Value::Object(id)) => *id,
            _ => return Err(RuntimeError::TypeError("crypto.getRandomValues: argument must be a typed array".into())),
        };
        let length = match rt.object_get(id, &"length".to_string()) {
            Value::Number(n) => n as usize,
            _ => 0,
        };
        for i in 0..length {
            rt.object_set(id, i.to_string(), Value::Number(next_random_byte() as f64));
        }
        Ok(Value::Object(id))
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

// Tier-Ω.5.hhh: xorshift64 PRNG seeded once from wall-clock + thread id.
// Thread-local for cheap mutation. Not cryptographically secure (Pin-Art
// deferred), but produces valid varying bytes — sufficient for ulid /
// nanoid / uuid to generate distinct identifiers.
thread_local! {
    static PRNG_STATE: std::cell::Cell<u64> = std::cell::Cell::new(seed_prng());
}
fn seed_prng() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0xdeadbeef_cafebabe);
    t ^ 0x9E3779B97F4A7C15
}
fn next_random_byte() -> u8 {
    PRNG_STATE.with(|cell| {
        let mut x = cell.get();
        if x == 0 { x = seed_prng(); }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        cell.set(x);
        (x & 0xff) as u8
    })
}
