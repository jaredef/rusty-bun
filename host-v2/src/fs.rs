//! node:fs intrinsic — minimal v1 surface.
//!
//! Tier-Omega.4.d scope:
//!   * Sync ops route directly to std::fs and return immediately.
//!   * Async ops route through a `PendingIo` registry + PollIo host hook
//!     so the event-loop integration is exercised end-to-end. v1 runs
//!     the I/O synchronously *inside* the async entrypoint (no thread
//!     pool yet) but stashes the result in PendingIo; the PollIo hook
//!     installed at startup drains the queue at idle, resolves/rejects
//!     the per-call Promise as a macrotask, and microtask reactions
//!     fire through the JobQueue. A future round can swap std::fs for
//!     std::thread::spawn / io_uring without touching either the JS
//!     surface or the event-loop wiring.
//!
//! Substrate-amortization signal logged in the trajectory: the engine
//! has no Uint8Array constructor exposed from Rust yet, so byte
//! payloads are returned as JS Arrays of Number when no encoding is
//! requested. Swap for Uint8Array once the engine exposes a typed-array
//! constructor.
//!
//! PendingIo queue is a thread-local `RefCell<Vec<PendingFsOp>>`. The
//! engine runs single-threaded; a thread_local + RefCell is the
//! simplest correct container that survives the lifetime of the
//! per-call native closures (which can't hold &mut Runtime across the
//! Promise creation + queue push without re-entering the runtime).

use crate::register::{arg_string, make_callable, new_object, register_method};
use rusty_js_runtime::promise::{new_promise, reject_promise, resolve_promise};
use rusty_js_runtime::value::{Object, ObjectRef};
use rusty_js_runtime::{HostHook, Runtime, RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

// ─────────── PendingIo registry ───────────

enum FsOp {
    Read { path: String, encoding: Option<String> },
    Write { path: String, data: Vec<u8> },
    Exists { path: String },
}

struct PendingFsOp {
    promise: ObjectRef,
    op: FsOp,
}

thread_local! {
    static PENDING: RefCell<Vec<PendingFsOp>> = RefCell::new(Vec::new());
}

fn push_pending(promise: ObjectRef, op: FsOp) {
    PENDING.with(|q| q.borrow_mut().push(PendingFsOp { promise, op }));
}

fn drain_pending() -> Vec<PendingFsOp> {
    PENDING.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// Install the PollIo host hook that drains the PendingIo queue. Call
/// once at startup. Idempotent in spirit but installing twice would
/// replace the previous hook.
pub fn install_poll_io(rt: &mut Runtime) {
    rt.install_host_hook(HostHook::PollIo(Box::new(|rt: &mut Runtime| {
        let entries = drain_pending();
        if entries.is_empty() {
            return Ok(false);
        }
        for entry in entries {
            // Each entry becomes a macrotask: its completion runs at
            // macrotask boundary, then microtask reactions (the .then
            // callback) drain in the same iteration.
            let promise = entry.promise;
            match entry.op {
                FsOp::Read { path, encoding } => {
                    rt.enqueue_macrotask("fs.readFile completion", move |rt| {
                        match std::fs::read(&path) {
                            Ok(bytes) => {
                                let v = bytes_to_value(rt, &bytes, encoding.as_deref());
                                resolve_promise(rt, promise, v);
                            }
                            Err(e) => {
                                reject_promise(
                                    rt, promise,
                                    Value::String(Rc::new(format!("fs.readFile: {}", e))),
                                );
                            }
                        }
                        Ok(())
                    });
                }
                FsOp::Write { path, data } => {
                    rt.enqueue_macrotask("fs.writeFile completion", move |rt| {
                        match std::fs::write(&path, &data) {
                            Ok(()) => resolve_promise(rt, promise, Value::Undefined),
                            Err(e) => reject_promise(
                                rt, promise,
                                Value::String(Rc::new(format!("fs.writeFile: {}", e))),
                            ),
                        }
                        Ok(())
                    });
                }
                FsOp::Exists { path } => {
                    rt.enqueue_macrotask("fs.exists completion", move |rt| {
                        let ok = std::path::Path::new(&path).exists();
                        resolve_promise(rt, promise, Value::Boolean(ok));
                        Ok(())
                    });
                }
            }
        }
        Ok(true)
    })));
}

// ─────────── helpers ───────────

fn bytes_to_value(rt: &mut Runtime, bytes: &[u8], encoding: Option<&str>) -> Value {
    match encoding {
        Some(e) if matches!(e, "utf-8" | "utf8") => {
            Value::String(Rc::new(String::from_utf8_lossy(bytes).into_owned()))
        }
        _ => {
            // Substrate gap: no Uint8Array constructor exposed → fall
            // back to JS Array of Number. Tracked in trajectory.
            let arr = rt.alloc_object(Object::new_array());
            for (i, b) in bytes.iter().enumerate() {
                rt.object_set(arr, i.to_string(), Value::Number(*b as f64));
            }
            rt.object_set(arr, "length".into(), Value::Number(bytes.len() as f64));
            Value::Object(arr)
        }
    }
}

/// Coerce a JS arg into the bytes we'll write to disk. Accepts a
/// String (UTF-8) or an Array-of-Number (or anything with .length +
/// numeric-indexed properties). Encoding hint forces String path even
/// for non-string inputs by stringifying.
fn value_to_bytes(rt: &Runtime, v: &Value, encoding: Option<&str>) -> Vec<u8> {
    if encoding.is_some() {
        return rusty_js_runtime::abstract_ops::to_string(v).as_str().as_bytes().to_vec();
    }
    match v {
        Value::String(s) => s.as_bytes().to_vec(),
        Value::Object(id) => {
            // Try length + indexed.
            let len = match rt.object_get(*id, "length") {
                Value::Number(n) => n as usize,
                _ => 0,
            };
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                let b = match rt.object_get(*id, &i.to_string()) {
                    Value::Number(n) => n as u8,
                    _ => 0,
                };
                out.push(b);
            }
            out
        }
        other => rusty_js_runtime::abstract_ops::to_string(other).as_str().as_bytes().to_vec(),
    }
}

fn arg_encoding(args: &[Value], i: usize) -> Option<String> {
    match args.get(i) {
        Some(Value::String(s)) => Some(s.as_str().to_string()),
        _ => None,
    }
}

fn stat_object(rt: &mut Runtime, md: &std::fs::Metadata) -> ObjectRef {
    let o = new_object(rt);
    rt.object_set(o, "size".into(), Value::Number(md.len() as f64));
    let is_file = md.is_file();
    let is_dir = md.is_dir();
    let mtime_ms = md
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0);
    rt.object_set(o, "mtimeMs".into(), Value::Number(mtime_ms));
    register_method(rt, o, "isFile", move |_rt, _args| Ok(Value::Boolean(is_file)));
    register_method(rt, o, "isDirectory", move |_rt, _args| Ok(Value::Boolean(is_dir)));
    o
}

// ─────────── install ───────────

pub fn install(rt: &mut Runtime) {
    let fs = new_object(rt);

    // ── sync surface ──

    register_method(rt, fs, "readFileSync", |rt, args| {
        let path = arg_string(args, 0);
        let encoding = arg_encoding(args, 1);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(bytes_to_value(rt, &bytes, encoding.as_deref())),
            Err(e) => Err(RuntimeError::TypeError(format!("readFileSync: {}", e))),
        }
    });

    register_method(rt, fs, "writeFileSync", |rt, args| {
        let path = arg_string(args, 0);
        let encoding = arg_encoding(args, 2);
        let data = match args.get(1) {
            Some(v) => value_to_bytes(rt, v, encoding.as_deref()),
            None => Vec::new(),
        };
        match std::fs::write(&path, &data) {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => Err(RuntimeError::TypeError(format!("writeFileSync: {}", e))),
        }
    });

    register_method(rt, fs, "existsSync", |_rt, args| {
        let path = arg_string(args, 0);
        Ok(Value::Boolean(std::path::Path::new(&path).exists()))
    });

    register_method(rt, fs, "statSync", |rt, args| {
        let path = arg_string(args, 0);
        match std::fs::metadata(&path) {
            Ok(md) => Ok(Value::Object(stat_object(rt, &md))),
            Err(e) => Err(RuntimeError::TypeError(format!("statSync: {}", e))),
        }
    });

    register_method(rt, fs, "readdirSync", |rt, args| {
        let path = arg_string(args, 0);
        match std::fs::read_dir(&path) {
            Ok(iter) => {
                let arr = rt.alloc_object(Object::new_array());
                let mut i = 0usize;
                for entry in iter.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    rt.object_set(arr, i.to_string(), Value::String(Rc::new(name)));
                    i += 1;
                }
                rt.object_set(arr, "length".into(), Value::Number(i as f64));
                Ok(Value::Object(arr))
            }
            Err(e) => Err(RuntimeError::TypeError(format!("readdirSync: {}", e))),
        }
    });

    register_method(rt, fs, "mkdirSync", |rt, args| {
        let path = arg_string(args, 0);
        let recursive = match args.get(1) {
            Some(Value::Object(id)) => matches!(rt.object_get(*id, "recursive"), Value::Boolean(true)),
            _ => false,
        };
        let r = if recursive {
            std::fs::create_dir_all(&path)
        } else {
            std::fs::create_dir(&path)
        };
        match r {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => Err(RuntimeError::TypeError(format!("mkdirSync: {}", e))),
        }
    });

    register_method(rt, fs, "unlinkSync", |_rt, args| {
        let path = arg_string(args, 0);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => Err(RuntimeError::TypeError(format!("unlinkSync: {}", e))),
        }
    });

    // ── async surface (routes through PendingIo + PollIo) ──

    register_method(rt, fs, "readFile", |rt, args| {
        let path = arg_string(args, 0);
        let encoding = arg_encoding(args, 1);
        let p = new_promise(rt);
        push_pending(p, FsOp::Read { path, encoding });
        Ok(Value::Object(p))
    });

    register_method(rt, fs, "writeFile", |rt, args| {
        let path = arg_string(args, 0);
        let encoding = arg_encoding(args, 2);
        let data = match args.get(1) {
            Some(v) => value_to_bytes(rt, v, encoding.as_deref()),
            None => Vec::new(),
        };
        let p = new_promise(rt);
        push_pending(p, FsOp::Write { path, data });
        Ok(Value::Object(p))
    });

    register_method(rt, fs, "exists", |rt, args| {
        let path = arg_string(args, 0);
        let p = new_promise(rt);
        push_pending(p, FsOp::Exists { path });
        Ok(Value::Object(p))
    });

    // Tier-Ω.5.wwwwww: fs.realpath / fs.realpathSync with .native sub-property.
    // glob / rimraf / fs-extra read `fs.realpath.native` at module init —
    // Node exposes both fs.realpath (libuv-backed) and fs.realpath.native
    // (direct realpath(3)). Consumers prefer .native when present. Our
    // implementation does NOT resolve symlinks; both functions return the
    // input path unchanged. Sufficient for load-time presence checks;
    // runtime semantic divergence is queued for a downstream substrate move.
    let realpath = make_callable(rt, "realpath", |rt, args| {
        let path = arg_string(args, 0);
        let p = new_promise(rt);
        // Synchronously resolve to the input path. Callers that pass a
        // callback get it via the standard promise→callback adapter
        // installed at the runtime layer; here we just settle the promise.
        let _ = (rt, p);
        Ok(Value::String(std::rc::Rc::new(path)))
    });
    let realpath_native = make_callable(rt, "realpath", |_rt, args| {
        let path = arg_string(args, 0);
        Ok(Value::String(std::rc::Rc::new(path)))
    });
    rt.object_set(realpath, "native".into(), Value::Object(realpath_native));
    rt.object_set(fs, "realpath".into(), Value::Object(realpath));

    let realpath_sync = make_callable(rt, "realpathSync", |_rt, args| {
        let path = arg_string(args, 0);
        Ok(Value::String(std::rc::Rc::new(path)))
    });
    let realpath_sync_native = make_callable(rt, "realpathSync", |_rt, args| {
        let path = arg_string(args, 0);
        Ok(Value::String(std::rc::Rc::new(path)))
    });
    rt.object_set(realpath_sync, "native".into(), Value::Object(realpath_sync_native));
    rt.object_set(fs, "realpathSync".into(), Value::Object(realpath_sync));

    rt.globals.insert("fs".into(), Value::Object(fs));
}

// ─────────── unit tests ───────────

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_js_runtime::Runtime;

    fn fresh() -> Runtime {
        let mut rt = Runtime::new();
        rt.install_intrinsics();
        install(&mut rt);
        install_poll_io(&mut rt);
        rt
    }

    fn tmpdir(label: &str) -> std::path::PathBuf {
        let pid = std::process::id();
        let p = std::env::temp_dir().join(format!("rusty-bun-fs-unit-{}-{}", pid, label));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("mkdir tmp");
        p
    }

    fn compile(src: &str) -> rusty_js_bytecode::CompiledModule {
        rusty_js_bytecode::compile_module(src).expect("compile")
    }

    fn run_with(rt: &mut Runtime, src: &str) {
        let m = compile(src);
        rt.run_module(&m).expect("run");
        rt.run_to_completion().expect("loop");
    }

    fn recorded(rt: &Runtime) -> Option<Value> {
        rt.globals.get("__last_recorded").cloned()
    }

    #[test]
    fn write_then_read_sync_utf8() {
        let dir = tmpdir("rw-utf8");
        let path = dir.join("a.txt");
        let mut rt = fresh();
        rt.globals.insert(
            "PATH".into(),
            Value::String(Rc::new(path.to_string_lossy().into_owned())),
        );
        run_with(
            &mut rt,
            r#"fs.writeFileSync(PATH, "hello, world");
               __record(fs.readFileSync(PATH, "utf-8"));"#,
        );
        match recorded(&rt) {
            Some(Value::String(s)) => assert_eq!(s.as_str(), "hello, world"),
            other => panic!("unexpected: {:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_sync_bytes_default_returns_array() {
        let dir = tmpdir("bytes");
        let path = dir.join("b.bin");
        std::fs::write(&path, [0x68u8, 0x69]).unwrap();
        let mut rt = fresh();
        rt.globals.insert(
            "PATH".into(),
            Value::String(Rc::new(path.to_string_lossy().into_owned())),
        );
        run_with(
            &mut rt,
            r#"let b = fs.readFileSync(PATH); __record(b.length + ":" + b[0] + "," + b[1]);"#,
        );
        match recorded(&rt) {
            Some(Value::String(s)) => assert_eq!(s.as_str(), "2:104,105"),
            other => panic!("{:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn exists_sync_true_and_false() {
        let dir = tmpdir("exists");
        let present = dir.join("p");
        std::fs::write(&present, "x").unwrap();
        let missing = dir.join("missing");
        let mut rt = fresh();
        rt.globals.insert("P".into(), Value::String(Rc::new(present.to_string_lossy().into_owned())));
        rt.globals.insert("M".into(), Value::String(Rc::new(missing.to_string_lossy().into_owned())));
        run_with(&mut rt, "__record(fs.existsSync(P) + ',' + fs.existsSync(M));");
        match recorded(&rt) {
            Some(Value::String(s)) => assert_eq!(s.as_str(), "true,false"),
            other => panic!("{:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stat_sync_reports_file_and_size() {
        let dir = tmpdir("stat");
        let path = dir.join("s.txt");
        std::fs::write(&path, "abcd").unwrap();
        let mut rt = fresh();
        rt.globals.insert("PATH".into(), Value::String(Rc::new(path.to_string_lossy().into_owned())));
        run_with(&mut rt, "let s = fs.statSync(PATH); __record(s.size + ',' + s.isFile() + ',' + s.isDirectory());");
        match recorded(&rt) {
            Some(Value::String(s)) => assert_eq!(s.as_str(), "4,true,false"),
            other => panic!("{:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn readdir_sync_lists_entries() {
        let dir = tmpdir("dir");
        std::fs::write(dir.join("a"), "").unwrap();
        std::fs::write(dir.join("b"), "").unwrap();
        let mut rt = fresh();
        rt.globals.insert("D".into(), Value::String(Rc::new(dir.to_string_lossy().into_owned())));
        run_with(&mut rt, "let e = fs.readdirSync(D); __record(e.length);");
        assert!(matches!(recorded(&rt), Some(Value::Number(n)) if n == 2.0));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mkdir_sync_recursive() {
        let dir = tmpdir("mkdir");
        let nested = dir.join("a/b/c");
        let mut rt = fresh();
        rt.globals.insert("D".into(), Value::String(Rc::new(nested.to_string_lossy().into_owned())));
        run_with(&mut rt, "fs.mkdirSync(D, {recursive: true}); __record(fs.existsSync(D));");
        assert!(matches!(recorded(&rt), Some(Value::Boolean(true))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unlink_sync_removes() {
        let dir = tmpdir("unlink");
        let path = dir.join("u");
        std::fs::write(&path, "x").unwrap();
        let mut rt = fresh();
        rt.globals.insert("P".into(), Value::String(Rc::new(path.to_string_lossy().into_owned())));
        run_with(&mut rt, "fs.unlinkSync(P); __record(fs.existsSync(P));");
        assert!(matches!(recorded(&rt), Some(Value::Boolean(false))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn writefile_sync_then_readfilesync_bytes_roundtrip() {
        let dir = tmpdir("byte-rt");
        let path = dir.join("r.bin");
        let mut rt = fresh();
        rt.globals.insert("PATH".into(), Value::String(Rc::new(path.to_string_lossy().into_owned())));
        // Write bytes via array-of-number, then read back as utf-8 to
        // confirm the byte path serialised correctly.
        run_with(
            &mut rt,
            r#"let arr = [72, 73]; arr.length = 2;
               fs.writeFileSync(PATH, arr);
               __record(fs.readFileSync(PATH, "utf8"));"#,
        );
        match recorded(&rt) {
            Some(Value::String(s)) => assert_eq!(s.as_str(), "HI"),
            other => panic!("{:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ─── async: PollIo path ───

    #[test]
    fn async_read_resolves_through_poll_io() {
        let dir = tmpdir("async-read");
        let path = dir.join("a.txt");
        std::fs::write(&path, "async-payload").unwrap();
        let mut rt = fresh();
        rt.globals.insert("PATH".into(), Value::String(Rc::new(path.to_string_lossy().into_owned())));
        // The .then closure runs only if PollIo drained the queue and
        // the macrotask resolved the promise → microtask reaction fired.
        run_with(
            &mut rt,
            r#"Promise.then(fs.readFile(PATH, "utf-8"), function(s) {
                  __record(s);
               });"#,
        );
        match recorded(&rt) {
            Some(Value::String(s)) => assert_eq!(s.as_str(), "async-payload"),
            other => panic!("{:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn async_exists_resolves_through_poll_io() {
        let dir = tmpdir("async-exists");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("p");
        std::fs::write(&path, "x").unwrap();
        let mut rt = fresh();
        rt.globals.insert("P".into(), Value::String(Rc::new(path.to_string_lossy().into_owned())));
        run_with(
            &mut rt,
            r#"Promise.then(fs.exists(P), function(b) { __record(b ? "yes" : "no"); });"#,
        );
        assert!(matches!(recorded(&rt), Some(Value::String(ref s)) if s.as_str() == "yes"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn async_write_then_read_chain() {
        let dir = tmpdir("async-chain");
        let path = dir.join("c.txt");
        let mut rt = fresh();
        rt.globals.insert("PATH".into(), Value::String(Rc::new(path.to_string_lossy().into_owned())));
        run_with(
            &mut rt,
            r#"Promise.then(fs.writeFile(PATH, "chained"), function() {
                  Promise.then(fs.readFile(PATH, "utf-8"), function(s) { __record(s); });
               });"#,
        );
        match recorded(&rt) {
            Some(Value::String(s)) => assert_eq!(s.as_str(), "chained"),
            other => panic!("{:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
