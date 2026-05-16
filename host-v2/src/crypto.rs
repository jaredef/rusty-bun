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
// Tier-Ω.5.ddddd: real cryptographic hash digests, hand-rolled. SHA-256
// per FIPS 180-4; MD5 per RFC 1321; SHA-1 per RFC 3174. These are the
// three the npm corpus consumes — object-hash, jsonwebtoken signature
// verification, scrypt, etc. all reach for at least one.
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,
        0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = data.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 { padded.push(0); }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut hh) = (h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let mj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(mj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0]=h[0].wrapping_add(a); h[1]=h[1].wrapping_add(b); h[2]=h[2].wrapping_add(c); h[3]=h[3].wrapping_add(d);
        h[4]=h[4].wrapping_add(e); h[5]=h[5].wrapping_add(f); h[6]=h[6].wrapping_add(g); h[7]=h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for i in 0..8 { out[i*4..i*4+4].copy_from_slice(&h[i].to_be_bytes()); }
    out
}

fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301,0xEFCDAB89,0x98BADCFE,0x10325476,0xC3D2E1F0];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = data.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 { padded.push(0); }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        for i in 16..80 { w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1); }
        let (mut a,mut b,mut c,mut d,mut e) = (h[0],h[1],h[2],h[3],h[4]);
        for i in 0..80 {
            let (f, k) = if i < 20 { ((b & c) | (!b & d), 0x5A827999) }
                else if i < 40 { (b ^ c ^ d, 0x6ED9EBA1) }
                else if i < 60 { ((b & c) | (b & d) | (c & d), 0x8F1BBCDC) }
                else { (b ^ c ^ d, 0xCA62C1D6) };
            let t = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e = d; d = c; c = b.rotate_left(30); b = a; a = t;
        }
        h[0]=h[0].wrapping_add(a); h[1]=h[1].wrapping_add(b); h[2]=h[2].wrapping_add(c); h[3]=h[3].wrapping_add(d); h[4]=h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for i in 0..5 { out[i*4..i*4+4].copy_from_slice(&h[i].to_be_bytes()); }
    out
}

fn md5(data: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7,12,17,22, 7,12,17,22, 7,12,17,22, 7,12,17,22,
        5, 9,14,20, 5, 9,14,20, 5, 9,14,20, 5, 9,14,20,
        4,11,16,23, 4,11,16,23, 4,11,16,23, 4,11,16,23,
        6,10,15,21, 6,10,15,21, 6,10,15,21, 6,10,15,21,
    ];
    const K: [u32; 64] = [
        0xd76aa478,0xe8c7b756,0x242070db,0xc1bdceee,0xf57c0faf,0x4787c62a,0xa8304613,0xfd469501,
        0x698098d8,0x8b44f7af,0xffff5bb1,0x895cd7be,0x6b901122,0xfd987193,0xa679438e,0x49b40821,
        0xf61e2562,0xc040b340,0x265e5a51,0xe9b6c7aa,0xd62f105d,0x02441453,0xd8a1e681,0xe7d3fbc8,
        0x21e1cde6,0xc33707d6,0xf4d50d87,0x455a14ed,0xa9e3e905,0xfcefa3f8,0x676f02d9,0x8d2a4c8a,
        0xfffa3942,0x8771f681,0x6d9d6122,0xfde5380c,0xa4beea44,0x4bdecfa9,0xf6bb4b60,0xbebfbc70,
        0x289b7ec6,0xeaa127fa,0xd4ef3085,0x04881d05,0xd9d4d039,0xe6db99e5,0x1fa27cf8,0xc4ac5665,
        0xf4292244,0x432aff97,0xab9423a7,0xfc93a039,0x655b59c3,0x8f0ccc92,0xffeff47d,0x85845dd1,
        0x6fa87e4f,0xfe2ce6e0,0xa3014314,0x4e0811a1,0xf7537e82,0xbd3af235,0x2ad7d2bb,0xeb86d391,
    ];
    let mut h = [0x67452301u32, 0xefcdab89, 0x98badcfe, 0x10325476];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = data.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 { padded.push(0); }
    padded.extend_from_slice(&bit_len.to_le_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        let (mut a,mut b,mut c,mut d) = (h[0],h[1],h[2],h[3]);
        for i in 0..64 {
            let (f, g) = if i < 16 { ((b & c) | (!b & d), i) }
                else if i < 32 { ((d & b) | (!d & c), (5*i + 1) % 16) }
                else if i < 48 { (b ^ c ^ d, (3*i + 5) % 16) }
                else { (c ^ (b | !d), (7*i) % 16) };
            let t = a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(m[g]);
            a = d; d = c; c = b; b = b.wrapping_add(t.rotate_left(S[i]));
        }
        h[0]=h[0].wrapping_add(a); h[1]=h[1].wrapping_add(b); h[2]=h[2].wrapping_add(c); h[3]=h[3].wrapping_add(d);
    }
    let mut out = [0u8; 16];
    for i in 0..4 { out[i*4..i*4+4].copy_from_slice(&h[i].to_le_bytes()); }
    out
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len()*2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

fn base64_encode(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i + 2 < bytes.len() {
        let b = ((bytes[i] as u32) << 16) | ((bytes[i+1] as u32) << 8) | (bytes[i+2] as u32);
        out.push(T[((b >> 18) & 0x3f) as usize] as char);
        out.push(T[((b >> 12) & 0x3f) as usize] as char);
        out.push(T[((b >> 6) & 0x3f) as usize] as char);
        out.push(T[(b & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let b = (bytes[i] as u32) << 16;
        out.push(T[((b >> 18) & 0x3f) as usize] as char);
        out.push(T[((b >> 12) & 0x3f) as usize] as char);
        out.push_str("==");
    } else if rem == 2 {
        let b = ((bytes[i] as u32) << 16) | ((bytes[i+1] as u32) << 8);
        out.push(T[((b >> 18) & 0x3f) as usize] as char);
        out.push(T[((b >> 12) & 0x3f) as usize] as char);
        out.push(T[((b >> 6) & 0x3f) as usize] as char);
        out.push('=');
    }
    out
}

fn digest_bytes(alg: &str, data: &[u8]) -> Vec<u8> {
    match alg {
        "sha256" | "SHA256" | "SHA-256" => sha256(data).to_vec(),
        "sha1" | "SHA1" | "SHA-1" => sha1(data).to_vec(),
        "md5" | "MD5" => md5(data).to_vec(),
        _ => sha256(data).to_vec(),
    }
}

fn extract_bytes(rt: &mut Runtime, v: &Value) -> Vec<u8> {
    match v {
        Value::String(s) => s.as_bytes().to_vec(),
        Value::Object(id) => {
            let len = match rt.object_get(*id, "length") {
                Value::Number(n) => n as usize,
                _ => 0,
            };
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                match rt.object_get(*id, &i.to_string()) {
                    Value::Number(n) => out.push(n as u8),
                    _ => {}
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

pub fn install(rt: &mut Runtime) {
    let crypto = new_object(rt);

    // Tier-Ω.5.ddddd: createHash with real digests. Buffer is held in a
    // RefCell on the object via __buf__ (a Vec captured via thread-local
    // would race across instances; use a sidechannel-keyed map).
    register_method(rt, crypto, "createHash", |rt, args| {
        let alg = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => "sha256".to_string(),
        };
        let hash = new_object(rt);
        set_constant(rt, hash, "algorithm", Value::String(Rc::new(alg)));
        // Store accumulator as an array on the object itself.
        let buf = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
        rt.object_set(buf, "length".into(), Value::Number(0.0));
        rt.object_set(hash, "__buf__".into(), Value::Object(buf));
        register_method(rt, hash, "update", |rt, args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
            let buf_id = match rt.object_get(this_id, "__buf__") { Value::Object(id) => id, _ => return Ok(rt.current_this()) };
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            let bytes = extract_bytes(rt, &v);
            let cur_len = match rt.object_get(buf_id, "length") { Value::Number(n) => n as usize, _ => 0 };
            for (i, b) in bytes.iter().enumerate() {
                rt.object_set(buf_id, (cur_len + i).to_string(), Value::Number(*b as f64));
            }
            rt.object_set(buf_id, "length".into(), Value::Number((cur_len + bytes.len()) as f64));
            Ok(rt.current_this())
        });
        register_method(rt, hash, "digest", |rt, args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
            let alg = match rt.object_get(this_id, "algorithm") { Value::String(s) => (*s).clone(), _ => "sha256".into() };
            let buf_id = match rt.object_get(this_id, "__buf__") { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
            let len = match rt.object_get(buf_id, "length") { Value::Number(n) => n as usize, _ => 0 };
            let mut bytes = Vec::with_capacity(len);
            for i in 0..len {
                if let Value::Number(n) = rt.object_get(buf_id, &i.to_string()) { bytes.push(n as u8); }
            }
            let d = digest_bytes(&alg, &bytes);
            let enc = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => "".to_string(),
            };
            match enc.as_str() {
                "hex" => Ok(Value::String(Rc::new(hex_encode(&d)))),
                "base64" => Ok(Value::String(Rc::new(base64_encode(&d)))),
                "" => {
                    // No encoding → return a Buffer-like.
                    let out = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
                    for (i, b) in d.iter().enumerate() {
                        rt.object_set(out, i.to_string(), Value::Number(*b as f64));
                    }
                    rt.object_set(out, "length".into(), Value::Number(d.len() as f64));
                    Ok(Value::Object(out))
                }
                _ => Ok(Value::String(Rc::new(hex_encode(&d)))),
            }
        });
        Ok(Value::Object(hash))
    });

    // HMAC per RFC 2104: H(K' XOR opad || H(K' XOR ipad || message))
    register_method(rt, crypto, "createHmac", |rt, args| {
        let alg = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => "sha256".to_string(),
        };
        let key_v = args.get(1).cloned().unwrap_or(Value::Undefined);
        let key_bytes = extract_bytes(rt, &key_v);
        let hmac = new_object(rt);
        set_constant(rt, hmac, "algorithm", Value::String(Rc::new(alg)));
        // Store key as Buffer-like.
        let key_obj = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
        for (i, b) in key_bytes.iter().enumerate() {
            rt.object_set(key_obj, i.to_string(), Value::Number(*b as f64));
        }
        rt.object_set(key_obj, "length".into(), Value::Number(key_bytes.len() as f64));
        rt.object_set(hmac, "__key__".into(), Value::Object(key_obj));
        let buf = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
        rt.object_set(buf, "length".into(), Value::Number(0.0));
        rt.object_set(hmac, "__buf__".into(), Value::Object(buf));
        register_method(rt, hmac, "update", |rt, args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
            let buf_id = match rt.object_get(this_id, "__buf__") { Value::Object(id) => id, _ => return Ok(rt.current_this()) };
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            let bytes = extract_bytes(rt, &v);
            let cur_len = match rt.object_get(buf_id, "length") { Value::Number(n) => n as usize, _ => 0 };
            for (i, b) in bytes.iter().enumerate() {
                rt.object_set(buf_id, (cur_len + i).to_string(), Value::Number(*b as f64));
            }
            rt.object_set(buf_id, "length".into(), Value::Number((cur_len + bytes.len()) as f64));
            Ok(rt.current_this())
        });
        register_method(rt, hmac, "digest", |rt, args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
            let alg = match rt.object_get(this_id, "algorithm") { Value::String(s) => (*s).clone(), _ => "sha256".into() };
            let key_id = match rt.object_get(this_id, "__key__") { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
            let buf_id = match rt.object_get(this_id, "__buf__") { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
            // Read key + message.
            let k_len = match rt.object_get(key_id, "length") { Value::Number(n) => n as usize, _ => 0 };
            let mut key: Vec<u8> = (0..k_len).map(|i| match rt.object_get(key_id, &i.to_string()) { Value::Number(n) => n as u8, _ => 0 }).collect();
            let m_len = match rt.object_get(buf_id, "length") { Value::Number(n) => n as usize, _ => 0 };
            let msg: Vec<u8> = (0..m_len).map(|i| match rt.object_get(buf_id, &i.to_string()) { Value::Number(n) => n as u8, _ => 0 }).collect();
            let block_size: usize = 64;
            if key.len() > block_size { key = digest_bytes(&alg, &key); }
            while key.len() < block_size { key.push(0); }
            let ipad: Vec<u8> = key.iter().map(|b| b ^ 0x36).collect();
            let opad: Vec<u8> = key.iter().map(|b| b ^ 0x5c).collect();
            let mut inner = ipad; inner.extend_from_slice(&msg);
            let inner_hash = digest_bytes(&alg, &inner);
            let mut outer = opad; outer.extend_from_slice(&inner_hash);
            let d = digest_bytes(&alg, &outer);
            let enc = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => "".to_string(),
            };
            match enc.as_str() {
                "hex" => Ok(Value::String(Rc::new(hex_encode(&d)))),
                "base64" => Ok(Value::String(Rc::new(base64_encode(&d)))),
                _ => {
                    let out = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
                    for (i, b) in d.iter().enumerate() {
                        rt.object_set(out, i.to_string(), Value::Number(*b as f64));
                    }
                    rt.object_set(out, "length".into(), Value::Number(d.len() as f64));
                    Ok(Value::Object(out))
                }
            }
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
