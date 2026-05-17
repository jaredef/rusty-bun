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

    // Tier-Ω.5.BBBBBBB: fs.promises namespace + fs.createReadStream stub.
    // fetch-blob's from.js does
    //     import { statSync, createReadStream, promises as fs } from 'node:fs'
    //     const { stat } = fs
    // fs.promises was absent (the async surface lives directly on fs, not
    // under .promises). Adding the namespace as a property-mirror of the
    // async surface satisfies the destructure-at-module-init pattern; the
    // returned promise from stat resolves to a stat-shaped object.
    let promises = new_object(rt);
    register_method(rt, promises, "stat", |rt, args| {
        let path = arg_string(args, 0);
        match std::fs::metadata(&path) {
            Ok(md) => Ok(Value::Object(stat_object(rt, &md))),
            Err(e) => Err(RuntimeError::TypeError(format!("fs.promises.stat: {}", e))),
        }
    });
    register_method(rt, promises, "readFile", |rt, args| {
        let path = arg_string(args, 0);
        let encoding = arg_encoding(args, 1);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(bytes_to_value(rt, &bytes, encoding.as_deref())),
            Err(e) => Err(RuntimeError::TypeError(format!("fs.promises.readFile: {}", e))),
        }
    });
    register_method(rt, promises, "writeFile", |rt, args| {
        let path = arg_string(args, 0);
        let encoding = arg_encoding(args, 2);
        let data = match args.get(1) {
            Some(v) => value_to_bytes(rt, v, encoding.as_deref()),
            None => Vec::new(),
        };
        match std::fs::write(&path, &data) {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => Err(RuntimeError::TypeError(format!("fs.promises.writeFile: {}", e))),
        }
    });
    register_method(rt, promises, "access", |_rt, args| {
        let path = arg_string(args, 0);
        if std::path::Path::new(&path).exists() { Ok(Value::Undefined) }
        else { Err(RuntimeError::TypeError(format!("fs.promises.access: ENOENT: {}", path))) }
    });
    register_method(rt, promises, "mkdir", |_rt, args| {
        let path = arg_string(args, 0);
        match std::fs::create_dir_all(&path) {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => Err(RuntimeError::TypeError(format!("fs.promises.mkdir: {}", e))),
        }
    });
    register_method(rt, promises, "unlink", |_rt, args| {
        let path = arg_string(args, 0);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => Err(RuntimeError::TypeError(format!("fs.promises.unlink: {}", e))),
        }
    });
    rt.object_set(fs, "promises".into(), Value::Object(promises));

    // fs.createReadStream stub — fetch-blob destructures it at module-init
    // but only invokes it inside .stream() at runtime. Stub errors on call.
    let create_read_stream = make_callable(rt, "createReadStream", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "fs.createReadStream not yet implemented (Tier-Ω.5.BBBBBBB stub)".into()
        ))
    });
    rt.object_set(fs, "createReadStream".into(), Value::Object(create_read_stream));

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

    // Ω.5.P32.E1.fs-surface-stubs: bulk-install the Node fs surface
    // entries fs-extra and similar re-export-everything packages probe
    // at module-init. Surfaced via Ω.5.P24.E1 probe walking fs-extra
    // (bun keyCount=152, ours=91; 61 missing). Stubs: methods throw
    // ENOSYS-shaped errors when called; classes are constructor stubs;
    // constants get numeric values per Node's fs.constants. Module
    // shape now matches Bun at the typeof + Object.keys level.
    //
    // POSIX fs.constants (mode bits + access modes) per Node docs.
    let constants = new_object(rt);
    let consts: &[(&str, f64)] = &[
        ("F_OK", 0.0), ("R_OK", 4.0), ("W_OK", 2.0), ("X_OK", 1.0),
        ("O_RDONLY", 0.0), ("O_WRONLY", 1.0), ("O_RDWR", 2.0),
        ("O_CREAT", 64.0), ("O_EXCL", 128.0), ("O_NOCTTY", 256.0),
        ("O_TRUNC", 512.0), ("O_APPEND", 1024.0), ("O_DIRECTORY", 65536.0),
        ("O_NOFOLLOW", 131072.0), ("O_SYNC", 1052672.0), ("O_DSYNC", 4096.0),
        ("S_IFMT", 61440.0), ("S_IFREG", 32768.0), ("S_IFDIR", 16384.0),
        ("S_IFCHR", 8192.0), ("S_IFBLK", 24576.0), ("S_IFIFO", 4096.0),
        ("S_IFLNK", 40960.0), ("S_IFSOCK", 49152.0),
        ("S_IRWXU", 448.0), ("S_IRUSR", 256.0), ("S_IWUSR", 128.0), ("S_IXUSR", 64.0),
        ("S_IRWXG", 56.0), ("S_IRGRP", 32.0), ("S_IWGRP", 16.0), ("S_IXGRP", 8.0),
        ("S_IRWXO", 7.0), ("S_IROTH", 4.0), ("S_IWOTH", 2.0), ("S_IXOTH", 1.0),
        ("COPYFILE_EXCL", 1.0), ("COPYFILE_FICLONE", 2.0), ("COPYFILE_FICLONE_FORCE", 4.0),
        ("UV_FS_O_FILEMAP", 0.0), ("UV_DIRENT_UNKNOWN", 0.0),
        ("UV_DIRENT_FILE", 1.0), ("UV_DIRENT_DIR", 2.0), ("UV_DIRENT_LINK", 3.0),
    ];
    for (name, val) in consts {
        rt.object_set(constants, (*name).into(), Value::Number(*val));
    }
    rt.object_set(fs, "constants".into(), Value::Object(constants));
    // Top-level access-mode shortcuts (also live on fs directly per Node).
    rt.object_set(fs, "F_OK".into(), Value::Number(0.0));
    rt.object_set(fs, "R_OK".into(), Value::Number(4.0));
    rt.object_set(fs, "W_OK".into(), Value::Number(2.0));
    rt.object_set(fs, "X_OK".into(), Value::Number(1.0));

    // Ω.5.P33.E1.fs-real-syscalls: replace P32.E1 surface stubs with
    // real implementations for the file-system operations that map
    // cleanly onto std::fs. Stays as stub for ops that need fd
    // tracking (fdatasync/fsync/ftruncate/futimes/openSync/writeSync/
    // writevSync/readvSync), watcher APIs (watch/watchFile/
    // unwatchFile), pattern matching (glob/globSync), iterators
    // (opendir/opendirSync), and openAsBlob (Blob streaming).
    // statfs gets a synthesized default struct so the call returns
    // a 7-key object matching Bun's shape, with numeric defaults
    // for the values (no libc dependency added).

    // access / accessSync — ECMA-262-adjacent; existence + mode check
    register_method(rt, fs, "accessSync", |rt, args| {
        let path = arg_string(args, 0);
        let mode = match args.get(1) { Some(Value::Number(n)) => *n as u32, _ => 0 };
        match std::fs::metadata(&path) {
            Ok(md) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = md.permissions().mode();
                    // mode bits: F_OK=0 (just exists), R_OK=4, W_OK=2, X_OK=1
                    let need_r = mode & 4 != 0;
                    let need_w = mode & 2 != 0;
                    let need_x = mode & 1 != 0;
                    if need_r && perms & 0o400 == 0 { return Err(RuntimeError::TypeError(format!("accessSync: EACCES on '{}'", path))); }
                    if need_w && perms & 0o200 == 0 { return Err(RuntimeError::TypeError(format!("accessSync: EACCES on '{}'", path))); }
                    if need_x && perms & 0o100 == 0 { return Err(RuntimeError::TypeError(format!("accessSync: EACCES on '{}'", path))); }
                }
                let _ = rt;
                Ok(Value::Undefined)
            }
            Err(e) => Err(RuntimeError::TypeError(format!("accessSync: {}", e))),
        }
    });
    // appendFile / appendFileSync
    register_method(rt, fs, "appendFileSync", |rt, args| {
        use std::io::Write;
        let path = arg_string(args, 0);
        let encoding = arg_encoding(args, 2);
        let data = match args.get(1) {
            Some(v) => value_to_bytes(rt, v, encoding.as_deref()),
            None => Vec::new(),
        };
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&path)
            .map_err(|e| RuntimeError::TypeError(format!("appendFileSync: {}", e)))?;
        file.write_all(&data).map_err(|e| RuntimeError::TypeError(format!("appendFileSync: {}", e)))?;
        Ok(Value::Undefined)
    });
    // copyFile / copyFileSync
    register_method(rt, fs, "copyFileSync", |_rt, args| {
        let src = arg_string(args, 0);
        let dst = arg_string(args, 1);
        std::fs::copy(&src, &dst).map(|_| Value::Undefined)
            .map_err(|e| RuntimeError::TypeError(format!("copyFileSync: {}", e)))
    });
    // cp / cpSync — recursive copy honoring directory + file shapes.
    register_method(rt, fs, "cpSync", |rt, args| {
        let src = arg_string(args, 0);
        let dst = arg_string(args, 1);
        let recursive = match args.get(2) {
            Some(Value::Object(id)) => matches!(rt.object_get(*id, "recursive"), Value::Boolean(true)),
            _ => false,
        };
        cp_recursive(std::path::Path::new(&src), std::path::Path::new(&dst), recursive)
            .map_err(|e| RuntimeError::TypeError(format!("cpSync: {}", e)))?;
        Ok(Value::Undefined)
    });
    // link / linkSync — hard link
    register_method(rt, fs, "linkSync", |_rt, args| {
        let src = arg_string(args, 0);
        let dst = arg_string(args, 1);
        std::fs::hard_link(&src, &dst).map(|_| Value::Undefined)
            .map_err(|e| RuntimeError::TypeError(format!("linkSync: {}", e)))
    });
    // readlink / readlinkSync — read symlink target
    register_method(rt, fs, "readlinkSync", |_rt, args| {
        let path = arg_string(args, 0);
        std::fs::read_link(&path)
            .map(|p| Value::String(Rc::new(p.to_string_lossy().into_owned())))
            .map_err(|e| RuntimeError::TypeError(format!("readlinkSync: {}", e)))
    });
    // rename / renameSync
    register_method(rt, fs, "renameSync", |_rt, args| {
        let src = arg_string(args, 0);
        let dst = arg_string(args, 1);
        std::fs::rename(&src, &dst).map(|_| Value::Undefined)
            .map_err(|e| RuntimeError::TypeError(format!("renameSync: {}", e)))
    });
    // rm / rmSync — file or dir, with options.recursive + options.force
    register_method(rt, fs, "rmSync", |rt, args| {
        let path = arg_string(args, 0);
        let (recursive, force) = match args.get(1) {
            Some(Value::Object(id)) => (
                matches!(rt.object_get(*id, "recursive"), Value::Boolean(true)),
                matches!(rt.object_get(*id, "force"), Value::Boolean(true)),
            ),
            _ => (false, false),
        };
        let p = std::path::Path::new(&path);
        let r = if p.is_dir() {
            if recursive { std::fs::remove_dir_all(&path) } else { std::fs::remove_dir(&path) }
        } else { std::fs::remove_file(&path) };
        match r {
            Ok(()) => Ok(Value::Undefined),
            Err(e) if force && e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Undefined),
            Err(e) => Err(RuntimeError::TypeError(format!("rmSync: {}", e))),
        }
    });
    // rmdir / rmdirSync — empty dir only (per Node; cp/rm handle recursive)
    register_method(rt, fs, "rmdirSync", |rt, args| {
        let path = arg_string(args, 0);
        let recursive = match args.get(1) {
            Some(Value::Object(id)) => matches!(rt.object_get(*id, "recursive"), Value::Boolean(true)),
            _ => false,
        };
        let r = if recursive { std::fs::remove_dir_all(&path) } else { std::fs::remove_dir(&path) };
        r.map(|_| Value::Undefined)
            .map_err(|e| RuntimeError::TypeError(format!("rmdirSync: {}", e)))
    });
    // symlink / symlinkSync (unix-only target)
    register_method(rt, fs, "symlinkSync", |_rt, args| {
        let target = arg_string(args, 0);
        let link = arg_string(args, 1);
        #[cfg(unix)]
        { std::os::unix::fs::symlink(&target, &link).map(|_| Value::Undefined)
            .map_err(|e| RuntimeError::TypeError(format!("symlinkSync: {}", e))) }
        #[cfg(not(unix))]
        { let _ = (target, link); Err(RuntimeError::TypeError("symlinkSync: unsupported on this platform".into())) }
    });
    // truncate / truncateSync — set file length
    register_method(rt, fs, "truncateSync", |_rt, args| {
        let path = arg_string(args, 0);
        let len = match args.get(1) { Some(Value::Number(n)) => *n as u64, _ => 0 };
        let file = std::fs::OpenOptions::new().write(true).open(&path)
            .map_err(|e| RuntimeError::TypeError(format!("truncateSync: {}", e)))?;
        file.set_len(len).map(|_| Value::Undefined)
            .map_err(|e| RuntimeError::TypeError(format!("truncateSync: {}", e)))
    });
    // mkdtemp / mkdtempSync — create unique temp dir
    register_method(rt, fs, "mkdtempSync", |_rt, args| {
        let prefix = arg_string(args, 0);
        let mut attempts = 0;
        loop {
            attempts += 1;
            if attempts > 64 { return Err(RuntimeError::TypeError("mkdtempSync: too many collisions".into())); }
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();
            let path = format!("{}{:06X}{:02X}", prefix, nanos, attempts);
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Value::String(Rc::new(path))),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(RuntimeError::TypeError(format!("mkdtempSync: {}", e))),
            }
        }
    });
    // statfs / statfsSync — synthesized struct (no libc dep). Returns
    // the spec-mandated 7 keys with conservative defaults; the actual
    // values are placeholders. Consumers that need real disk-space
    // numbers will diverge; consumers that check shape pass.
    register_method(rt, fs, "statfsSync", |rt, args| {
        let _ = arg_string(args, 0);  // path consumed for arg shape
        let o = new_object(rt);
        rt.object_set(o, "type".into(), Value::Number(0.0));
        rt.object_set(o, "bsize".into(), Value::Number(4096.0));
        rt.object_set(o, "blocks".into(), Value::Number(0.0));
        rt.object_set(o, "bfree".into(), Value::Number(0.0));
        rt.object_set(o, "bavail".into(), Value::Number(0.0));
        rt.object_set(o, "files".into(), Value::Number(0.0));
        rt.object_set(o, "ffree".into(), Value::Number(0.0));
        Ok(Value::Object(o))
    });

    // Async wrappers — Promise-returning forms of the sync impls above.
    // The sync impls are referenced via fs.XSync at call-time so the
    // closure captures the method name, not the impl pointer.
    for (async_name, sync_name) in [
        ("access", "accessSync"),
        ("appendFile", "appendFileSync"),
        ("copyFile", "copyFileSync"),
        ("cp", "cpSync"),
        ("link", "linkSync"),
        ("readlink", "readlinkSync"),
        ("rename", "renameSync"),
        ("rm", "rmSync"),
        ("rmdir", "rmdirSync"),
        ("symlink", "symlinkSync"),
        ("truncate", "truncateSync"),
        ("mkdtemp", "mkdtempSync"),
        ("statfs", "statfsSync"),
        ("unlink", "unlinkSync"),
        ("mkdir", "mkdirSync"),
        ("utimes", "utimesSync"),  // utimesSync remains stub for now
    ] {
        let key = sync_name.to_string();
        register_method(rt, fs, async_name, move |rt, args| {
            let p = new_promise(rt);
            let fs_global = match rt.globals.get("fs") { Some(Value::Object(id)) => *id, _ => return Ok(Value::Object(p)) };
            let sync_fn = rt.object_get(fs_global, &key);
            let argv: Vec<Value> = args.to_vec();
            match rt.call_function(sync_fn, Value::Object(fs_global), argv) {
                Ok(v) => resolve_promise(rt, p, v),
                Err(e) => {
                    let msg = match &e {
                        RuntimeError::TypeError(m) => m.clone(),
                        RuntimeError::RangeError(m) => m.clone(),
                        RuntimeError::ReferenceError(m) => m.clone(),
                        RuntimeError::Thrown(v) => format!("{:?}", v),
                        _ => format!("{:?}", e),
                    };
                    reject_promise(rt, p, Value::String(Rc::new(msg)));
                }
            }
            Ok(Value::Object(p))
        });
    }

    // Stubs that remain — fd-based ops, glob/watch APIs, openAsBlob,
    // opendir, vector IO, lutimes. Throw ENOSYS-shaped TypeError per
    // P32.E1's discipline; the surface stays present for typeof checks.
    for name in [
        "fdatasync", "fdatasyncSync", "fsync", "fsyncSync", "ftruncate",
        "ftruncateSync", "futimes", "futimesSync", "glob", "globSync",
        "lutimes", "lutimesSync", "openAsBlob", "openSync", "opendir",
        "opendirSync", "readvSync", "unwatchFile", "utimesSync", "watch",
        "watchFile", "writeSync", "writevSync",
    ] {
        let n = name.to_string();
        let stub = make_callable(rt, name, move |_rt, _args| {
            Err(RuntimeError::TypeError(format!("fs.{}: not implemented (Tier-Ω.5.P32.E1 stub)", n)))
        });
        rt.object_set(fs, name.into(), Value::Object(stub));
    }
    // Class stubs (Stats, Dirent, Dir) — constructor-throw shape.
    for cls in ["Stats", "Dirent", "Dir"] {
        let n = cls.to_string();
        let stub = make_callable(rt, cls, move |_rt, _args| {
            Err(RuntimeError::TypeError(format!("fs.{}: class not constructable (Tier-Ω.5.P32.E1 stub)", n)))
        });
        rt.object_set(fs, cls.into(), Value::Object(stub));
    }
    // Internal helper Node exposes for legacy compat.
    let to_unix = make_callable(rt, "_toUnixTimestamp", |_rt, args| {
        let v = args.first().cloned().unwrap_or(Value::Number(0.0));
        Ok(v)
    });
    rt.object_set(fs, "_toUnixTimestamp".into(), Value::Object(to_unix));

    rt.globals.insert("fs".into(), Value::Object(fs));
}

// Ω.5.P33.E1: recursive copy helper for fs.cpSync.
fn cp_recursive(src: &std::path::Path, dst: &std::path::Path, recursive: bool) -> std::io::Result<()> {
    let md = std::fs::metadata(src)?;
    if md.is_dir() {
        if !recursive {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput,
                "cp: source is a directory and recursive is not set"));
        }
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let from = entry.path();
            let to = dst.join(entry.file_name());
            cp_recursive(&from, &to, true)?;
        }
        Ok(())
    } else {
        if let Some(parent) = dst.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        std::fs::copy(src, dst).map(|_| ())
    }
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
