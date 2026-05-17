//! Tier-Ω.5.CCCCCCCC: signed BigInt substrate for the runtime.
//!
//! Per seed §A8.13 substrate-amortization: this round lands the
//! primitive (parse / format / add / sub / mul / cmp / neg / abs);
//! divmod and mod_pow follow when a closure round needs them.
//! Wrapped as Rc<JsBigInt> in Value::BigInt, replacing the v1
//! Rc<String> decimal representation that all arithmetic previously
//! coerced through f64.
//!
//! Limbs: little-endian Vec<u32>, sign-magnitude.

use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct JsBigInt {
    /// -1, 0, or +1. Zero magnitude must pair with sign 0.
    sign: i8,
    /// Little-endian base-2^32 limbs, trimmed of trailing zeros.
    /// `[0]` for zero (single zero limb kept for ergonomics).
    mag: Vec<u32>,
}

impl PartialEq for JsBigInt {
    fn eq(&self, other: &Self) -> bool {
        self.sign == other.sign && self.mag == other.mag
    }
}
impl Eq for JsBigInt {}

impl std::fmt::Display for JsBigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_decimal())
    }
}

impl JsBigInt {
    pub fn zero() -> Self { JsBigInt { sign: 0, mag: vec![0] } }
    pub fn one() -> Self { JsBigInt { sign: 1, mag: vec![1] } }
    pub fn neg_one() -> Self { JsBigInt { sign: -1, mag: vec![1] } }

    pub fn is_zero(&self) -> bool { self.sign == 0 }
    pub fn is_negative(&self) -> bool { self.sign < 0 }

    pub fn from_i64(v: i64) -> Self {
        if v == 0 { return Self::zero(); }
        let sign = if v < 0 { -1 } else { 1 };
        let u = if v == i64::MIN { (i64::MAX as u64) + 1 } else { v.unsigned_abs() };
        let lo = (u & 0xffff_ffff) as u32;
        let hi = (u >> 32) as u32;
        let mag = if hi == 0 { vec![lo] } else { vec![lo, hi] };
        JsBigInt { sign, mag }
    }

    pub fn from_u64(v: u64) -> Self {
        if v == 0 { return Self::zero(); }
        let lo = (v & 0xffff_ffff) as u32;
        let hi = (v >> 32) as u32;
        let mag = if hi == 0 { vec![lo] } else { vec![lo, hi] };
        JsBigInt { sign: 1, mag }
    }

    /// Parse a numeric string per ECMA-262 §7.1.13 StringToBigInt:
    /// accepts decimal, plus 0x/0o/0b prefixes with optional sign.
    /// Returns None on parse error.
    pub fn from_decimal(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() { return None; }
        let (sign_byte, rest) = match s.as_bytes()[0] {
            b'-' => (-1i8, &s[1..]),
            b'+' => (1i8, &s[1..]),
            _ => (1i8, s),
        };
        if rest.is_empty() { return None; }
        // Detect non-decimal prefix.
        let (radix, digits): (u32, &str) = if rest.len() >= 2 && rest.as_bytes()[0] == b'0' {
            match rest.as_bytes()[1] {
                b'x' | b'X' => (16, &rest[2..]),
                b'o' | b'O' => (8, &rest[2..]),
                b'b' | b'B' => (2, &rest[2..]),
                _ => (10, rest),
            }
        } else { (10, rest) };
        if digits.is_empty() { return None; }
        let mut mag = vec![0u32];
        if radix == 10 {
            let bytes = digits.as_bytes();
            if !bytes.iter().all(|b| b.is_ascii_digit()) { return None; }
            let mut i = 0;
            while i < bytes.len() {
                let take = (bytes.len() - i).min(9);
                let chunk: u32 = std::str::from_utf8(&bytes[i..i + take]).ok()?.parse().ok()?;
                let mul: u32 = 10u32.pow(take as u32);
                mag_mul_small(&mut mag, mul);
                mag_add_small(&mut mag, chunk);
                i += take;
            }
        } else {
            for c in digits.bytes() {
                let d = match (c, radix) {
                    (b'0'..=b'9', _) => (c - b'0') as u32,
                    (b'a'..=b'f', 16) => (c - b'a' + 10) as u32,
                    (b'A'..=b'F', 16) => (c - b'A' + 10) as u32,
                    _ => return None,
                };
                if d >= radix { return None; }
                mag_mul_small(&mut mag, radix);
                mag_add_small(&mut mag, d);
            }
        }
        mag_trim(&mut mag);
        let sign = if mag_is_zero(&mag) { 0 } else { sign_byte };
        Some(JsBigInt { sign, mag })
    }

    pub fn to_decimal(&self) -> String {
        if self.is_zero() { return "0".into(); }
        let mut limbs = self.mag.clone();
        let mut chunks: Vec<u32> = Vec::new();
        while !mag_is_zero(&limbs) {
            let rem = mag_div_small(&mut limbs, 1_000_000_000);
            chunks.push(rem);
        }
        let mut out = String::new();
        if self.sign < 0 { out.push('-'); }
        let last = chunks.pop().unwrap();
        out.push_str(&format!("{}", last));
        for c in chunks.iter().rev() {
            out.push_str(&format!("{:09}", c));
        }
        out
    }

    /// Format in the given radix (2..=36). Lowercase digits per ECMA §6.1.6.2.13.
    pub fn to_radix(&self, radix: u32) -> String {
        if radix == 10 { return self.to_decimal(); }
        assert!((2..=36).contains(&radix));
        if self.is_zero() { return "0".into(); }
        let mut limbs = self.mag.clone();
        let mut digits: Vec<u32> = Vec::new();
        while !mag_is_zero(&limbs) {
            let rem = mag_div_small(&mut limbs, radix);
            digits.push(rem);
        }
        let mut out = String::new();
        if self.sign < 0 { out.push('-'); }
        for d in digits.iter().rev() {
            out.push(std::char::from_digit(*d, radix).unwrap());
        }
        out
    }

    pub fn to_f64(&self) -> f64 {
        if self.is_zero() { return 0.0; }
        // Combine high limbs into an f64 (lossy for large magnitudes;
        // matches ECMA Number(bigint) lossy semantics).
        let mut v: f64 = 0.0;
        for &l in self.mag.iter().rev() {
            v = v * 4294967296.0 + (l as f64);
        }
        if self.sign < 0 { -v } else { v }
    }

    /// For mixed BigInt-Number relational comparison per ECMA §7.2.13.
    pub fn cmp_f64(&self, n: f64) -> Option<Ordering> {
        if n.is_nan() { return None; }
        if n.is_infinite() {
            return Some(if n > 0.0 { Ordering::Less } else { Ordering::Greater });
        }
        // Compare via to_f64 with a fallback when the BigInt is too
        // large to round-trip — convert n to a BigInt by truncation
        // and compare exactly. Inner loop pkgs care about correctness
        // at the i64 range; the fallback covers larger.
        let self_f = self.to_f64();
        if self_f.is_finite() && (n.abs() < 1e15) && (self_f.abs() < 1e15) {
            return self_f.partial_cmp(&n);
        }
        // Slow path: floor(n) as BigInt.
        if n == 0.0 { return Some(match self.sign { -1 => Ordering::Less, 0 => Ordering::Equal, _ => Ordering::Greater }); }
        let floor_n = n.trunc();
        if floor_n.abs() < (i64::MAX as f64) {
            let nb = JsBigInt::from_i64(floor_n as i64);
            let ord = self.cmp(&nb);
            // Compensate for non-integer n: if equal but n had fractional part, refine.
            if ord == Ordering::Equal && (n - floor_n) != 0.0 {
                return Some(if n > floor_n { Ordering::Less } else { Ordering::Greater });
            }
            return Some(ord);
        }
        self_f.partial_cmp(&n)
    }

    pub fn cmp(&self, other: &JsBigInt) -> Ordering {
        match self.sign.cmp(&other.sign) {
            Ordering::Equal => {}
            ord => return ord,
        }
        if self.sign == 0 { return Ordering::Equal; }
        let abs_ord = mag_cmp(&self.mag, &other.mag);
        if self.sign < 0 { abs_ord.reverse() } else { abs_ord }
    }

    pub fn neg(&self) -> JsBigInt {
        JsBigInt { sign: -self.sign, mag: self.mag.clone() }
    }

    pub fn add(&self, other: &JsBigInt) -> JsBigInt {
        if self.is_zero() { return other.clone(); }
        if other.is_zero() { return self.clone(); }
        if self.sign == other.sign {
            let mut mag = mag_add(&self.mag, &other.mag);
            mag_trim(&mut mag);
            JsBigInt { sign: self.sign, mag }
        } else {
            // Subtract smaller magnitude from larger.
            match mag_cmp(&self.mag, &other.mag) {
                Ordering::Greater => {
                    let mut mag = mag_sub(&self.mag, &other.mag);
                    mag_trim(&mut mag);
                    JsBigInt { sign: self.sign, mag }
                }
                Ordering::Less => {
                    let mut mag = mag_sub(&other.mag, &self.mag);
                    mag_trim(&mut mag);
                    JsBigInt { sign: other.sign, mag }
                }
                Ordering::Equal => JsBigInt::zero(),
            }
        }
    }

    pub fn sub(&self, other: &JsBigInt) -> JsBigInt {
        self.add(&other.neg())
    }

    pub fn mul(&self, other: &JsBigInt) -> JsBigInt {
        if self.is_zero() || other.is_zero() { return JsBigInt::zero(); }
        let mut mag = mag_mul(&self.mag, &other.mag);
        mag_trim(&mut mag);
        let sign = if self.sign == other.sign { 1 } else { -1 };
        JsBigInt { sign, mag }
    }

    /// (quotient, remainder) with truncation-toward-zero per ECMA
    /// §6.1.6.2.5/.6 BigInt::divide and BigInt::remainder semantics.
    pub fn divmod(&self, divisor: &JsBigInt) -> Option<(JsBigInt, JsBigInt)> {
        if divisor.is_zero() { return None; }
        if self.is_zero() { return Some((JsBigInt::zero(), JsBigInt::zero())); }
        let (q_mag, r_mag) = mag_divmod(&self.mag, &divisor.mag);
        let q_sign = if self.sign == divisor.sign { 1 } else { -1 };
        let q_is_zero = mag_is_zero(&q_mag);
        let r_is_zero = mag_is_zero(&r_mag);
        let q = JsBigInt {
            sign: if q_is_zero { 0 } else { q_sign },
            mag: if q_is_zero { vec![0] } else { q_mag },
        };
        // Remainder takes the sign of the dividend.
        let r = JsBigInt {
            sign: if r_is_zero { 0 } else { self.sign },
            mag: if r_is_zero { vec![0] } else { r_mag },
        };
        Some((q, r))
    }

    /// Left-shift by n bits. n must be non-negative and fit u32.
    /// Returns None otherwise (caller throws RangeError).
    pub fn shl(&self, n: &JsBigInt) -> Option<JsBigInt> {
        if n.is_negative() { return None; }
        if self.is_zero() { return Some(JsBigInt::zero()); }
        let nf = n.to_f64();
        if !nf.is_finite() || nf > (1u64 << 20) as f64 { return None; }
        let bits = nf as u32;
        let limb_shift = (bits / 32) as usize;
        let bit_shift = bits % 32;
        let mut out = vec![0u32; self.mag.len() + limb_shift + 1];
        for (i, &l) in self.mag.iter().enumerate() {
            let lo = (l as u64) << bit_shift;
            out[i + limb_shift] |= (lo & 0xffff_ffff) as u32;
            out[i + limb_shift + 1] |= (lo >> 32) as u32;
        }
        mag_trim(&mut out);
        Some(JsBigInt { sign: self.sign, mag: out })
    }

    /// Arithmetic right-shift by n bits. For positive BigInts this is
    /// divmod-by-2^n quotient; for negative it floors toward -infinity
    /// per JS semantics. n must be non-negative and reasonable.
    pub fn shr(&self, n: &JsBigInt) -> Option<JsBigInt> {
        if n.is_negative() { return None; }
        if self.is_zero() { return Some(JsBigInt::zero()); }
        let nf = n.to_f64();
        if !nf.is_finite() { return None; }
        let bits = nf as u64;
        if bits >= (self.mag.len() as u64) * 32 + 1 {
            return Some(if self.sign < 0 { JsBigInt::neg_one() } else { JsBigInt::zero() });
        }
        let bits = bits as u32;
        let limb_shift = (bits / 32) as usize;
        let bit_shift = bits % 32;
        let mut out: Vec<u32> = self.mag.iter().skip(limb_shift).copied().collect();
        if bit_shift > 0 {
            for i in 0..out.len() {
                let lo = out[i] >> bit_shift;
                let hi = out.get(i + 1).copied().unwrap_or(0) << (32 - bit_shift);
                out[i] = lo | hi;
            }
        }
        mag_trim(&mut out);
        if out.is_empty() || mag_is_zero(&out) {
            return Some(if self.sign < 0 { JsBigInt::neg_one() } else { JsBigInt::zero() });
        }
        // Floor toward -infinity for negative: if any low bit was truncated,
        // subtract 1 from the magnitude (which is +1 in floor terms).
        if self.sign < 0 {
            let mut truncated = false;
            for i in 0..limb_shift {
                if self.mag.get(i).copied().unwrap_or(0) != 0 { truncated = true; break; }
            }
            if !truncated && bit_shift > 0 && limb_shift < self.mag.len() {
                if self.mag[limb_shift] & ((1u32 << bit_shift) - 1) != 0 { truncated = true; }
            }
            if truncated {
                let bumped = mag_add(&out, &[1]);
                let mut b = bumped;
                mag_trim(&mut b);
                return Some(JsBigInt { sign: -1, mag: b });
            }
        }
        Some(JsBigInt { sign: self.sign, mag: out })
    }

    /// Bitwise AND. For two non-negative operands, simple per-limb AND.
    /// For sign-mixed or both-negative, uses two's-complement semantics
    /// per ECMA §6.1.6.2.20 BigInt::bitwiseAND.
    pub fn bit_and(&self, other: &JsBigInt) -> JsBigInt {
        if !self.is_negative() && !other.is_negative() {
            let n = self.mag.len().min(other.mag.len());
            let mut out: Vec<u32> = (0..n).map(|i| self.mag[i] & other.mag[i]).collect();
            mag_trim(&mut out);
            let sign = if mag_is_zero(&out) { 0 } else { 1 };
            return JsBigInt { sign, mag: if out.is_empty() { vec![0] } else { out } };
        }
        // Negative path: convert both to two's-complement over max-width,
        // AND, then convert back. Pilot scope: small magnitudes (snowflakes
        // are non-negative); broader negative-AND handling deferred.
        let max_limbs = self.mag.len().max(other.mag.len()) + 1;
        let a = mag_to_twos(self, max_limbs);
        let b = mag_to_twos(other, max_limbs);
        let r: Vec<u32> = (0..max_limbs).map(|i| a[i] & b[i]).collect();
        twos_to_bigint(r)
    }

    pub fn bit_or(&self, other: &JsBigInt) -> JsBigInt {
        if !self.is_negative() && !other.is_negative() {
            let n = self.mag.len().max(other.mag.len());
            let mut out: Vec<u32> = (0..n).map(|i|
                self.mag.get(i).copied().unwrap_or(0)
                    | other.mag.get(i).copied().unwrap_or(0)
            ).collect();
            mag_trim(&mut out);
            let sign = if mag_is_zero(&out) { 0 } else { 1 };
            return JsBigInt { sign, mag: out };
        }
        let max_limbs = self.mag.len().max(other.mag.len()) + 1;
        let a = mag_to_twos(self, max_limbs);
        let b = mag_to_twos(other, max_limbs);
        let r: Vec<u32> = (0..max_limbs).map(|i| a[i] | b[i]).collect();
        twos_to_bigint(r)
    }

    pub fn bit_xor(&self, other: &JsBigInt) -> JsBigInt {
        if !self.is_negative() && !other.is_negative() {
            let n = self.mag.len().max(other.mag.len());
            let mut out: Vec<u32> = (0..n).map(|i|
                self.mag.get(i).copied().unwrap_or(0)
                    ^ other.mag.get(i).copied().unwrap_or(0)
            ).collect();
            mag_trim(&mut out);
            let sign = if mag_is_zero(&out) { 0 } else { 1 };
            return JsBigInt { sign, mag: out };
        }
        let max_limbs = self.mag.len().max(other.mag.len()) + 1;
        let a = mag_to_twos(self, max_limbs);
        let b = mag_to_twos(other, max_limbs);
        let r: Vec<u32> = (0..max_limbs).map(|i| a[i] ^ b[i]).collect();
        twos_to_bigint(r)
    }

    pub fn bit_not(&self) -> JsBigInt {
        // ~x = -(x + 1) per two's-complement.
        self.add(&JsBigInt::one()).neg()
    }

    /// Non-negative integer exponent power. Returns None if exponent
    /// is negative (ECMA RangeError for BigInt ** -1).
    pub fn pow(&self, exp: &JsBigInt) -> Option<JsBigInt> {
        if exp.is_negative() { return None; }
        if exp.is_zero() { return Some(JsBigInt::one()); }
        if self.is_zero() { return Some(JsBigInt::zero()); }
        // Cap exponent at reasonable range to avoid hangs; real callers
        // exponentiate by small integers (typically <1024).
        let exp_u = exp.to_f64();
        if !exp_u.is_finite() || exp_u > (1u64 << 20) as f64 {
            return None;
        }
        let mut e = exp_u as u64;
        let mut base = self.clone();
        let mut result = JsBigInt::one();
        while e > 0 {
            if e & 1 == 1 { result = result.mul(&base); }
            e >>= 1;
            if e > 0 { base = base.mul(&base); }
        }
        Some(result)
    }
}

/// Two's-complement representation of a signed bigint in `n_limbs` u32 limbs.
fn mag_to_twos(x: &JsBigInt, n_limbs: usize) -> Vec<u32> {
    let mut out = vec![0u32; n_limbs];
    if x.sign >= 0 {
        for (i, &l) in x.mag.iter().enumerate() {
            if i < n_limbs { out[i] = l; }
        }
    } else {
        // -x: invert magnitude bits and add 1
        for i in 0..n_limbs {
            out[i] = !x.mag.get(i).copied().unwrap_or(0);
        }
        let mut carry: u64 = 1;
        for limb in out.iter_mut() {
            let s = (*limb as u64) + carry;
            *limb = (s & 0xffff_ffff) as u32;
            carry = s >> 32;
            if carry == 0 { break; }
        }
    }
    out
}

fn twos_to_bigint(r: Vec<u32>) -> JsBigInt {
    // High-bit of top limb determines sign.
    let top = *r.last().unwrap_or(&0);
    let is_neg = (top & 0x8000_0000) != 0;
    if !is_neg {
        let mut m = r;
        mag_trim(&mut m);
        let sign = if mag_is_zero(&m) { 0 } else { 1 };
        return JsBigInt { sign, mag: m };
    }
    // Negate two's-complement: invert + 1
    let n = r.len();
    let mut inv = vec![0u32; n];
    for i in 0..n { inv[i] = !r[i]; }
    let mut carry: u64 = 1;
    for limb in inv.iter_mut() {
        let s = (*limb as u64) + carry;
        *limb = (s & 0xffff_ffff) as u32;
        carry = s >> 32;
        if carry == 0 { break; }
    }
    mag_trim(&mut inv);
    JsBigInt { sign: -1, mag: inv }
}

// ───────────────────── Unsigned magnitude helpers ────────────────

fn mag_is_zero(m: &[u32]) -> bool {
    m.iter().all(|&l| l == 0)
}

fn mag_trim(m: &mut Vec<u32>) {
    while m.len() > 1 && *m.last().unwrap() == 0 { m.pop(); }
}

fn mag_cmp(a: &[u32], b: &[u32]) -> Ordering {
    let la = a.iter().rposition(|&l| l != 0).map(|i| i + 1).unwrap_or(0);
    let lb = b.iter().rposition(|&l| l != 0).map(|i| i + 1).unwrap_or(0);
    match la.cmp(&lb) {
        Ordering::Equal => {}
        ord => return ord,
    }
    for i in (0..la).rev() {
        match a[i].cmp(&b[i]) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    Ordering::Equal
}

fn mag_add(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len().max(b.len()) + 1;
    let mut out = vec![0u32; n];
    let mut carry: u64 = 0;
    for i in 0..n {
        let x = a.get(i).copied().unwrap_or(0) as u64;
        let y = b.get(i).copied().unwrap_or(0) as u64;
        let s = x + y + carry;
        out[i] = (s & 0xffff_ffff) as u32;
        carry = s >> 32;
    }
    out
}

/// a >= b required.
fn mag_sub(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len();
    let mut out = vec![0u32; n];
    let mut borrow: i64 = 0;
    for i in 0..n {
        let x = a[i] as i64;
        let y = b.get(i).copied().unwrap_or(0) as i64;
        let d = x - y - borrow;
        if d < 0 { out[i] = (d + (1i64 << 32)) as u32; borrow = 1; }
        else { out[i] = d as u32; borrow = 0; }
    }
    out
}

fn mag_mul(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len() + b.len();
    let mut acc = vec![0u64; n];
    for i in 0..a.len() {
        for j in 0..b.len() {
            let p = (a[i] as u64) * (b[j] as u64);
            acc[i + j] += p & 0xffff_ffff;
            acc[i + j + 1] += p >> 32;
        }
    }
    let mut out = vec![0u32; n + 1];
    let mut carry: u64 = 0;
    for i in 0..n {
        let s = acc[i] + carry;
        out[i] = (s & 0xffff_ffff) as u32;
        carry = s >> 32;
    }
    out[n] = carry as u32;
    out
}

fn mag_mul_small(m: &mut Vec<u32>, k: u32) {
    let mut carry: u64 = 0;
    for limb in m.iter_mut() {
        let p = (*limb as u64) * (k as u64) + carry;
        *limb = (p & 0xffff_ffff) as u32;
        carry = p >> 32;
    }
    if carry != 0 { m.push(carry as u32); }
}

fn mag_add_small(m: &mut Vec<u32>, k: u32) {
    let mut carry: u64 = k as u64;
    for limb in m.iter_mut() {
        let s = (*limb as u64) + carry;
        *limb = (s & 0xffff_ffff) as u32;
        carry = s >> 32;
        if carry == 0 { break; }
    }
    if carry != 0 { m.push(carry as u32); }
}

/// Divide magnitude by a small u32; returns remainder, mutates m.
fn mag_div_small(m: &mut Vec<u32>, k: u32) -> u32 {
    let mut rem: u64 = 0;
    for i in (0..m.len()).rev() {
        let cur = (rem << 32) | (m[i] as u64);
        m[i] = (cur / (k as u64)) as u32;
        rem = cur % (k as u64);
    }
    mag_trim(m);
    rem as u32
}

fn mag_shl1(m: &[u32]) -> Vec<u32> {
    let mut out = vec![0u32; m.len() + 1];
    let mut carry: u32 = 0;
    for (i, &l) in m.iter().enumerate() {
        out[i] = (l << 1) | carry;
        carry = l >> 31;
    }
    out[m.len()] = carry;
    out
}

fn mag_bit_len(m: &[u32]) -> usize {
    for i in (0..m.len()).rev() {
        if m[i] != 0 {
            return i * 32 + (32 - m[i].leading_zeros() as usize);
        }
    }
    0
}

fn mag_bit(m: &[u32], i: usize) -> bool {
    let limb = i / 32;
    let bit = i % 32;
    m.get(limb).copied().unwrap_or(0) & (1u32 << bit) != 0
}

/// Binary long-division. Adequate for substrate; not constant-time.
fn mag_divmod(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
    let bits = mag_bit_len(a);
    let mut q = vec![0u32; (bits + 31) / 32 + 1];
    let mut r = vec![0u32];
    for i in (0..bits).rev() {
        r = mag_shl1(&r);
        if mag_bit(a, i) {
            if r.is_empty() { r.push(0); }
            r[0] |= 1;
        }
        if mag_cmp(&r, b) != Ordering::Less {
            r = mag_sub(&r, b);
            q[i / 32] |= 1u32 << (i % 32);
        }
    }
    mag_trim(&mut q);
    mag_trim(&mut r);
    (q, r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_format_roundtrip() {
        for s in &["0", "1", "-1", "42", "-42", "1000000000",
                    "18446744073709551615", "-18446744073709551616",
                    "123456789012345678901234567890"] {
            let b = JsBigInt::from_decimal(s).unwrap();
            assert_eq!(b.to_decimal(), *s);
        }
    }

    #[test]
    fn add_sub_signs() {
        let a = JsBigInt::from_decimal("100").unwrap();
        let b = JsBigInt::from_decimal("-30").unwrap();
        assert_eq!(a.add(&b).to_decimal(), "70");
        assert_eq!(a.sub(&b).to_decimal(), "130");
        assert_eq!(b.sub(&a).to_decimal(), "-130");
        assert_eq!(a.add(&a.neg()).to_decimal(), "0");
    }

    #[test]
    fn mul_signs() {
        let a = JsBigInt::from_decimal("12345678901234567890").unwrap();
        let b = JsBigInt::from_decimal("-2").unwrap();
        assert_eq!(a.mul(&b).to_decimal(), "-24691357802469135780");
    }

    #[test]
    fn divmod_trunc_toward_zero() {
        let a = JsBigInt::from_decimal("-7").unwrap();
        let b = JsBigInt::from_decimal("2").unwrap();
        let (q, r) = a.divmod(&b).unwrap();
        // -7 / 2 = -3 (trunc), remainder -1
        assert_eq!(q.to_decimal(), "-3");
        assert_eq!(r.to_decimal(), "-1");
    }

    #[test]
    fn pow_small() {
        let a = JsBigInt::from_decimal("2").unwrap();
        let e = JsBigInt::from_decimal("64").unwrap();
        assert_eq!(a.pow(&e).unwrap().to_decimal(), "18446744073709551616");
    }
}
