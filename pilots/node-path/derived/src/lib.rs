// node-path pilot — Node's `path` module derivation.
//
// Inputs:
//   AUDIT — pilots/node-path/AUDIT.md
//   SPEC  — Node.js docs §path (https://nodejs.org/api/path.html)
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/path.constraints.md
//           (21 properties, 375 clauses)
//
// Tier 2 ecosystem-compat surface per Doc 707. Bun's tests serve as the
// authoritative spec since Node has no formal IDL/RFC.
//
// Pilot exposes posix and win32 namespaces. Top-level path::* delegates to
// posix::* by convention (Node's API behaves identically on Unix; Bun
// follows this).

pub mod posix;
pub mod win32;

pub use posix::PathParsed;

// Top-level convenience re-exports — match Node's default-namespace shape.
pub fn basename(path: &str, ext: Option<&str>) -> String { posix::basename(path, ext) }
pub fn dirname(path: &str) -> String { posix::dirname(path) }
pub fn extname(path: &str) -> String { posix::extname(path) }
pub fn parse(path: &str) -> PathParsed { posix::parse(path) }
pub fn format(parsed: &PathParsed) -> String { posix::format(parsed) }
pub fn is_absolute(path: &str) -> bool { posix::is_absolute(path) }
pub fn join(parts: &[&str]) -> String { posix::join(parts) }
pub fn normalize(path: &str) -> String { posix::normalize(path) }
pub fn relative(from: &str, to: &str) -> String { posix::relative(from, to) }
pub fn resolve(parts: &[&str], cwd: &str) -> String { posix::resolve(parts, cwd) }
pub const SEP: &str = posix::SEP;
pub const DELIMITER: &str = posix::DELIMITER;
