//! Phase 3 — invert. Reads a `ClusterReport` and emits `.constraints.md`
//! documents in the [rederive grammar](https://github.com/jaredef/rederive)
//! per surface (Bun, fs, fetch, URL, …). The output is consumable directly
//! by rederive's parse stage; together they realize the formalization-then-
//! derivation pipeline articulated in [Doc 704](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error)
//! and [`docs/invert-phase-design.md`](../../../docs/invert-phase-design.md).
//!
//! The MVP emits draft prose stitched from antichain representatives. Per
//! the design's honest scope: the substrate at rederive's derive step
//! ultimately interprets the prose into target-language code, and the
//! prose may need keeper-side editing before it derives well. Invert is
//! a draft-author for the keeper, not a replacement for keeper authorship.

use crate::cluster::{ClusterReport, Property, RepresentativeConstraint, VerbClass};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct InvertReport {
    pub output_dir: PathBuf,
    pub surfaces_emitted: u32,
    pub constraints_emitted: u32,
    pub properties_skipped: u32,
}

const MIN_BEHAVIORAL_CARDINALITY: u64 = 5;
const MAX_CONSTRAINTS_PER_SURFACE: usize = 80;

/// Emit one `.constraints.md` per architectural surface, plus a top-level
/// `bun-runtime.constraints.md` index that imports each surface module.
pub fn invert(report: &ClusterReport, out_dir: &Path) -> Result<InvertReport> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("create output dir {}", out_dir.display()))?;

    let mut by_surface: BTreeMap<String, Vec<&Property>> = BTreeMap::new();
    let mut skipped: u32 = 0;
    for prop in &report.properties {
        if !is_emittable(prop) {
            skipped += 1;
            continue;
        }
        let surface = surface_of(&prop.subject);
        by_surface.entry(surface).or_default().push(prop);
    }

    // Surface-level filter — three cuts. (1) Drop surfaces whose name
    // is a lowercase-first identifier not in the known-namespace
    // allowlist; these are local variables (`ctx`, `res`, `err`, etc.)
    // whose property accesses (`ctx.foo`, `err.message`) survived the
    // subject-level filter despite being incidental. (2) Drop surfaces
    // whose only contents are a handful of low-cardinality behavioral
    // properties — typically the long tail. A surface is emittable if
    // it contains any construction-style property, has total witnessing
    // >= 20 clauses, or has at least one property at cardinality >= 10.
    by_surface.retain(|surface, props| {
        if !is_emittable_surface_name(surface) {
            return false;
        }
        let any_cs = props.iter().any(|p| p.construction_style);
        let total: u64 = props.iter().map(|p| p.constraints_in).sum();
        let max_card = props.iter().map(|p| p.constraints_in).max().unwrap_or(0);
        any_cs || total >= 20 || max_card >= 10
    });

    let mut surfaces_emitted = 0u32;
    let mut constraints_emitted = 0u32;
    let mut surface_order: Vec<(String, u64)> = Vec::new();

    for (surface, mut props) in by_surface {
        // Sort within a surface: construction-style first, then by
        // cardinality descending. Cap to MAX_CONSTRAINTS_PER_SURFACE so a
        // single surface document remains readable.
        props.sort_by(|a, b| {
            b.construction_style
                .cmp(&a.construction_style)
                .then(b.constraints_in.cmp(&a.constraints_in))
                .then(a.subject.cmp(&b.subject))
        });
        if props.len() > MAX_CONSTRAINTS_PER_SURFACE {
            props.truncate(MAX_CONSTRAINTS_PER_SURFACE);
        }
        let total_witness = props.iter().map(|p| p.constraints_in).sum::<u64>();
        let path = out_dir.join(format!("{}.constraints.md", filename_safe(&surface)));
        let content = render_surface(&surface, &props);
        std::fs::write(&path, content)
            .with_context(|| format!("write {}", path.display()))?;
        surfaces_emitted += 1;
        constraints_emitted += props.len() as u32;
        surface_order.push((surface, total_witness));
    }

    // Top-level index. Lists each surface module ordered by total witness
    // count; the index itself does not declare a single induced property
    // (the runtime is composed of multiple), it imports each surface as
    // a separately-derivable module.
    surface_order.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let index_path = out_dir.join("bun-runtime.constraints.md");
    let index = render_index(&surface_order);
    std::fs::write(&index_path, index)
        .with_context(|| format!("write {}", index_path.display()))?;

    Ok(InvertReport {
        output_dir: out_dir.to_path_buf(),
        surfaces_emitted,
        constraints_emitted,
        properties_skipped: skipped,
    })
}

/// A property is emittable if it has substantive architectural content:
/// either it's classified construction-style, or it has cardinality at
/// or above the behavioral floor and is not an obvious noise subject
/// (`<anonymous>`, test-framework calls, single-letter locals).
fn is_emittable(prop: &Property) -> bool {
    if !is_substantive_subject(&prop.subject) {
        return false;
    }
    if prop.construction_style {
        return true;
    }
    prop.constraints_in >= MIN_BEHAVIORAL_CARDINALITY
}

fn is_substantive_subject(subject: &str) -> bool {
    if subject == "<anonymous>" {
        return false;
    }
    // Test-framework noise: assertEqual / assertEquals / assert.* / ok /
    // expect (when used as a subject — i.e., constraints about the test
    // framework itself rather than the surface under test).
    let head = subject.split('.').next().unwrap_or("");
    if matches!(
        head,
        "assertEqual"
            | "assertEquals"
            | "assertEqualValues"
            | "assertEqualBuffers"
            | "assertNotEqual"
            | "assertThrows"
            | "ok"
            | "fail"
            | "deepEqual"
            | "strictEqual"
    ) {
        return false;
    }
    if head == "assert" && !subject.contains('.') {
        return false;
    }
    // Single-letter or short lowercase subjects with no namespace are
    // almost always test-local variables that escaped binding substitution.
    if subject.len() <= 2 && !subject.contains('.') {
        let bytes = subject.as_bytes();
        if !bytes.is_empty() && !(bytes[0] as char).is_ascii_uppercase() {
            return false;
        }
    }
    // Bare lowercase subjects (no namespace, lowercase-first) are almost
    // always test-local variables — `result`, `value`, `output`, `parsed`,
    // `obj`, `actual`, `expected`, `parsed`, etc. Keep them only if they
    // match a known global/namespace head.
    if !subject.contains('.') {
        let bytes = subject.as_bytes();
        let first_lower = !bytes.is_empty() && !(bytes[0] as char).is_ascii_uppercase();
        if first_lower {
            let known_lc = matches!(
                head,
                "fetch"
                    | "structuredClone"
                    | "queueMicrotask"
                    | "atob"
                    | "btoa"
                    | "performance"
                    | "console"
                    | "globalThis"
                    | "process"
                    | "require"
                    | "import"
                    | "fs"
                    | "path"
                    | "http"
                    | "https"
                    | "http2"
                    | "net"
                    | "tls"
                    | "dgram"
                    | "dns"
                    | "url"
                    | "querystring"
                    | "crypto"
                    | "zlib"
                    | "stream"
                    | "events"
                    | "util"
                    | "os"
                    | "cluster"
                    | "vm"
                    | "v8"
                    | "buffer"
                    | "module"
                    | "readline"
                    | "tty"
                    | "assert"
                    | "timers"
                    | "setTimeout"
                    | "setInterval"
                    | "setImmediate"
                    | "clearTimeout"
                    | "clearInterval"
                    | "clearImmediate"
            );
            if !known_lc {
                return false;
            }
        }
    }
    true
}

/// A surface name is emittable if it is uppercase-first (a typical class
/// or namespace name like `Bun`, `URL`, `Buffer`, `Response`) or appears
/// in the lowercase-namespace allowlist (`fs`, `path`, `http`, …). All
/// other lowercase-first names are local variables that escaped subject-
/// level filtering and should not become surface modules.
fn is_emittable_surface_name(surface: &str) -> bool {
    let bytes = surface.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    if (bytes[0] as char).is_ascii_uppercase() {
        return true;
    }
    matches!(
        surface,
        "fetch"
            | "structuredClone"
            | "queueMicrotask"
            | "atob"
            | "btoa"
            | "performance"
            | "console"
            | "globalThis"
            | "process"
            | "fs"
            | "path"
            | "http"
            | "https"
            | "http2"
            | "net"
            | "tls"
            | "dgram"
            | "dns"
            | "url"
            | "querystring"
            | "crypto"
            | "zlib"
            | "stream"
            | "events"
            | "util"
            | "os"
            | "cluster"
            | "vm"
            | "v8"
            | "buffer"
            | "module"
            | "readline"
            | "tty"
            | "assert"
            | "timers"
            | "setTimeout"
            | "setInterval"
            | "setImmediate"
            | "clearTimeout"
            | "clearInterval"
            | "clearImmediate"
    )
}

fn surface_of(subject: &str) -> String {
    subject
        .split('.')
        .next()
        .unwrap_or(subject)
        .to_string()
}

/// Sanitize a surface name into a portable filename: lowercase ASCII,
/// non-alphanumerics → `-`, no leading/trailing dashes.
fn filename_safe(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_dash = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "unnamed".to_string()
    } else {
        out
    }
}

fn render_surface(surface: &str, props: &[&Property]) -> String {
    let mut out = String::new();
    let safe = filename_safe(surface);
    let property_name = format!("{}-surface-property", safe);
    let interface = props
        .iter()
        .filter(|p| p.construction_style)
        .map(|p| p.subject.clone())
        .collect::<Vec<_>>();
    let interface_render = if interface.is_empty() {
        // Fall back to the top construction-or-not subjects.
        let top: Vec<String> = props.iter().take(8).map(|p| p.subject.clone()).collect();
        top.join(", ")
    } else {
        let truncated: Vec<String> = interface.into_iter().take(16).collect();
        truncated.join(", ")
    };

    out.push_str(&format!(
        "# {} — surface constraints derived from the Bun test corpus\n\n",
        surface
    ));
    out.push_str(&format!(
        "*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at {}/runs/2026-05-10-bun-derive-constraints. \
This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). \
The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. \
See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*\n\n",
        "rusty-bun"
    ));

    let threshold_id = if let Some(p) = props.first() {
        property_id(&safe, 1, &p.subject)
    } else {
        format!("{}1", surface_id_prefix(&safe))
    };

    out.push_str(&format!("@provides: {}\n", property_name));
    out.push_str(&format!("  threshold: {}\n", threshold_id));
    out.push_str(&format!("  interface: [{}]\n\n", interface_render));

    out.push_str("@imports: []\n\n");
    out.push_str("@pins: []\n\n");

    out.push_str(&format!(
        "Surface drawn from {} candidate properties across the Bun test corpus. \
Construction-style: {}; behavioral (high-cardinality): {}. \
Total witnessing constraint clauses: {}.\n\n",
        props.len(),
        props.iter().filter(|p| p.construction_style).count(),
        props.iter().filter(|p| !p.construction_style).count(),
        props.iter().map(|p| p.constraints_in).sum::<u64>()
    ));

    for (i, p) in props.iter().enumerate() {
        out.push_str(&render_constraint(&safe, i + 1, p));
        out.push_str("\n");
    }
    out
}

fn render_constraint(surface_safe: &str, idx: usize, p: &Property) -> String {
    let prefix = surface_id_prefix(surface_safe);
    let id = format!("{}{}", prefix, idx);
    let kind = constraint_type_of(p);
    let scope = if p.construction_style { "module" } else { "module" };
    let mut out = String::new();
    out.push_str(&format!("## {}\n", id));
    out.push_str(&format!("type: {}\n", kind));
    out.push_str("authority: derived\n");
    out.push_str(&format!("scope: {}\n", scope));
    out.push_str("status: active\n");
    out.push_str("depends-on: []\n\n");

    out.push_str(&format!(
        "**{}** — {} {}\n\n",
        p.subject,
        verb_narrative(p.verb_class),
        if p.construction_style {
            "(construction-style)".to_string()
        } else {
            format!("(behavioral; cardinality {})", p.constraints_in)
        }
    ));

    out.push_str(&format!(
        "Witnessed by {} constraint clauses across {} test files. Antichain representatives:\n\n",
        p.constraints_in,
        p.source_files.len()
    ));
    for r in &p.antichain {
        out.push_str(&render_representative(r));
    }

    out
}

fn render_representative(r: &RepresentativeConstraint) -> String {
    let raw = if r.raw.len() > 200 {
        format!("{}…", &r.raw[..200])
    } else {
        r.raw.clone()
    };
    format!(
        "- `{}:{}` — {} → `{}`\n",
        r.file,
        r.line,
        if r.test_name.len() > 90 {
            format!("{}…", &r.test_name[..90])
        } else {
            r.test_name.clone()
        },
        raw.replace('`', "'")
    )
}

fn verb_narrative(v: VerbClass) -> &'static str {
    match v {
        VerbClass::TypeInstance => "exposes values of the expected type or class.",
        VerbClass::Existence => "is defined and resolves to a non-nullish value at the documented call site.",
        VerbClass::Error => "throws or rejects with a documented error shape on invalid inputs.",
        VerbClass::Equivalence => "produces values matching the documented patterns under the documented inputs.",
        VerbClass::Containment => "satisfies the documented containment / structural-shape invariants.",
        VerbClass::Ordering => "satisfies the documented ordering / proximity invariants.",
        VerbClass::GenericAssertion => "satisfies the documented invariant.",
        VerbClass::Other => "exhibits the property captured in the witnessing test.",
    }
}

fn constraint_type_of(p: &Property) -> &'static str {
    match p.verb_class {
        VerbClass::TypeInstance | VerbClass::Existence => "specification",
        VerbClass::Error => "predicate",
        VerbClass::Equivalence => "predicate",
        VerbClass::Containment | VerbClass::Ordering => "invariant",
        VerbClass::GenericAssertion | VerbClass::Other => "predicate",
    }
}

fn property_id(surface_safe: &str, idx: usize, _subject: &str) -> String {
    format!("{}{}", surface_id_prefix(surface_safe), idx)
}

/// Produce an identifier prefix that fits rederive's H2 anchor format:
/// uppercase ASCII letters only (rederive's parser uses these as opaque
/// IDs but consistent shape aids readability).
fn surface_id_prefix(surface_safe: &str) -> String {
    let cleaned: String = surface_safe
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect();
    if cleaned.is_empty() {
        "X".to_string()
    } else {
        // First 4 letters uppercased — short enough to remain readable
        // when concatenated with a numeric suffix.
        cleaned
            .chars()
            .take(4)
            .map(|c| c.to_ascii_uppercase())
            .collect()
    }
}

fn render_index(surfaces: &[(String, u64)]) -> String {
    let mut out = String::new();
    out.push_str("# bun-runtime — root constraint set\n\n");
    out.push_str("*Index of surface constraint modules. Each `@imports` entry points at a per-surface document drafted by `derive-constraints invert`. The runtime's contract is the composition of the imported surface properties.*\n\n");
    out.push_str("@provides: bun-runtime-property\n");
    out.push_str("  threshold: COMPOSITE\n");
    out.push_str("  interface: []\n\n");

    out.push_str("@imports:\n");
    for (surface, witness) in surfaces.iter().take(64) {
        let safe = filename_safe(surface);
        out.push_str(&format!(
            "  - property: {}-surface-property\n    from: path\n    path: ./{}.constraints.md\n    as: {}\n",
            safe, safe, safe.replace('-', "_")
        ));
        out.push_str(&format!("    # witnessing-clauses: {}\n", witness));
    }
    out.push_str("\n@pins: []\n\n");

    out.push_str("## COMPOSITE\n");
    out.push_str("type: bridge\n");
    out.push_str("authority: derived\n");
    out.push_str("scope: system\n");
    out.push_str("status: active\n");
    out.push_str("depends-on: []\n\n");
    out.push_str(&format!(
        "The Bun runtime contract is composed of {} surface modules drafted from the test corpus. \
Per [Doc 704 §3](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error), \
target-language derivation operates over this composition; the constraint set is the durable artifact \
and target-language implementations are ephemeral cache.\n\n",
        surfaces.len()
    ));
    out.push_str(
        "Top surfaces by witnessing-clause count:\n\n",
    );
    for (surface, witness) in surfaces.iter().take(20) {
        out.push_str(&format!("- **{}** — {} clauses\n", surface, witness));
    }
    out
}
