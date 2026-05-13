// Π2.6.d.b — child-process pipes via the mio reactor.
//
// Wraps std::process::Command with a handle registry. Each spawn
// returns a handle id; the caller can fetch raw pipe fds for
// reactor registration, nonblocking-read stdout/stderr, write
// stdin, nonblocking-wait for exit, and kill.
//
// Per-thread registry (cargo test parallelism). Each Rust test
// thread gets its own ProcessRegistry, isolated from the others.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};

pub struct ProcessHandle {
    pub child: Child,
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
    pub exit_code: Option<i32>,
}

thread_local! {
    static REGISTRY: RefCell<HashMap<u32, ProcessHandle>> = RefCell::new(HashMap::new());
    static NEXT_ID: RefCell<u32> = const { RefCell::new(1) };
}

fn next_id() -> u32 {
    NEXT_ID.with(|n| {
        let mut n = n.borrow_mut();
        let id = *n;
        *n = n.wrapping_add(1);
        id
    })
}

pub fn spawn_async(args: &[String], env: Option<Vec<(String, String)>>, cwd: Option<String>) -> Result<u32, String> {
    if args.is_empty() { return Err("spawn_async: empty args".to_string()); }
    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    if let Some(cwd) = cwd { cmd.current_dir(cwd); }
    if let Some(env) = env {
        cmd.env_clear();
        for (k, v) in env { cmd.env(k, v); }
    }
    let mut child = cmd.spawn().map_err(|e| format!("spawn: {}", e))?;
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    // Set pipes nonblocking so try_read_pipe surfaces WouldBlock
    // instead of blocking the eval loop.
    #[cfg(unix)] {
        use std::os::unix::io::AsRawFd;
        unsafe fn set_nonblocking(fd: std::os::unix::io::RawFd) {
            let flags = libc::fcntl(fd, libc::F_GETFL, 0);
            if flags >= 0 {
                libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
            }
        }
        unsafe {
            if let Some(ref s) = stdout { set_nonblocking(s.as_raw_fd()); }
            if let Some(ref s) = stderr { set_nonblocking(s.as_raw_fd()); }
        }
    }
    let id = next_id();
    REGISTRY.with(|r| {
        r.borrow_mut().insert(id, ProcessHandle {
            child, stdin, stdout, stderr, exit_code: None,
        });
    });
    Ok(id)
}

pub fn pid(id: u32) -> Result<u32, String> {
    REGISTRY.with(|r| {
        let r = r.borrow();
        let h = r.get(&id).ok_or("spawn_async: invalid handle")?;
        Ok(h.child.id())
    })
}

#[cfg(unix)]
pub fn pipe_fd(id: u32, which: u32) -> Result<std::os::unix::io::RawFd, String> {
    use std::os::unix::io::AsRawFd;
    REGISTRY.with(|r| {
        let r = r.borrow();
        let h = r.get(&id).ok_or("spawn_async: invalid handle")?;
        match which {
            0 => h.stdin.as_ref().map(|s| s.as_raw_fd()).ok_or("stdin not piped".to_string()),
            1 => h.stdout.as_ref().map(|s| s.as_raw_fd()).ok_or("stdout not piped".to_string()),
            2 => h.stderr.as_ref().map(|s| s.as_raw_fd()).ok_or("stderr not piped".to_string()),
            _ => Err(format!("invalid which: {}", which)),
        }
    })
}

/// Nonblocking pipe read. Returns:
///   Ok(Some(bytes)) — data (possibly empty Vec on EOF)
///   Ok(None) — WouldBlock
pub fn try_read_pipe(id: u32, which: u32, max: usize) -> Result<Option<Vec<u8>>, String> {
    REGISTRY.with(|r| {
        let mut r = r.borrow_mut();
        let h = r.get_mut(&id).ok_or("spawn_async: invalid handle")?;
        let mut buf = vec![0u8; max];
        let read_result = match which {
            1 => h.stdout.as_mut().map(|s| s.read(&mut buf)),
            2 => h.stderr.as_mut().map(|s| s.read(&mut buf)),
            _ => return Err(format!("try_read_pipe: invalid which: {}", which)),
        };
        match read_result {
            Some(Ok(n)) => { buf.truncate(n); Ok(Some(buf)) }
            Some(Err(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Some(Err(e)) => Err(format!("read pipe: {}", e)),
            None => Err("pipe already closed".to_string()),
        }
    })
}

/// Write to child stdin. Returns bytes written (may be short).
pub fn write_pipe(id: u32, data: &[u8]) -> Result<usize, String> {
    REGISTRY.with(|r| {
        let mut r = r.borrow_mut();
        let h = r.get_mut(&id).ok_or("spawn_async: invalid handle")?;
        match h.stdin.as_mut() {
            Some(s) => s.write(data).map_err(|e| format!("write stdin: {}", e)),
            None => Err("stdin closed".to_string()),
        }
    })
}

/// Close a pipe (typically stdin so the child sees EOF).
pub fn close_pipe(id: u32, which: u32) -> Result<(), String> {
    REGISTRY.with(|r| {
        let mut r = r.borrow_mut();
        let h = r.get_mut(&id).ok_or("spawn_async: invalid handle")?;
        match which {
            0 => { h.stdin = None; Ok(()) }
            1 => { h.stdout = None; Ok(()) }
            2 => { h.stderr = None; Ok(()) }
            _ => Err(format!("close_pipe: invalid which: {}", which)),
        }
    })
}

/// Nonblocking waitpid via std::Child::try_wait.
/// Returns Ok(Some(exit_code)) once the child exits, Ok(None) while running.
pub fn try_wait(id: u32) -> Result<Option<i32>, String> {
    REGISTRY.with(|r| {
        let mut r = r.borrow_mut();
        let h = r.get_mut(&id).ok_or("spawn_async: invalid handle")?;
        if let Some(c) = h.exit_code { return Ok(Some(c)); }
        match h.child.try_wait() {
            Ok(Some(status)) => {
                let c = status.code().unwrap_or(-1);
                h.exit_code = Some(c);
                Ok(Some(c))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("try_wait: {}", e)),
        }
    })
}

pub fn kill(id: u32) -> Result<(), String> {
    REGISTRY.with(|r| {
        let mut r = r.borrow_mut();
        let h = r.get_mut(&id).ok_or("spawn_async: invalid handle")?;
        h.child.kill().map_err(|e| format!("kill: {}", e))
    })
}

pub fn drop_handle(id: u32) -> Result<(), String> {
    REGISTRY.with(|r| {
        let mut r = r.borrow_mut();
        // Drop drops child; if still running it becomes a zombie until
        // the OS reaps. For test fixtures this is fine.
        r.remove(&id);
        Ok(())
    })
}
