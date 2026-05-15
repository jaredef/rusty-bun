#!/usr/bin/env bash
# Pin-Art probe-builder. Each shape-feature is named; a probe is an
# explicit combination of named features. When a real package failure
# resists isolated repro, the builder enumerates combinations until one
# triggers the same fault tag.
#
# Doc 723 corollary (2026-05-15): trace fidelity requires Pin-Art
# structure at every layer that participates in the trace — including
# the probe-construction apparatus itself.
#
# Usage:
#   probe-builder.sh emit <feature-set> <out-file>
#   probe-builder.sh run  <feature-set>
#   probe-builder.sh bisect <expected-fault-substring>
#
# Feature set is a comma-separated list of feature names. Each feature
# is a small JS fragment with a stable role in the emitted file.

set -u
RB=/media/jaredef/T7/rusty-bun-target/debug/rusty-bun-host-v2
OUT_DIR=${OUT_DIR:-/tmp/rb-probes}
mkdir -p "$OUT_DIR"

# ───────────────────── named features ─────────────────────
# Each feature contributes one or more text fragments to a probe file.
# Stable roles: "header" (top of file), "module_const", "class_pre"
# (class body before methods), "class_methods", "class_post" (after
# class), "runner" (the call-into code at bottom).

declare -A FEATURE_HEADER
declare -A FEATURE_MODULE_CONST
declare -A FEATURE_CLASS_FIELDS
declare -A FEATURE_CLASS_METHODS
declare -A FEATURE_CLASS_POST
declare -A FEATURE_RUNNER

# F01: module-level const arrow with default param
FEATURE_MODULE_CONST[mod_const_arrow_default]='const helper = (p, opts = {}) => "h:" + p;'
# F02: module-level const arrow without default param
FEATURE_MODULE_CONST[mod_const_arrow_plain]='const helper = (p) => "h:" + p;'
# F03: module-level function decl
FEATURE_MODULE_CONST[mod_function_decl]='function helper(p) { return "h:" + p; }'

# F11: 0 class fields
FEATURE_CLASS_FIELDS[fields_0]=''
# F12: 3 class fields (no init)
FEATURE_CLASS_FIELDS[fields_3]='    a;
    b;
    c;'
# F13: 18 class fields (matching minimatch's count)
FEATURE_CLASS_FIELDS[fields_18]='    f1; f2; f3; f4; f5; f6; f7; f8; f9;
    f10; f11; f12; f13; f14; f15; f16; f17; f18;'

# F21: simple constructor calling this.make()
FEATURE_CLASS_METHODS[ctor_calls_make]='    constructor(p) { this.p = p; this.make(); }
    make() { this.v = this.helper_method(); }
    helper_method() { return helper(this.p); }'
# F22: constructor with many this.* assignments before this.make()
FEATURE_CLASS_METHODS[ctor_heavy_then_make]='    constructor(p, opts = {}) {
        this.p = p;
        this.opts = opts;
        this.a = opts.a || 0;
        this.b = opts.b || "";
        this.c = !!opts.c;
        this.d = opts.d || null;
        this.e = opts.e || [];
        this.f = opts.f ?? 0;
        this.make();
    }
    make() { this.v = this.helper_method(); }
    helper_method() { return helper(this.p); }'
# F23: constructor calling this.make(); make calls this.helper_method
# which calls module-level helper with TWO args (matching minimatch's
# braceExpand(this.pattern, this.options) shape)
FEATURE_CLASS_METHODS[helper_two_args]='    constructor(p, opts = {}) { this.p = p; this.opts = opts; this.make(); }
    make() { this.v = this.helper_method(); }
    helper_method() { return helper(this.p, this.opts); }'

# F31: export the class via `export class`
FEATURE_CLASS_POST[export_class]='__EXPORT_CLASS__'
# F32: declare class then export via named statement
FEATURE_CLASS_POST[export_named]='__EXPORT_NAMED__'

# F41: also assign properties on a top-level arrow AFTER class declaration
# (matching minimatch.AST = AST; minimatch.Minimatch = Minimatch pattern)
FEATURE_CLASS_POST[assign_to_arrow_after]='const sibling = (p, pattern) => { return p === pattern; };
sibling.C = C;
sibling.helper = helper;
export { sibling };'

# F51: simple runner — instantiate class and read field
FEATURE_RUNNER[run_construct]='const inst = new C("x");
console.log("v:", inst.v);'
# F52: runner via TOP-LEVEL ARROW that itself constructs the class
# (matches minimatch's `export const minimatch = (...) => new
# Minimatch(...)` pattern — calling INTO an arrow that internally
# constructs the class).
FEATURE_RUNNER[run_via_arrow]='const inst = entry("x", "pat");
console.log("v:", inst);'

# F61: pre-class module-level arrow that references the class
# (matching minimatch's `export const makeRe = (p, opts = {}) =>
# new Minimatch(p, opts).makeRe();` BEFORE class Minimatch declared).
FEATURE_MODULE_CONST[mod_arrow_refs_class]='const helper = (p, opts = {}) => "h:" + p;
const entry = (p, pattern, opts = {}) => new C(p, opts).v;'
# F62: also includes property writes onto the arrow AFTER class
# (minimatch.AST = AST style)
FEATURE_CLASS_POST[arrow_prop_writes_after_class]='entry.theClass = C;
entry.helper = helper;
export { entry };'
# F63: TWO module-level arrows where the second references the class
FEATURE_MODULE_CONST[two_mod_arrows]='const helper = (p, opts = {}) => "h:" + p + JSON.stringify(opts);
const sibling = (p, opts = {}) => helper(p, opts);
const entry = (p, pattern, opts = {}) => new C(p, opts).v;'

# F71: class method that uses Set + spread (matching minimatch's
# `[...new Set(this.braceExpand())]`)
FEATURE_CLASS_METHODS[helper_set_spread]='    constructor(p, opts = {}) { this.p = p; this.opts = opts; this.make(); }
    make() { this.globSet = [...new Set(this.helper_method())]; this.v = this.globSet[0]; }
    helper_method() { return [helper(this.p, this.opts)]; }'

# F81: class with TEN methods (broader surface, more upvalues per class
# build — closer to minimatch which has many)
FEATURE_CLASS_METHODS[ctor_ten_methods]='    constructor(p, opts = {}) { this.p = p; this.opts = opts; this.make(); }
    make() { this.v = this.helper_method(); }
    helper_method() { return helper(this.p, this.opts); }
    m3() { return helper("m3", {}); }
    m4() { return helper("m4", {}); }
    m5() { return helper("m5", {}); }
    m6() { return helper("m6", {}); }
    m7() { return helper("m7", {}); }
    m8() { return helper("m8", {}); }
    m9() { return helper("m9", {}); }
    m10() { return helper("m10", {}); }'

# F91: class with field initializers (`= value`) not just declarations
FEATURE_CLASS_FIELDS[fields_with_init]='    a = null;
    b = 0;
    c = "";
    d = [];
    e = {};
    constructor_only_marker;'

# F92: nullish coalescing in field defaults
FEATURE_CLASS_FIELDS[fields_nullish]='    options = null;
    set = [];
    pattern = "";'

# ───────────────────── emitter ─────────────────────
emit_probe() {
    local features="$1"
    local out="$2"
    local has_mod_const=""
    local fields=""
    local methods=""
    local class_post="__EXPORT_CLASS__"
    local runner=""

    IFS=',' read -ra FS <<< "$features"
    for f in "${FS[@]}"; do
        if [ -n "${FEATURE_MODULE_CONST[$f]:-}" ]; then has_mod_const="${FEATURE_MODULE_CONST[$f]}"; fi
        if [ -n "${FEATURE_CLASS_FIELDS[$f]:-}" ] || [ "$f" = "fields_0" ]; then fields="${FEATURE_CLASS_FIELDS[$f]:-}"; fi
        if [ -n "${FEATURE_CLASS_METHODS[$f]:-}" ]; then methods="${FEATURE_CLASS_METHODS[$f]}"; fi
        if [ -n "${FEATURE_CLASS_POST[$f]:-}" ]; then class_post="${FEATURE_CLASS_POST[$f]}"; fi
        if [ -n "${FEATURE_RUNNER[$f]:-}" ]; then runner="${FEATURE_RUNNER[$f]}"; fi
    done

    {
        # The module-level const must be declared BEFORE the class so the
        # class-method body's upvalue capture can find it. This matches
        # minimatch's source ordering (braceExpand at line 143, class at 179).
        [ -n "$has_mod_const" ] && echo "$has_mod_const"
        echo ""
        case "$class_post" in
            __EXPORT_CLASS__) echo "export class C {" ;;
            *)                echo "class C {" ;;
        esac
        [ -n "$fields" ] && echo "$fields"
        [ -n "$methods" ] && echo "$methods"
        echo "}"
        case "$class_post" in
            __EXPORT_NAMED__) echo "export { C };" ;;
            __EXPORT_CLASS__) ;;
            *)                echo "$class_post" ;;
        esac
        echo ""
        [ -n "$runner" ] && echo "$runner"
    } > "$out"
}

# ───────────────────── runner ─────────────────────
run_probe() {
    local out="$1"
    timeout 5 "$RB" "$out" 2>&1
}

# ───────────────────── bisect ─────────────────────
bisect_for_fault() {
    local fault_pattern="$1"
    echo "=== bisect: looking for probe matching '$fault_pattern' ==="
    # Curated feature sets, ordered roughly from simple to complex.
    local feature_sets=(
        "mod_const_arrow_plain,fields_0,ctor_calls_make,run_construct"
        "mod_const_arrow_default,fields_0,ctor_calls_make,run_construct"
        "mod_const_arrow_default,fields_3,ctor_calls_make,run_construct"
        "mod_const_arrow_default,fields_18,ctor_calls_make,run_construct"
        "mod_const_arrow_default,fields_0,helper_two_args,run_construct"
        "mod_const_arrow_default,fields_3,helper_two_args,run_construct"
        "mod_const_arrow_default,fields_18,helper_two_args,run_construct"
        "mod_const_arrow_default,fields_0,ctor_heavy_then_make,run_construct"
        "mod_const_arrow_default,fields_3,ctor_heavy_then_make,run_construct"
        "mod_const_arrow_default,fields_18,ctor_heavy_then_make,run_construct"
        "mod_const_arrow_default,fields_18,helper_two_args,assign_to_arrow_after,run_construct"
        # Newer named features (post-first-bisect)
        "mod_arrow_refs_class,fields_0,helper_two_args,run_via_arrow"
        "mod_arrow_refs_class,fields_18,helper_two_args,run_via_arrow"
        "mod_arrow_refs_class,fields_18,helper_set_spread,run_via_arrow"
        "mod_arrow_refs_class,fields_18,helper_set_spread,arrow_prop_writes_after_class,run_via_arrow"
        "two_mod_arrows,fields_18,helper_two_args,run_via_arrow"
        "two_mod_arrows,fields_18,helper_set_spread,run_via_arrow"
        "two_mod_arrows,fields_18,helper_set_spread,arrow_prop_writes_after_class,run_via_arrow"
        "two_mod_arrows,fields_with_init,helper_set_spread,arrow_prop_writes_after_class,run_via_arrow"
        "two_mod_arrows,fields_18,ctor_ten_methods,arrow_prop_writes_after_class,run_via_arrow"
        "two_mod_arrows,fields_nullish,ctor_ten_methods,arrow_prop_writes_after_class,run_via_arrow"
    )
    for fs in "${feature_sets[@]}"; do
        local out="$OUT_DIR/$(echo "$fs" | tr ',' '_').mjs"
        emit_probe "$fs" "$out"
        local result=$(run_probe "$out")
        local last=$(echo "$result" | tail -1)
        if echo "$result" | grep -q "$fault_pattern"; then
            echo "MATCH  $fs"
            echo "       file=$out"
            echo "       fault=$last"
            return 0
        else
            echo "ok     $fs :: $last"
        fi
    done
    echo "=== NO MATCH FOUND ==="
    return 1
}

# ───────────────────── dispatch ─────────────────────
cmd="${1:-help}"
case "$cmd" in
    emit)
        emit_probe "$2" "$3"
        echo "wrote $3"
        ;;
    run)
        out="$OUT_DIR/$(echo "$2" | tr ',' '_').mjs"
        emit_probe "$2" "$out"
        run_probe "$out"
        ;;
    bisect)
        bisect_for_fault "$2"
        ;;
    *)
        echo "usage: $0 {emit|run|bisect} ..."
        exit 1
        ;;
esac
