//! Constants pool for compiled modules. Per design spec §III.

#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    BigInt(String),
    String(String),
    Regex { body: String, flags: String },
    /// Nested function prototype. Holds its own CompiledModule shape.
    Function(Box<crate::compiler::FunctionProto>),
}

#[derive(Debug, Default, Clone)]
pub struct ConstantsPool {
    entries: Vec<Constant>,
}

impl ConstantsPool {
    pub fn new() -> Self { Self::default() }

    /// Intern a constant. Equal constants return the same index. Numbers
    /// compare bit-for-bit (NaN bit-patterns are distinguished — caller
    /// should handle if needed).
    pub fn intern(&mut self, c: Constant) -> u16 {
        if let Some(idx) = self.entries.iter().position(|existing| same_constant(existing, &c)) {
            return idx as u16;
        }
        let idx = self.entries.len();
        assert!(idx < u16::MAX as usize, "constants pool overflow");
        self.entries.push(c);
        idx as u16
    }

    pub fn get(&self, idx: u16) -> Option<&Constant> {
        self.entries.get(idx as usize)
    }

    pub fn entries(&self) -> &[Constant] { &self.entries }

    pub fn len(&self) -> usize { self.entries.len() }
}

fn same_constant(a: &Constant, b: &Constant) -> bool {
    match (a, b) {
        (Constant::Number(x), Constant::Number(y)) => x.to_bits() == y.to_bits(),
        (Constant::BigInt(x), Constant::BigInt(y)) => x == y,
        (Constant::String(x), Constant::String(y)) => x == y,
        (Constant::Regex { body: b1, flags: f1 }, Constant::Regex { body: b2, flags: f2 }) =>
            b1 == b2 && f1 == f2,
        // Functions are unique per declaration site; never deduplicated.
        (Constant::Function(_), Constant::Function(_)) => false,
        _ => false,
    }
}
