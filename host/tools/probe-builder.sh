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
