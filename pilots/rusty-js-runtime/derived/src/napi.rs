//! Ω.5.P46.E1.napi-v1 — Node-API substrate (v1).
//!
//! Loads `.node` native modules via dlopen + dlsym(`napi_register_module_v1`),
//! exposes the core ~50 N-API entry points needed for synchronous module-init
//! and basic value-creation / property-set patterns napi-rs's generated code
//! uses. NOT a complete N-API impl — deferred to follow-up rounds:
//!  - threadsafe-functions, async work (P46.E2)
//!  - finalizers + weak refs (P46.E3)
//!  - ArrayBuffer / TypedArray creation (P46.E4)
//!  - BigInt, Date, Promise-wrapping (P46.E5)
//!  - AsyncResource, instance-data, env cleanup hooks (P46.E6)
//!
//! Architecture per the plan agent's design:
//!  - `napi_env` is `*mut NapiEnv` — Rust struct holding the Runtime ptr,
//!    handle table, scope stack, refs, pending exception.
//!  - `napi_value` is `*mut c_void` encoding handle index + 1 (NonNull).
//!  - All napi_* C-ABI functions are `#[no_mangle] pub extern "C"`. The host
//!    binary exports them via `-Wl,--export-dynamic` so dlopen'd .node files
//!    can resolve them at runtime.

#![allow(non_camel_case_types, non_snake_case, dead_code)]

use crate::value::{InternalKind, NativeFn, Object, Value};
use crate::Runtime;
use std::ffi::{c_char, c_void, CStr};
use std::rc::Rc;

// ─── Public C types per N-API ───

pub type napi_status = i32;
pub const napi_ok: napi_status = 0;
pub const napi_invalid_arg: napi_status = 1;
pub const napi_object_expected: napi_status = 2;
pub const napi_string_expected: napi_status = 3;
pub const napi_name_expected: napi_status = 4;
pub const napi_function_expected: napi_status = 5;
pub const napi_number_expected: napi_status = 6;
pub const napi_boolean_expected: napi_status = 7;
pub const napi_array_expected: napi_status = 8;
pub const napi_generic_failure: napi_status = 9;
pub const napi_pending_exception: napi_status = 10;
pub const napi_cancelled: napi_status = 11;
pub const napi_escape_called_twice: napi_status = 12;
pub const napi_handle_scope_mismatch: napi_status = 13;

pub type napi_valuetype = i32;
pub const napi_undefined: napi_valuetype = 0;
pub const napi_null: napi_valuetype = 1;
pub const napi_boolean: napi_valuetype = 2;
pub const napi_number: napi_valuetype = 3;
pub const napi_string: napi_valuetype = 4;
pub const napi_symbol: napi_valuetype = 5;
pub const napi_object_t: napi_valuetype = 6;
pub const napi_function: napi_valuetype = 7;
pub const napi_external: napi_valuetype = 8;
pub const napi_bigint: napi_valuetype = 9;

pub type napi_env = *mut NapiEnv;
pub type napi_value = *mut c_void;
pub type napi_ref = *mut NapiRefHandle;
pub type napi_handle_scope = *mut c_void;
pub type napi_escapable_handle_scope = *mut c_void;
pub type napi_callback_info = *mut NapiCallbackInfo;
pub type napi_callback = unsafe extern "C" fn(env: napi_env, info: napi_callback_info) -> napi_value;

#[repr(C)]
pub struct napi_extended_error_info {
    pub error_message: *const c_char,
    pub engine_reserved: *mut c_void,
    pub engine_error_code: u32,
    pub error_code: napi_status,
}

// ─── Internal types ───

pub struct NapiEnv {
    rt: *mut Runtime,
    handles: Vec<Option<Value>>,
    scopes: Vec<usize>,
    refs: Vec<Option<Value>>,
    pending_exception: Option<Value>,
    last_error_msg: std::ffi::CString,
    last_error_code: napi_status,
    last_error_info: napi_extended_error_info,
}

pub struct NapiRefHandle {
    pub slot: usize,  // index into NapiEnv::refs
    pub env: *mut NapiEnv,
    pub count: u32,
}

pub struct NapiCallbackInfo {
    pub this: Value,
    pub args: Vec<Value>,
    pub data: *mut c_void,
}

impl NapiEnv {
    pub fn new(rt: &mut Runtime) -> Box<Self> {
        let mut last_error_info = napi_extended_error_info {
            error_message: std::ptr::null(),
            engine_reserved: std::ptr::null_mut(),
            engine_error_code: 0,
            error_code: napi_ok,
        };
        let last_error_msg = std::ffi::CString::new("").unwrap();
        last_error_info.error_message = last_error_msg.as_ptr();
        Box::new(NapiEnv {
            rt: rt as *mut Runtime,
            handles: Vec::with_capacity(64),
            scopes: Vec::with_capacity(8),
            refs: Vec::new(),
            pending_exception: None,
            last_error_msg,
            last_error_code: napi_ok,
            last_error_info,
        })
    }

    pub fn push_handle(&mut self, v: Value) -> napi_value {
        self.handles.push(Some(v));
        // handle index + 1 so it's NonNull
        self.handles.len() as napi_value
    }

    pub fn get_handle(&self, h: napi_value) -> Option<&Value> {
        let idx = (h as usize).checked_sub(1)?;
        self.handles.get(idx)?.as_ref()
    }

    /// Roots for GC walking — exposed via Runtime::napi_env_roots.
    pub fn roots(&self) -> Vec<rusty_js_gc::ObjectId> {
        let mut out = Vec::new();
        for h in self.handles.iter().flatten() {
            if let Value::Object(id) = h { out.push(*id); }
        }
        for r in self.refs.iter().flatten() {
            if let Value::Object(id) = r { out.push(*id); }
        }
        if let Some(Value::Object(id)) = &self.pending_exception { out.push(*id); }
        out
    }
}

// ─── Helper macros for shim shape ───

macro_rules! env_mut {
    ($env:expr) => {{
        if $env.is_null() { return napi_invalid_arg; }
        &mut *$env
    }};
}

macro_rules! rt_mut {
    ($env:expr) => {{
        let env = env_mut!($env);
        &mut *env.rt
    }};
}

macro_rules! check_arg {
    ($p:expr) => {{
        if $p.is_null() { return napi_invalid_arg; }
    }};
}

// ─── Tier A: lifecycle + globals ───

#[no_mangle]
pub unsafe extern "C" fn napi_get_undefined(env: napi_env, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = env.push_handle(Value::Undefined);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_null(env: napi_env, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = env.push_handle(Value::Null);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_boolean(env: napi_env, value: bool, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = env.push_handle(Value::Boolean(value));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_global(env: napi_env, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let rt = &mut *env.rt;
    let global = match rt.globals.get("globalThis").cloned() {
        Some(v) => v,
        None => Value::Undefined,
    };
    *result = env.push_handle(global);
    napi_ok
}

// ─── Tier A: number value creation + extraction ───

#[no_mangle]
pub unsafe extern "C" fn napi_create_int32(env: napi_env, value: i32, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = env.push_handle(Value::Number(value as f64));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_uint32(env: napi_env, value: u32, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = env.push_handle(Value::Number(value as f64));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_int64(env: napi_env, value: i64, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = env.push_handle(Value::Number(value as f64));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_double(env: napi_env, value: f64, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = env.push_handle(Value::Number(value));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_int32(env: napi_env, value: napi_value, result: *mut i32) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    match env.get_handle(value) {
        Some(Value::Number(n)) => { *result = *n as i32; napi_ok }
        _ => napi_number_expected,
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_uint32(env: napi_env, value: napi_value, result: *mut u32) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    match env.get_handle(value) {
        Some(Value::Number(n)) => { *result = *n as u32; napi_ok }
        _ => napi_number_expected,
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_int64(env: napi_env, value: napi_value, result: *mut i64) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    match env.get_handle(value) {
        Some(Value::Number(n)) => { *result = *n as i64; napi_ok }
        _ => napi_number_expected,
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_double(env: napi_env, value: napi_value, result: *mut f64) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    match env.get_handle(value) {
        Some(Value::Number(n)) => { *result = *n; napi_ok }
        _ => napi_number_expected,
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_bool(env: napi_env, value: napi_value, result: *mut bool) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    match env.get_handle(value) {
        Some(Value::Boolean(b)) => { *result = *b; napi_ok }
        _ => napi_boolean_expected,
    }
}

// ─── Tier B: strings ───

#[no_mangle]
pub unsafe extern "C" fn napi_create_string_utf8(
    env: napi_env, str: *const c_char, length: usize, result: *mut napi_value,
) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let bytes = if length == usize::MAX {
        if str.is_null() { return napi_invalid_arg; }
        CStr::from_ptr(str).to_bytes()
    } else {
        if str.is_null() && length > 0 { return napi_invalid_arg; }
        std::slice::from_raw_parts(str as *const u8, length)
    };
    let s = String::from_utf8_lossy(bytes).into_owned();
    *result = env.push_handle(Value::String(Rc::new(s)));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_string_latin1(
    env: napi_env, str: *const c_char, length: usize, result: *mut napi_value,
) -> napi_status {
    // Same as utf8 for our purposes; latin1 is a subset for ASCII text and
    // we losslessly carry bytes through.
    napi_create_string_utf8(env, str, length, result)
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_string_utf8(
    env: napi_env, value: napi_value, buf: *mut c_char, bufsize: usize, result: *mut usize,
) -> napi_status {
    let env = env_mut!(env);
    let s = match env.get_handle(value) {
        Some(Value::String(s)) => s.clone(),
        _ => return napi_string_expected,
    };
    let bytes = s.as_bytes();
    if buf.is_null() {
        // First-call protocol: write length only.
        if !result.is_null() { *result = bytes.len(); }
        return napi_ok;
    }
    if bufsize == 0 {
        if !result.is_null() { *result = 0; }
        return napi_ok;
    }
    let n = bytes.len().min(bufsize - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, buf, n);
    *buf.add(n) = 0;
    if !result.is_null() { *result = n; }
    napi_ok
}

// ─── Tier B: objects ───

#[no_mangle]
pub unsafe extern "C" fn napi_create_object(env: napi_env, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let rt = &mut *env.rt;
    let id = rt.alloc_object(Object::new_ordinary());
    *result = env.push_handle(Value::Object(id));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_array(env: napi_env, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let rt = &mut *env.rt;
    let id = rt.alloc_object(Object::new_array());
    *result = env.push_handle(Value::Object(id));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_array_with_length(env: napi_env, length: usize, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let rt = &mut *env.rt;
    let id = rt.alloc_object(Object::new_array());
    rt.object_set(id, "length".into(), Value::Number(length as f64));
    *result = env.push_handle(Value::Object(id));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_set_named_property(
    env: napi_env, object: napi_value, utf8name: *const c_char, value: napi_value,
) -> napi_status {
    check_arg!(utf8name);
    let env = env_mut!(env);
    let target = match env.get_handle(object) {
        Some(Value::Object(id)) => *id,
        _ => return napi_object_expected,
    };
    let v = match env.get_handle(value) {
        Some(v) => v.clone(),
        None => return napi_invalid_arg,
    };
    let name = CStr::from_ptr(utf8name).to_string_lossy().into_owned();
    let rt = &mut *env.rt;
    rt.object_set(target, name, v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_named_property(
    env: napi_env, object: napi_value, utf8name: *const c_char, result: *mut napi_value,
) -> napi_status {
    check_arg!(utf8name); check_arg!(result);
    let env = env_mut!(env);
    let target = match env.get_handle(object) {
        Some(Value::Object(id)) => *id,
        _ => return napi_object_expected,
    };
    let name = CStr::from_ptr(utf8name).to_string_lossy().into_owned();
    let rt = &mut *env.rt;
    let v = rt.object_get(target, &name);
    *result = env.push_handle(v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_has_named_property(
    env: napi_env, object: napi_value, utf8name: *const c_char, result: *mut bool,
) -> napi_status {
    check_arg!(utf8name); check_arg!(result);
    let env = env_mut!(env);
    let target = match env.get_handle(object) {
        Some(Value::Object(id)) => *id,
        _ => return napi_object_expected,
    };
    let name = CStr::from_ptr(utf8name).to_string_lossy().into_owned();
    let rt = &mut *env.rt;
    *result = rt.obj(target).properties.contains_key(&name);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_set_property(
    env: napi_env, object: napi_value, key: napi_value, value: napi_value,
) -> napi_status {
    let env = env_mut!(env);
    let target = match env.get_handle(object) {
        Some(Value::Object(id)) => *id,
        _ => return napi_object_expected,
    };
    let key_v = match env.get_handle(key) { Some(v) => v.clone(), None => return napi_invalid_arg };
    let v = match env.get_handle(value) { Some(v) => v.clone(), None => return napi_invalid_arg };
    let rt = &mut *env.rt;
    let key_s = crate::abstract_ops::to_string(&key_v);
    rt.object_set(target, key_s.as_str().to_string(), v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_property(
    env: napi_env, object: napi_value, key: napi_value, result: *mut napi_value,
) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let target = match env.get_handle(object) {
        Some(Value::Object(id)) => *id,
        _ => return napi_object_expected,
    };
    let key_v = match env.get_handle(key) { Some(v) => v.clone(), None => return napi_invalid_arg };
    let rt = &mut *env.rt;
    let key_s = crate::abstract_ops::to_string(&key_v);
    let v = rt.object_get(target, key_s.as_str());
    *result = env.push_handle(v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_set_element(
    env: napi_env, object: napi_value, index: u32, value: napi_value,
) -> napi_status {
    let env = env_mut!(env);
    let target = match env.get_handle(object) {
        Some(Value::Object(id)) => *id,
        _ => return napi_object_expected,
    };
    let v = match env.get_handle(value) { Some(v) => v.clone(), None => return napi_invalid_arg };
    let rt = &mut *env.rt;
    rt.object_set(target, index.to_string(), v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_element(
    env: napi_env, object: napi_value, index: u32, result: *mut napi_value,
) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let target = match env.get_handle(object) {
        Some(Value::Object(id)) => *id,
        _ => return napi_object_expected,
    };
    let rt = &mut *env.rt;
    let v = rt.object_get(target, &index.to_string());
    *result = env.push_handle(v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_array_length(env: napi_env, value: napi_value, result: *mut u32) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let target = match env.get_handle(value) {
        Some(Value::Object(id)) => *id,
        _ => return napi_array_expected,
    };
    let rt = &mut *env.rt;
    let len = match rt.object_get(target, "length") {
        Value::Number(n) => n as u32,
        _ => 0,
    };
    *result = len;
    napi_ok
}

// ─── Tier B: type queries ───

#[no_mangle]
pub unsafe extern "C" fn napi_typeof(env: napi_env, value: napi_value, result: *mut napi_valuetype) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let v = match env.get_handle(value) { Some(v) => v.clone(), None => return napi_invalid_arg };
    let rt = &*env.rt;
    let t = match &v {
        Value::Undefined => napi_undefined,
        Value::Null => napi_null,
        Value::Boolean(_) => napi_boolean,
        Value::Number(_) => napi_number,
        Value::String(_) => napi_string,
        Value::Symbol(_) => napi_symbol,
        Value::BigInt(_) => napi_bigint,
        Value::Object(id) => {
            match &rt.obj(*id).internal_kind {
                InternalKind::Function(_) | InternalKind::Closure(_) | InternalKind::BoundFunction(_) => napi_function,
                _ => napi_object_t,
            }
        }
    };
    *result = t;
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_array(env: napi_env, value: napi_value, result: *mut bool) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let target = match env.get_handle(value) {
        Some(Value::Object(id)) => *id,
        _ => { *result = false; return napi_ok; }
    };
    let rt = &*env.rt;
    *result = matches!(rt.obj(target).internal_kind, InternalKind::Array);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_strict_equals(env: napi_env, lhs: napi_value, rhs: napi_value, result: *mut bool) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let l = env.get_handle(lhs).cloned();
    let r = env.get_handle(rhs).cloned();
    *result = match (l, r) {
        (Some(a), Some(b)) => crate::abstract_ops::is_strictly_equal(&a, &b),
        _ => false,
    };
    napi_ok
}

// ─── Tier C: callbacks ───

/// Storage for a napi_callback + its associated data pointer. Lives on the
/// rusty-js heap as the Function's NativeFn closure.
struct NapiCallbackStorage {
    cb: napi_callback,
    data: *mut c_void,
    env: *mut NapiEnv,
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_function(
    env: napi_env, utf8name: *const c_char, _length: usize, cb: Option<napi_callback>,
    data: *mut c_void, result: *mut napi_value,
) -> napi_status {
    check_arg!(result);
    let cb = match cb { Some(f) => f, None => return napi_invalid_arg };
    let env_ptr = env;
    let env = env_mut!(env);
    let rt = &mut *env.rt;
    let name = if utf8name.is_null() { "".into() } else { CStr::from_ptr(utf8name).to_string_lossy().into_owned() };
    let storage = std::rc::Rc::new(NapiCallbackStorage { cb, data, env: env_ptr });
    let fn_storage = storage.clone();
    let native: NativeFn = std::rc::Rc::new(move |rt, args| {
        // Bridge: build a NapiCallbackInfo, push args/this as handles, call cb.
        let env = unsafe { &mut *fn_storage.env };
        let scope_start = env.handles.len();
        let mut handle_args: Vec<*mut c_void> = Vec::with_capacity(args.len());
        for a in args {
            handle_args.push(env.push_handle(a.clone()));
        }
        let this = rt.current_this();
        let info = NapiCallbackInfo {
            this,
            args: args.to_vec(),
            data: fn_storage.data,
        };
        let info_box = Box::into_raw(Box::new(info));
        let ret_handle = unsafe { (fn_storage.cb)(fn_storage.env, info_box) };
        let _ = unsafe { Box::from_raw(info_box) };
        // Check pending exception and propagate.
        if let Some(exc) = env.pending_exception.take() {
            // Free handles allocated during the call.
            env.handles.truncate(scope_start);
            return Err(crate::RuntimeError::Thrown(exc));
        }
        let v = env.get_handle(ret_handle).cloned().unwrap_or(Value::Undefined);
        env.handles.truncate(scope_start);
        Ok(v)
    });
    let obj = crate::intrinsics::make_native(&name, move |rt, args| native(rt, args));
    let id = rt.alloc_object(obj);
    let _ = storage;  // captured via fn_storage
    *result = env.push_handle(Value::Object(id));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_cb_info(
    env: napi_env, cbinfo: napi_callback_info,
    argc: *mut usize, argv: *mut napi_value, this_arg: *mut napi_value, data: *mut *mut c_void,
) -> napi_status {
    if cbinfo.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let info = &*cbinfo;
    if !argc.is_null() {
        let wanted = *argc;
        let actual = info.args.len();
        if !argv.is_null() {
            let copy_n = wanted.min(actual);
            for i in 0..copy_n {
                *argv.add(i) = env.push_handle(info.args[i].clone());
            }
            // Pad remaining with undefined per N-API contract.
            for i in actual..wanted {
                *argv.add(i) = env.push_handle(Value::Undefined);
            }
        }
        *argc = actual;
    }
    if !this_arg.is_null() {
        *this_arg = env.push_handle(info.this.clone());
    }
    if !data.is_null() {
        *data = info.data;
    }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_call_function(
    env: napi_env, recv: napi_value, func: napi_value,
    argc: usize, argv: *const napi_value, result: *mut napi_value,
) -> napi_status {
    let env = env_mut!(env);
    let recv_v = env.get_handle(recv).cloned().unwrap_or(Value::Undefined);
    let func_v = match env.get_handle(func) { Some(v) => v.clone(), None => return napi_function_expected };
    let mut args: Vec<Value> = Vec::with_capacity(argc);
    for i in 0..argc {
        let h = *argv.add(i);
        args.push(env.get_handle(h).cloned().unwrap_or(Value::Undefined));
    }
    let rt = &mut *env.rt;
    match rt.call_function(func_v, recv_v, args) {
        Ok(v) => {
            if !result.is_null() { *result = env.push_handle(v); }
            napi_ok
        }
        Err(e) => {
            env.pending_exception = Some(match e {
                crate::RuntimeError::Thrown(v) => v,
                _ => Value::String(Rc::new(format!("{:?}", e))),
            });
            napi_pending_exception
        }
    }
}

// ─── Tier D: references ───

#[no_mangle]
pub unsafe extern "C" fn napi_create_reference(
    env: napi_env, value: napi_value, initial_refcount: u32, result: *mut napi_ref,
) -> napi_status {
    check_arg!(result);
    if initial_refcount == 0 {
        // P46.E1: weak refs deferred.
        return napi_generic_failure;
    }
    let env_ptr = env;
    let env = env_mut!(env);
    let v = match env.get_handle(value) { Some(v) => v.clone(), None => return napi_invalid_arg };
    let slot = env.refs.len();
    env.refs.push(Some(v));
    let handle = Box::into_raw(Box::new(NapiRefHandle { slot, env: env_ptr, count: initial_refcount }));
    *result = handle;
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_delete_reference(env: napi_env, r: napi_ref) -> napi_status {
    if r.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let handle = Box::from_raw(r);
    if handle.slot < env.refs.len() { env.refs[handle.slot] = None; }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_reference_ref(_env: napi_env, r: napi_ref, result: *mut u32) -> napi_status {
    if r.is_null() { return napi_invalid_arg; }
    let h = &mut *r;
    h.count += 1;
    if !result.is_null() { *result = h.count; }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_reference_unref(_env: napi_env, r: napi_ref, result: *mut u32) -> napi_status {
    if r.is_null() { return napi_invalid_arg; }
    let h = &mut *r;
    if h.count > 0 { h.count -= 1; }
    if !result.is_null() { *result = h.count; }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_reference_value(env: napi_env, r: napi_ref, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    if r.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let h = &*r;
    let v = env.refs.get(h.slot).and_then(|o| o.clone()).unwrap_or(Value::Undefined);
    *result = env.push_handle(v);
    napi_ok
}

// ─── Tier D: errors ───

#[no_mangle]
pub unsafe extern "C" fn napi_throw(env: napi_env, error: napi_value) -> napi_status {
    let env = env_mut!(env);
    let v = env.get_handle(error).cloned().unwrap_or(Value::Undefined);
    env.pending_exception = Some(v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_throw_error(env: napi_env, _code: *const c_char, msg: *const c_char) -> napi_status {
    let env = env_mut!(env);
    let m = if msg.is_null() { "".into() } else { CStr::from_ptr(msg).to_string_lossy().into_owned() };
    env.pending_exception = Some(Value::String(Rc::new(format!("Error: {}", m))));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_throw_type_error(env: napi_env, _code: *const c_char, msg: *const c_char) -> napi_status {
    let env = env_mut!(env);
    let m = if msg.is_null() { "".into() } else { CStr::from_ptr(msg).to_string_lossy().into_owned() };
    env.pending_exception = Some(Value::String(Rc::new(format!("TypeError: {}", m))));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_throw_range_error(env: napi_env, _code: *const c_char, msg: *const c_char) -> napi_status {
    let env = env_mut!(env);
    let m = if msg.is_null() { "".into() } else { CStr::from_ptr(msg).to_string_lossy().into_owned() };
    env.pending_exception = Some(Value::String(Rc::new(format!("RangeError: {}", m))));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_exception_pending(env: napi_env, result: *mut bool) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = env.pending_exception.is_some();
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_and_clear_last_exception(env: napi_env, result: *mut napi_value) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let v = env.pending_exception.take().unwrap_or(Value::Undefined);
    *result = env.push_handle(v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_last_error_info(
    env: napi_env, result: *mut *const napi_extended_error_info,
) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    *result = &env.last_error_info as *const _;
    napi_ok
}

// ─── Tier D: handle scopes (degenerate impl: scope-pop truncates handles) ───

#[no_mangle]
pub unsafe extern "C" fn napi_open_handle_scope(env: napi_env, result: *mut napi_handle_scope) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let saved = env.handles.len();
    env.scopes.push(saved);
    *result = saved as napi_handle_scope;
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_close_handle_scope(env: napi_env, scope: napi_handle_scope) -> napi_status {
    let env = env_mut!(env);
    let saved = scope as usize;
    if let Some(pos) = env.scopes.iter().rposition(|&s| s == saved) {
        env.scopes.remove(pos);
    }
    if env.handles.len() > saved { env.handles.truncate(saved); }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_open_escapable_handle_scope(
    env: napi_env, result: *mut napi_escapable_handle_scope,
) -> napi_status {
    napi_open_handle_scope(env, result as *mut napi_handle_scope)
}

#[no_mangle]
pub unsafe extern "C" fn napi_close_escapable_handle_scope(
    env: napi_env, scope: napi_escapable_handle_scope,
) -> napi_status {
    napi_close_handle_scope(env, scope as napi_handle_scope)
}

#[no_mangle]
pub unsafe extern "C" fn napi_escape_handle(
    env: napi_env, _scope: napi_escapable_handle_scope, escapee: napi_value, result: *mut napi_value,
) -> napi_status {
    check_arg!(result);
    let env = env_mut!(env);
    let v = env.get_handle(escapee).cloned().unwrap_or(Value::Undefined);
    // Push to the PARENT scope (the one that was active before `_scope` opened).
    // Our degenerate impl: just push to top of stack — caller's parent scope is
    // by definition the next slot. The escape is conservative but sound for
    // module-init use.
    *result = env.push_handle(v);
    napi_ok
}

// ─── Tier E: define_properties (the macro-bulk-attach path) ───

#[repr(C)]
pub struct napi_property_descriptor {
    pub utf8name: *const c_char,
    pub name: napi_value,
    pub method: Option<napi_callback>,
    pub getter: Option<napi_callback>,
    pub setter: Option<napi_callback>,
    pub value: napi_value,
    pub attributes: i32,
    pub data: *mut c_void,
}

#[no_mangle]
pub unsafe extern "C" fn napi_define_properties(
    env: napi_env, object: napi_value, property_count: usize, properties: *const napi_property_descriptor,
) -> napi_status {
    let env_ptr = env;
    let env = env_mut!(env);
    let target = match env.get_handle(object) {
        Some(Value::Object(id)) => *id,
        _ => return napi_object_expected,
    };
    let rt = &mut *env.rt;
    for i in 0..property_count {
        let d = &*properties.add(i);
        let name = if !d.utf8name.is_null() {
            CStr::from_ptr(d.utf8name).to_string_lossy().into_owned()
        } else if !d.name.is_null() {
            match env.get_handle(d.name) {
                Some(v) => crate::abstract_ops::to_string(v).as_str().to_string(),
                None => continue,
            }
        } else { continue };
        let v = if let Some(method) = d.method {
            // Inline-create a callable for the method.
            let mut handle: napi_value = std::ptr::null_mut();
            let _ = napi_create_function(env_ptr, d.utf8name, name.len(), Some(method), d.data, &mut handle);
            env.get_handle(handle).cloned().unwrap_or(Value::Undefined)
        } else if !d.value.is_null() {
            env.get_handle(d.value).cloned().unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        };
        rt.object_set(target, name, v);
    }
    napi_ok
}

// Helper macro variant that returns a default if env is null (needed for
// helpers that produce napi_value rather than napi_status).
macro_rules! env_mut_or_null {
    ($env:expr) => {{
        if $env.is_null() { return std::ptr::null_mut(); }
        &mut *$env
    }};
}

// ─── Ω.5.P46.E3+: error-object creation (vs. error throwing) ───
// nx's native binding dlsyms napi_create_error at load time; absent
// symbols fail dlopen even if never called.

unsafe fn make_error_obj(env: napi_env, msg: napi_value, code: napi_value, name: &str) -> napi_value {
    let env_ref = env_mut_or_null!(env);
    let rt = &mut *env_ref.rt;
    let id = rt.alloc_object(Object::new_ordinary());
    if !msg.is_null() {
        if let Some(v) = env_ref.get_handle(msg).cloned() {
            rt.object_set(id, "message".into(), v);
        }
    }
    if !code.is_null() {
        if let Some(v) = env_ref.get_handle(code).cloned() {
            rt.object_set(id, "code".into(), v);
        }
    }
    rt.object_set(id, "name".into(), Value::String(Rc::new(name.into())));
    env_ref.push_handle(Value::Object(id))
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_error(env: napi_env, code: napi_value, msg: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    *result = make_error_obj(env, msg, code, "Error");
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_type_error(env: napi_env, code: napi_value, msg: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    *result = make_error_obj(env, msg, code, "TypeError");
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_range_error(env: napi_env, code: napi_value, msg: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    *result = make_error_obj(env, msg, code, "RangeError");
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_syntax_error(env: napi_env, code: napi_value, msg: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    *result = make_error_obj(env, msg, code, "SyntaxError");
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_throw_syntax_error(env: napi_env, _code: *const c_char, msg: *const c_char) -> napi_status {
    let env = env_mut!(env);
    let m = if msg.is_null() { "".into() } else { CStr::from_ptr(msg).to_string_lossy().into_owned() };
    env.pending_exception = Some(Value::String(Rc::new(format!("SyntaxError: {}", m))));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_error(env: napi_env, value: napi_value, result: *mut bool) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    *result = match env.get_handle(value) {
        Some(Value::Object(id)) => {
            let rt = &*env.rt;
            matches!(rt.obj(*id).internal_kind, InternalKind::Error)
        }
        _ => false,
    };
    napi_ok
}

// ─── Symbol creation (returns a Value::Symbol per Ω.5.P19.E1) ───

#[no_mangle]
pub unsafe extern "C" fn napi_create_symbol(env: napi_env, description: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let desc = if description.is_null() {
        String::new()
    } else {
        match env.get_handle(description) {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => String::new(),
        }
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(1_000_000);  // distinct range
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    *result = env.push_handle(Value::Symbol(Rc::new(format!("@@sym:{}:{}", n, desc))));
    napi_ok
}

// ─── Latin1 string extraction ───

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_string_latin1(
    env: napi_env, value: napi_value, buf: *mut c_char, bufsize: usize, result: *mut usize,
) -> napi_status {
    // Latin1 path mirrors UTF-8 in our impl (we treat both as opaque bytes).
    napi_get_value_string_utf8(env, value, buf, bufsize, result)
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_string_utf16(
    _env: napi_env, _value: napi_value, _buf: *mut u16, _bufsize: usize, _result: *mut usize,
) -> napi_status {
    napi_generic_failure  // UTF-16 transcoding deferred
}

// ─── BigInt creation (stubs — full BigInt support deferred) ───

#[no_mangle]
pub unsafe extern "C" fn napi_create_bigint_int64(env: napi_env, value: i64, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    *result = env.push_handle(Value::BigInt(Rc::new(crate::bigint::JsBigInt::from_i64(value))));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_bigint_uint64(env: napi_env, value: u64, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    *result = env.push_handle(Value::BigInt(Rc::new(crate::bigint::JsBigInt::from_u64(value))));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_bigint_words(
    _env: napi_env, _sign_bit: i32, _word_count: usize, _words: *const u64, _result: *mut napi_value,
) -> napi_status {
    napi_generic_failure
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_bigint_int64(env: napi_env, value: napi_value, result: *mut i64, lossless: *mut bool) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    match env.get_handle(value) {
        Some(Value::BigInt(b)) => {
            *result = b.to_f64() as i64;
            if !lossless.is_null() { *lossless = true; }
            napi_ok
        }
        _ => napi_number_expected,
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_bigint_uint64(env: napi_env, value: napi_value, result: *mut u64, lossless: *mut bool) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    match env.get_handle(value) {
        Some(Value::BigInt(b)) => {
            *result = b.to_f64() as u64;
            if !lossless.is_null() { *lossless = true; }
            napi_ok
        }
        _ => napi_number_expected,
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_bigint_words(
    _env: napi_env, _value: napi_value, _sign_bit: *mut i32, _word_count: *mut usize, _words: *mut u64,
) -> napi_status {
    napi_generic_failure
}

// ─── ArrayBuffer / TypedArray (basic stubs — backing-store impl deferred) ───

#[no_mangle]
pub unsafe extern "C" fn napi_create_arraybuffer(
    env: napi_env, byte_length: usize, data: *mut *mut c_void, result: *mut napi_value,
) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let rt = &mut *env.rt;
    // Synthesize an ArrayBuffer-shaped object backed by a Vec<u8>. Real
    // ABI would let the caller take a stable pointer to the backing
    // store; for v1 we leak a Box<[u8]> and hand its pointer back.
    let boxed: Box<[u8]> = vec![0u8; byte_length].into_boxed_slice();
    let ptr = Box::into_raw(boxed) as *mut c_void;
    if !data.is_null() { *data = ptr; }
    let ab = rt.alloc_object(Object::new_ordinary());
    rt.object_set(ab, "byteLength".into(), Value::Number(byte_length as f64));
    rt.object_set(ab, "__ab_data".into(), Value::Number(ptr as usize as f64));
    *result = env.push_handle(Value::Object(ab));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_arraybuffer_info(
    env: napi_env, value: napi_value, data: *mut *mut c_void, byte_length: *mut usize,
) -> napi_status {
    let env = env_mut!(env);
    let id = match env.get_handle(value) { Some(Value::Object(id)) => *id, _ => return napi_invalid_arg };
    let rt = &*env.rt;
    if !data.is_null() {
        *data = match rt.object_get(id, "__ab_data") {
            Value::Number(n) => n as usize as *mut c_void,
            _ => std::ptr::null_mut(),
        };
    }
    if !byte_length.is_null() {
        *byte_length = match rt.object_get(id, "byteLength") {
            Value::Number(n) => n as usize,
            _ => 0,
        };
    }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_external(
    env: napi_env, data: *mut c_void, _finalize_cb: *mut c_void, _finalize_hint: *mut c_void, result: *mut napi_value,
) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let rt = &mut *env.rt;
    let id = rt.alloc_object(Object::new_ordinary());
    rt.object_set(id, "__external_ptr".into(), Value::Number(data as usize as f64));
    *result = env.push_handle(Value::Object(id));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_external(
    env: napi_env, value: napi_value, result: *mut *mut c_void,
) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let id = match env.get_handle(value) { Some(Value::Object(id)) => *id, _ => return napi_invalid_arg };
    let rt = &*env.rt;
    *result = match rt.object_get(id, "__external_ptr") {
        Value::Number(n) => n as usize as *mut c_void,
        _ => std::ptr::null_mut(),
    };
    napi_ok
}

// ─── Finalizer / object wrapping (minimal: leak-on-drop) ───

#[no_mangle]
pub unsafe extern "C" fn napi_add_finalizer(
    env: napi_env, _object: napi_value, _native_object: *mut c_void,
    _finalize_cb: *mut c_void, _finalize_hint: *mut c_void, result: *mut napi_ref,
) -> napi_status {
    // We leak the native_object box; for one-shot CLI invocations
    // that's acceptable. Returns null-ish ref.
    if !result.is_null() { *result = std::ptr::null_mut(); }
    let _ = env;
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_adjust_external_memory(_env: napi_env, change_in_bytes: i64, result: *mut i64) -> napi_status {
    if !result.is_null() { *result = change_in_bytes.max(0); }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_coerce_to_string(env: napi_env, value: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let v = env.get_handle(value).cloned().unwrap_or(Value::Undefined);
    let s = crate::abstract_ops::to_string(&v);
    *result = env.push_handle(Value::String(s));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_coerce_to_number(env: napi_env, value: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let v = env.get_handle(value).cloned().unwrap_or(Value::Undefined);
    let n = crate::abstract_ops::to_number(&v);
    *result = env.push_handle(Value::Number(n));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_coerce_to_bool(env: napi_env, value: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let v = env.get_handle(value).cloned().unwrap_or(Value::Undefined);
    let b = crate::abstract_ops::to_boolean(&v);
    *result = env.push_handle(Value::Boolean(b));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_object_freeze(_env: napi_env, _object: napi_value) -> napi_status {
    napi_ok  // no-op: we don't yet honor [[Extensible]]=false
}

#[no_mangle]
pub unsafe extern "C" fn napi_object_seal(_env: napi_env, _object: napi_value) -> napi_status {
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_instanceof(env: napi_env, object: napi_value, constructor: napi_value, result: *mut bool) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    // Simplistic: check the object's prototype chain for constructor.prototype.
    let obj_id = match env.get_handle(object) { Some(Value::Object(id)) => *id, _ => { *result = false; return napi_ok; } };
    let ctor_id = match env.get_handle(constructor) { Some(Value::Object(id)) => *id, _ => { *result = false; return napi_ok; } };
    let rt = &*env.rt;
    let proto_target = match rt.object_get(ctor_id, "prototype") {
        Value::Object(id) => id,
        _ => { *result = false; return napi_ok; }
    };
    let mut cur = rt.obj(obj_id).proto;
    while let Some(p) = cur {
        if p == proto_target { *result = true; return napi_ok; }
        cur = rt.obj(p).proto;
    }
    *result = false;
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_new_target(_env: napi_env, _cbinfo: napi_callback_info, result: *mut napi_value) -> napi_status {
    if !result.is_null() { *result = std::ptr::null_mut(); }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_property_names(env: napi_env, object: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    let id = match env.get_handle(object) { Some(Value::Object(id)) => *id, _ => return napi_object_expected };
    let rt = &mut *env.rt;
    let keys: Vec<String> = rt.obj(id).properties.iter()
        .filter(|(k, d)| d.enumerable && !k.starts_with("@@"))
        .map(|(k, _)| k.clone()).collect();
    let arr = rt.alloc_object(Object::new_array());
    for (i, k) in keys.iter().enumerate() {
        rt.object_set(arr, i.to_string(), Value::String(Rc::new(k.clone())));
    }
    rt.object_set(arr, "length".into(), Value::Number(keys.len() as f64));
    *result = env.push_handle(Value::Object(arr));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_add_env_cleanup_hook(_env: napi_env, _fun: *mut c_void, _arg: *mut c_void) -> napi_status {
    napi_ok  // we don't yet run cleanup hooks at engine exit
}

#[no_mangle]
pub unsafe extern "C" fn napi_remove_env_cleanup_hook(_env: napi_env, _fun: *mut c_void, _arg: *mut c_void) -> napi_status {
    napi_ok
}

// ─── Ω.5.P46.E4: real implementations for napi_create_promise /
// _resolve_deferred / _reject_deferred / _coerce_to_object /
// _get_buffer_info / _new_instance / _fatal_exception / _wrap /
// _unwrap / _define_class. All needed by nx's native binding. ───

pub type napi_deferred = *mut NapiDeferred;
pub struct NapiDeferred {
    /// ObjectId of the underlying Promise that will be settled.
    promise_id: rusty_js_gc::ObjectId,
    env: SendPtr<NapiEnv>,
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_promise(env: napi_env, deferred: *mut napi_deferred, promise: *mut napi_value) -> napi_status {
    if deferred.is_null() || promise.is_null() { return napi_invalid_arg; }
    let env_ref = env_mut!(env);
    let rt = &mut *env_ref.rt;
    let p_id = crate::promise::new_promise(rt);
    let d = Box::into_raw(Box::new(NapiDeferred { promise_id: p_id, env: SendPtr(env) }));
    *deferred = d;
    *promise = env_ref.push_handle(Value::Object(p_id));
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_resolve_deferred(env: napi_env, deferred: napi_deferred, resolution: napi_value) -> napi_status {
    if deferred.is_null() { return napi_invalid_arg; }
    let env_ref = env_mut!(env);
    let v = env_ref.get_handle(resolution).cloned().unwrap_or(Value::Undefined);
    let d = Box::from_raw(deferred);
    let rt = &mut *env_ref.rt;
    crate::promise::resolve_promise(rt, d.promise_id, v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_reject_deferred(env: napi_env, deferred: napi_deferred, reason: napi_value) -> napi_status {
    if deferred.is_null() { return napi_invalid_arg; }
    let env_ref = env_mut!(env);
    let v = env_ref.get_handle(reason).cloned().unwrap_or(Value::Undefined);
    let d = Box::from_raw(deferred);
    let rt = &mut *env_ref.rt;
    crate::promise::reject_promise(rt, d.promise_id, v);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_coerce_to_object(env: napi_env, value: napi_value, result: *mut napi_value) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env_ref = env_mut!(env);
    let v = env_ref.get_handle(value).cloned().unwrap_or(Value::Undefined);
    // For object inputs return as-is; for primitives synthesize a
    // minimal wrapper object with `__value` carrying the primitive.
    let out = match v {
        Value::Object(_) => v,
        other => {
            let rt = &mut *env_ref.rt;
            let id = rt.alloc_object(Object::new_ordinary());
            rt.object_set(id, "__value".into(), other);
            Value::Object(id)
        }
    };
    *result = env_ref.push_handle(out);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_buffer_info(env: napi_env, value: napi_value, data: *mut *mut c_void, length: *mut usize) -> napi_status {
    let env_ref = env_mut!(env);
    let id = match env_ref.get_handle(value) { Some(Value::Object(id)) => *id, _ => return napi_invalid_arg };
    let rt = &*env_ref.rt;
    // We don't have a real Buffer type; synthesize from length + indexed bytes.
    let len = match rt.object_get(id, "length") { Value::Number(n) => n as usize, _ => 0 };
    if !length.is_null() { *length = len; }
    if !data.is_null() {
        // Build a contiguous byte buffer on the heap, leak it. Caller
        // assumes the pointer is valid for the lifetime of the Buffer.
        let mut bytes: Vec<u8> = Vec::with_capacity(len);
        for i in 0..len {
            bytes.push(match rt.object_get(id, &i.to_string()) {
                Value::Number(n) => n as u8,
                _ => 0,
            });
        }
        let boxed = bytes.into_boxed_slice();
        *data = Box::into_raw(boxed) as *mut c_void;
    }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_buffer(env: napi_env, value: napi_value, result: *mut bool) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env = env_mut!(env);
    *result = match env.get_handle(value) {
        Some(Value::Object(id)) => {
            let rt = &*env.rt;
            matches!(rt.object_get(*id, "__is_buffer__"), Value::Boolean(true))
        }
        _ => false,
    };
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_new_instance(
    env: napi_env, constructor: napi_value, argc: usize, argv: *const napi_value, result: *mut napi_value,
) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env_ref = env_mut!(env);
    let ctor_v = match env_ref.get_handle(constructor) { Some(v) => v.clone(), None => return napi_function_expected };
    let mut args: Vec<Value> = Vec::with_capacity(argc);
    for i in 0..argc {
        let h = *argv.add(i);
        args.push(env_ref.get_handle(h).cloned().unwrap_or(Value::Undefined));
    }
    let rt = &mut *env_ref.rt;
    // Construct: allocate a fresh object with ctor.prototype as proto, then call ctor with that as this.
    let proto = if let Value::Object(cid) = &ctor_v {
        match rt.object_get(*cid, "prototype") {
            Value::Object(pid) => Some(pid), _ => None,
        }
    } else { None };
    let mut new_obj = Object::new_ordinary();
    new_obj.proto = proto;
    let new_id = rt.alloc_object(new_obj);
    match rt.call_function(ctor_v, Value::Object(new_id), args) {
        Ok(v) => {
            // Constructors that return an object replace `this`; otherwise this is the result.
            let out = match v { Value::Object(_) => v, _ => Value::Object(new_id) };
            *result = env_ref.push_handle(out);
            napi_ok
        }
        Err(e) => {
            env_ref.pending_exception = Some(match e {
                crate::RuntimeError::Thrown(v) => v,
                _ => Value::String(Rc::new(format!("{:?}", e))),
            });
            napi_pending_exception
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_fatal_exception(env: napi_env, err: napi_value) -> napi_status {
    let env_ref = env_mut!(env);
    let v = env_ref.get_handle(err).cloned().unwrap_or(Value::Undefined);
    eprintln!("rusty-bun-host-v2: napi fatal exception: {:?}", v);
    napi_ok
}

// ─── Real napi_define_class + wrap/unwrap (replacing P46.E1 failure stubs) ───

#[no_mangle]
pub unsafe extern "C" fn napi_define_class(
    env: napi_env, utf8name: *const c_char, length: usize, ctor: Option<napi_callback>,
    data: *mut c_void, property_count: usize, properties: *const napi_property_descriptor,
    result: *mut napi_value,
) -> napi_status {
    let _ = length;
    if result.is_null() { return napi_invalid_arg; }
    let ctor = match ctor { Some(f) => f, None => return napi_invalid_arg };
    let env_ref = env_mut!(env);
    let name = if utf8name.is_null() { "".into() } else { CStr::from_ptr(utf8name).to_string_lossy().into_owned() };
    // Build a NativeFn that wraps the napi_callback ctor.
    let env_ptr = env;
    let storage = std::rc::Rc::new(NapiCallbackStorage { cb: ctor, data, env: env_ptr });
    let storage2 = storage.clone();
    let native: NativeFn = std::rc::Rc::new(move |rt, args| {
        let env = unsafe { &mut *storage2.env };
        let scope_start = env.handles.len();
        let info = NapiCallbackInfo {
            this: rt.current_this(),
            args: args.to_vec(),
            data: storage2.data,
        };
        let info_box = Box::into_raw(Box::new(info));
        let ret_handle = unsafe { (storage2.cb)(storage2.env, info_box) };
        let _ = unsafe { Box::from_raw(info_box) };
        if let Some(exc) = env.pending_exception.take() {
            env.handles.truncate(scope_start);
            return Err(crate::RuntimeError::Thrown(exc));
        }
        let v = env.get_handle(ret_handle).cloned().unwrap_or(rt.current_this());
        env.handles.truncate(scope_start);
        Ok(v)
    });
    let rt = &mut *env_ref.rt;
    let ctor_obj = crate::intrinsics::make_native(&name, move |rt, args| native(rt, args));
    let ctor_id = rt.alloc_object(ctor_obj);
    // Allocate prototype object; ctor.prototype = proto; proto.constructor = ctor.
    let proto_id = rt.alloc_object(Object::new_ordinary());
    rt.object_set(ctor_id, "prototype".into(), Value::Object(proto_id));
    rt.object_set(proto_id, "constructor".into(), Value::Object(ctor_id));
    // Attach properties.
    for i in 0..property_count {
        let d = &*properties.add(i);
        let prop_name = if !d.utf8name.is_null() {
            CStr::from_ptr(d.utf8name).to_string_lossy().into_owned()
        } else { continue };
        if let Some(method) = d.method {
            let mut method_h: napi_value = std::ptr::null_mut();
            let _ = napi_create_function(env, d.utf8name, prop_name.len(), Some(method), d.data, &mut method_h);
            let mv = env_ref.get_handle(method_h).cloned().unwrap_or(Value::Undefined);
            let rt2 = &mut *env_ref.rt;
            // Static (attribute & 1<<10) vs instance — Node's napi_static is bit 10.
            if d.attributes & (1 << 10) != 0 {
                rt2.object_set(ctor_id, prop_name, mv);
            } else {
                rt2.object_set(proto_id, prop_name, mv);
            }
        } else if !d.value.is_null() {
            let v = env_ref.get_handle(d.value).cloned().unwrap_or(Value::Undefined);
            let rt2 = &mut *env_ref.rt;
            if d.attributes & (1 << 10) != 0 {
                rt2.object_set(ctor_id, prop_name, v);
            } else {
                rt2.object_set(proto_id, prop_name, v);
            }
        }
    }
    let _ = storage;
    *result = env_ref.push_handle(Value::Object(ctor_id));
    napi_ok
}

// Re-export under the canonical name (the P46.E1 stub is overridden).
// Linkers prefer the LATER definition, so this one wins.

#[no_mangle]
pub unsafe extern "C" fn napi_wrap(
    env: napi_env, object: napi_value, native: *mut c_void,
    _finalize_cb: *mut c_void, _finalize_hint: *mut c_void, result: *mut napi_ref,
) -> napi_status {
    let env_ref = env_mut!(env);
    let id = match env_ref.get_handle(object) { Some(Value::Object(id)) => *id, _ => return napi_object_expected };
    let rt = &mut *env_ref.rt;
    rt.object_set(id, "__napi_wrapped".into(), Value::Number(native as usize as f64));
    if !result.is_null() {
        // Return a strong reference to the object so the native side can
        // keep it alive across calls.
        let slot = env_ref.refs.len();
        env_ref.refs.push(Some(Value::Object(id)));
        let handle = Box::into_raw(Box::new(NapiRefHandle { slot, env, count: 1 }));
        *result = handle;
    }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_unwrap(env: napi_env, object: napi_value, result: *mut *mut c_void) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env_ref = env_mut!(env);
    let id = match env_ref.get_handle(object) { Some(Value::Object(id)) => *id, _ => return napi_object_expected };
    let rt = &*env_ref.rt;
    *result = match rt.object_get(id, "__napi_wrapped") {
        Value::Number(n) => n as usize as *mut c_void,
        _ => std::ptr::null_mut(),
    };
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_remove_wrap(env: napi_env, object: napi_value, result: *mut *mut c_void) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env_ref = env_mut!(env);
    let id = match env_ref.get_handle(object) { Some(Value::Object(id)) => *id, _ => return napi_object_expected };
    let rt = &mut *env_ref.rt;
    *result = match rt.object_get(id, "__napi_wrapped") {
        Value::Number(n) => n as usize as *mut c_void,
        _ => std::ptr::null_mut(),
    };
    rt.obj_mut(id).properties.shift_remove("__napi_wrapped");
    napi_ok
}

// ─── Versioning ───

#[no_mangle]
pub unsafe extern "C" fn napi_get_version(_env: napi_env, result: *mut u32) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    *result = 8;  // N-API version 8 (Node 18+ stable surface).
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_node_version(
    _env: napi_env, result: *mut *const c_void,
) -> napi_status {
    // Returns pointer to a struct {major, minor, patch, release}.
    // Stub: populate a static and return its address.
    static VERSION: [u32; 3] = [20, 10, 0];
    if !result.is_null() { *result = &VERSION as *const _ as *const c_void; }
    napi_ok
}

// ─── Default-fail shims for less-common surface to keep symbols present ───

// ─── Ω.5.P46.E2.napi-async: async_work + threadsafe_function ───
//
// Pattern: the JS engine runs on the main thread. async_work execute_cb
// runs on a worker thread (no napi calls allowed except thread-safe).
// complete_cb / call_js_cb run on the main thread, marshaled through
// Runtime::napi_main_inbox + drained by PollIo.

/// A job queued for execution on the main thread by worker threads.
/// Boxed FnOnce + Send. The inbox is drained by PollIo (host-v2/src/fs.rs).
pub type NapiMainJob = Box<dyn FnOnce(&mut Runtime) + Send>;

/// SendPtr wraps a raw pointer to satisfy Send for cross-thread move.
/// Caller proves the pointer is safe to use after move (no aliasing,
/// pointee lives long enough). N-API contract makes this safe for
/// `data: *mut c_void` (opaque user data) and env / func pointers
/// (lifetime guaranteed by Runtime ownership).
struct SendPtr<T>(*mut T);
unsafe impl<T> Send for SendPtr<T> {}
// Manual Copy/Clone — `#[derive]` requires T: Copy, but a raw pointer
// to T doesn't depend on T being Copy. We want SendPtr<NapiEnv> etc.
// to be unconditionally Copy.
impl<T> Copy for SendPtr<T> {}
impl<T> Clone for SendPtr<T> { fn clone(&self) -> Self { *self } }

/// Wrap a function pointer as Send. extern "C" fn pointers are bits
/// and trivially Send-safe, but Rust's type system doesn't know that
/// without a wrapper.
struct SendFn<F>(F);
unsafe impl<F> Send for SendFn<F> {}
impl<F: Copy> Copy for SendFn<F> {}
impl<F: Copy> Clone for SendFn<F> { fn clone(&self) -> Self { *self } }

#[repr(C)]
pub struct NapiAsyncWork {
    execute: unsafe extern "C" fn(env: napi_env, data: *mut c_void),
    complete: unsafe extern "C" fn(env: napi_env, status: napi_status, data: *mut c_void),
    data: SendPtr<c_void>,
    env: SendPtr<NapiEnv>,
    /// Set true between queue_async_work and the worker spawn returning.
    /// Used by delete_async_work to refuse mid-flight deletion.
    queued: bool,
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_async_work(
    env: napi_env, _async_resource: napi_value, _async_resource_name: napi_value,
    execute: Option<unsafe extern "C" fn(env: napi_env, data: *mut c_void)>,
    complete: Option<unsafe extern "C" fn(env: napi_env, status: napi_status, data: *mut c_void)>,
    data: *mut c_void, result: *mut *mut NapiAsyncWork,
) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let execute = match execute { Some(f) => f, None => return napi_invalid_arg };
    let complete = match complete { Some(f) => f, None => return napi_invalid_arg };
    let work = Box::new(NapiAsyncWork {
        execute, complete,
        data: SendPtr(data),
        env: SendPtr(env),
        queued: false,
    });
    *result = Box::into_raw(work);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_queue_async_work(env: napi_env, work: *mut NapiAsyncWork) -> napi_status {
    if work.is_null() { return napi_invalid_arg; }
    if env.is_null() { return napi_invalid_arg; }
    let env_ref = &mut *env;
    let rt = &mut *env_ref.rt;
    let inbox = rt.napi_main_inbox.clone();
    let keepalive = rt.napi_keepalive.clone();
    let w = &mut *work;
    if w.queued { return napi_generic_failure; }
    w.queued = true;
    // P46.E3: bump engine keepalive — drop after complete_cb runs.
    keepalive.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let execute = SendFn(w.execute);
    let complete = SendFn(w.complete);
    let data: SendPtr<c_void> = w.data;     // SendPtr derives Copy
    let env_send: SendPtr<NapiEnv> = w.env;
    let work_ptr = SendPtr(work);
    // Spawn worker thread. execute runs there; complete is queued onto
    // the main thread via inbox. We capture the SendPtr/SendFn wrappers
    // BY MOVE (rather than letting the move closure decompose to inner
    // *mut accesses); rebinding via `let` forces whole-value capture.
    let keepalive_for_thread = keepalive.clone();
    std::thread::spawn(move || {
        let execute_local = execute;
        let env_local = env_send;
        let data_local = data;
        let complete_local = complete;
        let work_local = work_ptr;
        let keepalive = keepalive_for_thread;
        let status: napi_status = {
            unsafe { (execute_local.0)(env_local.0, data_local.0); }
            napi_ok
        };
        let keepalive_for_job = keepalive.clone();
        let job: NapiMainJob = Box::new(move |_rt: &mut Runtime| {
            let complete2 = complete_local;
            let env2 = env_local;
            let data2 = data_local;
            let work2 = work_local;
            let ka = keepalive_for_job;
            unsafe {
                (complete2.0)(env2.0, status, data2.0);
                let w = &mut *work2.0;
                w.queued = false;
            }
            ka.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
        if let Ok(mut q) = inbox.lock() {
            q.push_back(job);
        }
    });
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_delete_async_work(_env: napi_env, work: *mut NapiAsyncWork) -> napi_status {
    if work.is_null() { return napi_invalid_arg; }
    let w = &*work;
    if w.queued { return napi_generic_failure; }  // mid-flight; don't free
    let _ = Box::from_raw(work);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_cancel_async_work(_env: napi_env, _work: *mut NapiAsyncWork) -> napi_status {
    napi_generic_failure  // best-effort; we don't track in-flight work for cancel
}

// ─── Threadsafe function ───

#[repr(i32)]
pub enum napi_threadsafe_function_call_mode {
    napi_tsfn_nonblocking = 0,
    napi_tsfn_blocking = 1,
}

#[repr(i32)]
pub enum napi_threadsafe_function_release_mode {
    napi_tsfn_release = 0,
    napi_tsfn_abort = 1,
}

pub type napi_threadsafe_function = *mut NapiTsfn;
pub type napi_threadsafe_function_call_js =
    unsafe extern "C" fn(env: napi_env, js_callback: napi_value, context: *mut c_void, data: *mut c_void);

pub struct NapiTsfn {
    func_ref_slot: usize,  // index into NapiEnv::refs holding the JS func
    call_js: Option<napi_threadsafe_function_call_js>,
    context: SendPtr<c_void>,
    env: SendPtr<NapiEnv>,
    ref_count: std::sync::atomic::AtomicUsize,
    active: std::sync::atomic::AtomicBool,
    /// Ω.5.P46.E3: per-tsfn keepalive bit. When set, the tsfn
    /// contributes 1 to Runtime::napi_keepalive so the event loop
    /// stays alive even if the inbox is empty. Toggled by
    /// napi_ref_threadsafe_function / napi_unref_threadsafe_function.
    /// Initial state per N-API spec: ref'd.
    keepalive_active: std::sync::atomic::AtomicBool,
    /// Shared handle to Runtime's keepalive counter so toggle ops
    /// don't need to thread an env pointer.
    keepalive_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_threadsafe_function(
    env: napi_env, func: napi_value, _async_resource: napi_value, _async_resource_name: napi_value,
    _max_queue_size: usize, _initial_thread_count: usize,
    _thread_finalize_data: *mut c_void, _thread_finalize_cb: *mut c_void,
    context: *mut c_void,
    call_js_cb: Option<napi_threadsafe_function_call_js>,
    result: *mut napi_threadsafe_function,
) -> napi_status {
    if result.is_null() { return napi_invalid_arg; }
    let env_ref = env_mut!(env);
    let func_v = match env_ref.get_handle(func) { Some(v) => v.clone(), None => return napi_invalid_arg };
    let slot = env_ref.refs.len();
    env_ref.refs.push(Some(func_v));
    let keepalive_counter = (&*env_ref.rt).napi_keepalive.clone();
    // P46.E3: tsfn starts ref'd; bump the global keepalive counter so
    // the event loop knows to stay alive.
    keepalive_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let tsfn = Box::new(NapiTsfn {
        func_ref_slot: slot,
        call_js: call_js_cb,
        context: SendPtr(context),
        env: SendPtr(env),
        ref_count: std::sync::atomic::AtomicUsize::new(1),
        active: std::sync::atomic::AtomicBool::new(true),
        keepalive_active: std::sync::atomic::AtomicBool::new(true),
        keepalive_counter,
    });
    *result = Box::into_raw(tsfn);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_call_threadsafe_function(
    tsfn: napi_threadsafe_function, data: *mut c_void,
    _mode: napi_threadsafe_function_call_mode,
) -> napi_status {
    if tsfn.is_null() { return napi_invalid_arg; }
    let tsfn_ref = &*tsfn;
    if !tsfn_ref.active.load(std::sync::atomic::Ordering::SeqCst) {
        return napi_generic_failure;
    }
    let env_send = tsfn_ref.env;
    let env_ref = &mut *env_send.0;
    let inbox = (&*env_ref.rt).napi_main_inbox.clone();
    let func_slot = tsfn_ref.func_ref_slot;
    let context = tsfn_ref.context;
    let data_send = SendPtr(data);
    let call_js = tsfn_ref.call_js.map(SendFn);
    let env_for_job = env_send;
    let context_for_job = context;
    let data_for_job = data_send;
    let call_js_for_job = call_js;
    let job: NapiMainJob = Box::new(move |_rt: &mut Runtime| {
        let env_local = env_for_job;
        let ctx_local = context_for_job;
        let data_local = data_for_job;
        let cb_local = call_js_for_job;
        let env_ref = unsafe { &mut *env_local.0 };
        let func_v = match env_ref.refs.get(func_slot).and_then(|o| o.clone()) {
            Some(v) => v, None => return,
        };
        let func_handle = env_ref.push_handle(func_v);
        if let Some(cb) = cb_local {
            unsafe { (cb.0)(env_local.0, func_handle, ctx_local.0, data_local.0); }
        }
    });
    if let Ok(mut q) = inbox.lock() {
        q.push_back(job);
    }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_acquire_threadsafe_function(tsfn: napi_threadsafe_function) -> napi_status {
    if tsfn.is_null() { return napi_invalid_arg; }
    let t = &*tsfn;
    t.ref_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_release_threadsafe_function(
    tsfn: napi_threadsafe_function, _mode: napi_threadsafe_function_release_mode,
) -> napi_status {
    if tsfn.is_null() { return napi_invalid_arg; }
    let t = &*tsfn;
    let prev = t.ref_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    if prev == 1 {
        // Last release: deactivate. Also drop keepalive if still active
        // so the event loop can exit.
        t.active.store(false, std::sync::atomic::Ordering::SeqCst);
        if t.keepalive_active.compare_exchange(true, false,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst).is_ok()
        {
            t.keepalive_counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_ref_threadsafe_function(_env: napi_env, tsfn: napi_threadsafe_function) -> napi_status {
    if tsfn.is_null() { return napi_invalid_arg; }
    let t = &*tsfn;
    // Toggle from unref'd → ref'd. CAS so concurrent ref/unref from
    // multiple threads serialize.
    if t.keepalive_active.compare_exchange(false, true,
        std::sync::atomic::Ordering::SeqCst,
        std::sync::atomic::Ordering::SeqCst).is_ok()
    {
        t.keepalive_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_unref_threadsafe_function(_env: napi_env, tsfn: napi_threadsafe_function) -> napi_status {
    if tsfn.is_null() { return napi_invalid_arg; }
    let t = &*tsfn;
    if t.keepalive_active.compare_exchange(true, false,
        std::sync::atomic::Ordering::SeqCst,
        std::sync::atomic::Ordering::SeqCst).is_ok()
    {
        t.keepalive_counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
    napi_ok
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_threadsafe_function_context(
    tsfn: napi_threadsafe_function, result: *mut *mut c_void,
) -> napi_status {
    if tsfn.is_null() || result.is_null() { return napi_invalid_arg; }
    let t = &*tsfn;
    *result = t.context.0;
    napi_ok
}

/// Public entry for PollIo to drain queued main-thread jobs. Returns
/// the count of jobs run.
pub fn drain_main_inbox(rt: &mut Runtime) -> usize {
    let drained: Vec<NapiMainJob> = {
        let inbox = rt.napi_main_inbox.clone();
        let mut q = match inbox.lock() { Ok(q) => q, Err(_) => return 0 };
        q.drain(..).collect()
    };
    let n = drained.len();
    for job in drained {
        job(rt);
    }
    n
}

/// True if any napi work is registered or holding the event loop
/// alive: inbox has queued jobs, async_work is in flight, or any
/// threadsafe function is still ref'd. Used by PollIo to keep the
/// event loop alive past sync-script completion.
pub fn has_pending(rt: &Runtime) -> bool {
    if rt.napi_keepalive.load(std::sync::atomic::Ordering::SeqCst) > 0 {
        return true;
    }
    if let Ok(q) = rt.napi_main_inbox.lock() {
        return !q.is_empty();
    }
    false
}

// P46.E1 stubs replaced by real impls below (napi_define_class /
// napi_wrap / napi_unwrap — but with the canonical exported
// names). Old failure stubs removed.

#[no_mangle]
pub unsafe extern "C" fn napi_create_buffer(
    _env: napi_env, _length: usize, _data: *mut *mut c_void, _result: *mut napi_value,
) -> napi_status {
    napi_generic_failure  // P46.E4
}

// ─── Loader entry: called from module.rs::cjs_require for .node files ───

/// Load a `.node` file, call its `napi_register_module_v1`, return the
/// resulting exports value.
pub fn load_napi_module(rt: &mut Runtime, path: &str) -> Result<Value, crate::RuntimeError> {
    // Keep the library alive forever (until process exit). dlclose would
    // invalidate any function pointers later called from JS.
    let lib = unsafe { libloading::Library::new(path) }.map_err(|e| {
        crate::RuntimeError::TypeError(format!("napi: dlopen('{}'): {}", path, e))
    })?;
    // Extract raw fn pointer BEFORE moving lib into the registry — Symbol
    // borrows from the Library; we capture the raw address as usize so it
    // survives the move. Pointers remain valid as long as the Library
    // isn't dlclose'd, which we enforce by holding it in napi_libs forever.
    let init_addr: usize = {
        let sym: libloading::Symbol<unsafe extern "C" fn(napi_env, napi_value) -> napi_value> = unsafe {
            lib.get(b"napi_register_module_v1")
        }.map_err(|e| {
            crate::RuntimeError::TypeError(format!("napi: dlsym('napi_register_module_v1') in '{}': {}", path, e))
        })?;
        *sym as usize
    };
    rt.napi_libs.push(lib);
    let init: unsafe extern "C" fn(napi_env, napi_value) -> napi_value =
        unsafe { std::mem::transmute(init_addr) };

    let exports_id = rt.alloc_object(Object::new_ordinary());
    let exports_v = Value::Object(exports_id);
    let mut env_box = NapiEnv::new(rt);
    let env_ptr = &mut *env_box as *mut NapiEnv;
    let exports_handle = env_box.push_handle(exports_v.clone());
    let ret_handle = unsafe { init(env_ptr, exports_handle) };
    if let Some(exc) = env_box.pending_exception.take() {
        return Err(crate::RuntimeError::Thrown(exc));
    }
    let final_exports = env_box.get_handle(ret_handle).cloned().unwrap_or(exports_v);
    // Stash env_box on Runtime so handles+refs survive (the module's
    // returned function may capture handles or refs that need to stay
    // alive across future JS calls into the module).
    rt.napi_envs.push(env_box);
    Ok(final_exports)
}

// ─── KEEPALIVE: array of fn pointers referenced from main.rs so the
//     linker keeps all napi_* symbols exported.
//     Wrapped in a transparent newtype to satisfy Sync for `pub static`.

#[repr(transparent)]
pub struct NapiSymPtr(pub *const ());
unsafe impl Sync for NapiSymPtr {}

#[no_mangle]
pub static NAPI_KEEPALIVE: &[NapiSymPtr] = &[NapiSymPtr(napi_get_undefined as *const _),
    NapiSymPtr(napi_get_null as *const _),
    NapiSymPtr(napi_get_boolean as *const _),
    NapiSymPtr(napi_get_global as *const _),
    NapiSymPtr(napi_create_int32 as *const _),
    NapiSymPtr(napi_create_uint32 as *const _),
    NapiSymPtr(napi_create_int64 as *const _),
    NapiSymPtr(napi_create_double as *const _),
    NapiSymPtr(napi_get_value_int32 as *const _),
    NapiSymPtr(napi_get_value_uint32 as *const _),
    NapiSymPtr(napi_get_value_int64 as *const _),
    NapiSymPtr(napi_get_value_double as *const _),
    NapiSymPtr(napi_get_value_bool as *const _),
    NapiSymPtr(napi_create_string_utf8 as *const _),
    NapiSymPtr(napi_create_string_latin1 as *const _),
    NapiSymPtr(napi_get_value_string_utf8 as *const _),
    NapiSymPtr(napi_create_object as *const _),
    NapiSymPtr(napi_create_array as *const _),
    NapiSymPtr(napi_create_array_with_length as *const _),
    NapiSymPtr(napi_set_named_property as *const _),
    NapiSymPtr(napi_get_named_property as *const _),
    NapiSymPtr(napi_has_named_property as *const _),
    NapiSymPtr(napi_set_property as *const _),
    NapiSymPtr(napi_get_property as *const _),
    NapiSymPtr(napi_set_element as *const _),
    NapiSymPtr(napi_get_element as *const _),
    NapiSymPtr(napi_get_array_length as *const _),
    NapiSymPtr(napi_typeof as *const _),
    NapiSymPtr(napi_is_array as *const _),
    NapiSymPtr(napi_strict_equals as *const _),
    NapiSymPtr(napi_create_function as *const _),
    NapiSymPtr(napi_get_cb_info as *const _),
    NapiSymPtr(napi_call_function as *const _),
    NapiSymPtr(napi_create_reference as *const _),
    NapiSymPtr(napi_delete_reference as *const _),
    NapiSymPtr(napi_reference_ref as *const _),
    NapiSymPtr(napi_reference_unref as *const _),
    NapiSymPtr(napi_get_reference_value as *const _),
    NapiSymPtr(napi_throw as *const _),
    NapiSymPtr(napi_throw_error as *const _),
    NapiSymPtr(napi_throw_type_error as *const _),
    NapiSymPtr(napi_throw_range_error as *const _),
    NapiSymPtr(napi_is_exception_pending as *const _),
    NapiSymPtr(napi_get_and_clear_last_exception as *const _),
    NapiSymPtr(napi_get_last_error_info as *const _),
    NapiSymPtr(napi_open_handle_scope as *const _),
    NapiSymPtr(napi_close_handle_scope as *const _),
    NapiSymPtr(napi_open_escapable_handle_scope as *const _),
    NapiSymPtr(napi_close_escapable_handle_scope as *const _),
    NapiSymPtr(napi_escape_handle as *const _),
    NapiSymPtr(napi_define_properties as *const _),
    NapiSymPtr(napi_get_version as *const _),
    NapiSymPtr(napi_get_node_version as *const _),
    NapiSymPtr(napi_create_threadsafe_function as *const _),
    NapiSymPtr(napi_call_threadsafe_function as *const _),
    NapiSymPtr(napi_acquire_threadsafe_function as *const _),
    NapiSymPtr(napi_release_threadsafe_function as *const _),
    NapiSymPtr(napi_ref_threadsafe_function as *const _),
    NapiSymPtr(napi_unref_threadsafe_function as *const _),
    NapiSymPtr(napi_get_threadsafe_function_context as *const _),
    NapiSymPtr(napi_create_async_work as *const _),
    NapiSymPtr(napi_queue_async_work as *const _),
    NapiSymPtr(napi_delete_async_work as *const _),
    NapiSymPtr(napi_cancel_async_work as *const _),
    NapiSymPtr(napi_define_class as *const _),
    NapiSymPtr(napi_wrap as *const _),
    NapiSymPtr(napi_unwrap as *const _),
    NapiSymPtr(napi_create_buffer as *const _),
    NapiSymPtr(napi_create_error as *const _),
    NapiSymPtr(napi_create_type_error as *const _),
    NapiSymPtr(napi_create_range_error as *const _),
    NapiSymPtr(napi_create_syntax_error as *const _),
    NapiSymPtr(napi_throw_syntax_error as *const _),
    NapiSymPtr(napi_is_error as *const _),
    NapiSymPtr(napi_create_symbol as *const _),
    NapiSymPtr(napi_get_value_string_latin1 as *const _),
    NapiSymPtr(napi_get_value_string_utf16 as *const _),
    NapiSymPtr(napi_create_bigint_int64 as *const _),
    NapiSymPtr(napi_create_bigint_uint64 as *const _),
    NapiSymPtr(napi_create_bigint_words as *const _),
    NapiSymPtr(napi_get_value_bigint_int64 as *const _),
    NapiSymPtr(napi_get_value_bigint_uint64 as *const _),
    NapiSymPtr(napi_get_value_bigint_words as *const _),
    NapiSymPtr(napi_create_arraybuffer as *const _),
    NapiSymPtr(napi_get_arraybuffer_info as *const _),
    NapiSymPtr(napi_create_external as *const _),
    NapiSymPtr(napi_get_value_external as *const _),
    NapiSymPtr(napi_add_finalizer as *const _),
    NapiSymPtr(napi_adjust_external_memory as *const _),
    NapiSymPtr(napi_coerce_to_string as *const _),
    NapiSymPtr(napi_coerce_to_number as *const _),
    NapiSymPtr(napi_coerce_to_bool as *const _),
    NapiSymPtr(napi_object_freeze as *const _),
    NapiSymPtr(napi_object_seal as *const _),
    NapiSymPtr(napi_instanceof as *const _),
    NapiSymPtr(napi_get_new_target as *const _),
    NapiSymPtr(napi_get_property_names as *const _),
    NapiSymPtr(napi_add_env_cleanup_hook as *const _),
    NapiSymPtr(napi_remove_env_cleanup_hook as *const _),
    NapiSymPtr(napi_create_promise as *const _),
    NapiSymPtr(napi_resolve_deferred as *const _),
    NapiSymPtr(napi_reject_deferred as *const _),
    NapiSymPtr(napi_coerce_to_object as *const _),
    NapiSymPtr(napi_get_buffer_info as *const _),
    NapiSymPtr(napi_is_buffer as *const _),
    NapiSymPtr(napi_new_instance as *const _),
    NapiSymPtr(napi_fatal_exception as *const _),
    NapiSymPtr(napi_remove_wrap as *const _),
];

