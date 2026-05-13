// Π2.6.d.a — file watcher substrate via inotify (Linux).
//
// Pure-libc wrapper, no inotify crate dependency. Each Watcher owns
// an inotify fd; add_watch returns a watch descriptor (wd); read_events
// returns the queued event tuples since the last call. The inotify
// fd is set nonblocking so read returns WouldBlock when empty; the
// reactor signals readability when new events arrive.
//
// Scope: directory + file watch (IN_MODIFY | IN_CREATE | IN_DELETE
// | IN_MOVED_FROM | IN_MOVED_TO | IN_CLOSE_WRITE), event name decoded
// from the trailing NUL-terminated filename. Recursive watch is the
// caller's job (walk the directory + add_watch each subdir).
//
// macOS path uses kqueue (out of scope this round; non-Linux unix
// platforms fall back to the no-op stub).

#[cfg(target_os = "linux")]
pub mod inotify {
    use std::collections::HashMap;
    use std::os::unix::io::RawFd;
    use std::sync::Mutex;

    pub struct Watcher {
        pub fd: RawFd,
        pub watches: HashMap<i32, String>,  // wd -> path
    }

    pub struct WatchEvent {
        pub wd: i32,
        pub mask: u32,
        pub name: String,  // filename within the watched dir; "" for file-target
    }

    static REGISTRY: Mutex<Option<HashMap<u64, Watcher>>> = Mutex::new(None);
    static NEXT_ID: Mutex<u64> = Mutex::new(1);

    pub fn create() -> Result<u64, String> {
        // IN_NONBLOCK | IN_CLOEXEC = 0x800 | 0x80000 = 0x80800
        let fd = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
        if fd < 0 {
            return Err(format!("inotify_init1: {}", std::io::Error::last_os_error()));
        }
        let mut id_lock = NEXT_ID.lock().map_err(|e| e.to_string())?;
        let id = *id_lock;
        *id_lock = id_lock.wrapping_add(1);
        let mut reg = REGISTRY.lock().map_err(|e| e.to_string())?;
        if reg.is_none() { *reg = Some(HashMap::new()); }
        reg.as_mut().unwrap().insert(id, Watcher { fd, watches: HashMap::new() });
        Ok(id)
    }

    pub fn raw_fd(id: u64) -> Result<RawFd, String> {
        let reg = REGISTRY.lock().map_err(|e| e.to_string())?;
        let map = reg.as_ref().ok_or("no watchers")?;
        let w = map.get(&id).ok_or("invalid watcher id")?;
        Ok(w.fd)
    }

    pub fn add_watch(id: u64, path: &str) -> Result<i32, String> {
        let mut reg = REGISTRY.lock().map_err(|e| e.to_string())?;
        let map = reg.as_mut().ok_or("no watchers")?;
        let w = map.get_mut(&id).ok_or("invalid watcher id")?;
        let cpath = std::ffi::CString::new(path).map_err(|e| e.to_string())?;
        let mask = libc::IN_MODIFY | libc::IN_CREATE | libc::IN_DELETE
            | libc::IN_MOVED_FROM | libc::IN_MOVED_TO | libc::IN_CLOSE_WRITE
            | libc::IN_ATTRIB;
        let wd = unsafe { libc::inotify_add_watch(w.fd, cpath.as_ptr(), mask) };
        if wd < 0 {
            return Err(format!("inotify_add_watch({}): {}", path, std::io::Error::last_os_error()));
        }
        w.watches.insert(wd, path.to_string());
        Ok(wd)
    }

    /// Drain queued events. Returns Vec<WatchEvent>. Empty vec on
    /// WouldBlock (nothing pending) is normal — the reactor will
    /// signal next readability.
    pub fn read_events(id: u64) -> Result<Vec<WatchEvent>, String> {
        let fd = {
            let reg = REGISTRY.lock().map_err(|e| e.to_string())?;
            let map = reg.as_ref().ok_or("no watchers")?;
            let w = map.get(&id).ok_or("invalid watcher id")?;
            w.fd
        };
        let mut buf = [0u8; 4096];
        let mut out = Vec::new();
        loop {
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    break;
                }
                return Err(format!("inotify read: {}", err));
            }
            if n == 0 { break; }
            let n = n as usize;
            // Walk the buffer parsing struct inotify_event { i32 wd, u32 mask, u32 cookie, u32 len, char name[len] }
            let mut off = 0;
            while off + 16 <= n {
                let wd = i32::from_ne_bytes(buf[off..off+4].try_into().unwrap());
                let mask = u32::from_ne_bytes(buf[off+4..off+8].try_into().unwrap());
                let _cookie = u32::from_ne_bytes(buf[off+8..off+12].try_into().unwrap());
                let len = u32::from_ne_bytes(buf[off+12..off+16].try_into().unwrap()) as usize;
                let name_start = off + 16;
                let name_end = name_start + len;
                if name_end > n { break; }
                let name_bytes = &buf[name_start..name_end];
                let name = match name_bytes.iter().position(|&b| b == 0) {
                    Some(z) => String::from_utf8_lossy(&name_bytes[..z]).into_owned(),
                    None => String::from_utf8_lossy(name_bytes).into_owned(),
                };
                out.push(WatchEvent { wd, mask, name });
                off = name_end;
            }
            if n < buf.len() { break; }  // drained
        }
        Ok(out)
    }

    pub fn close(id: u64) -> Result<(), String> {
        let mut reg = REGISTRY.lock().map_err(|e| e.to_string())?;
        let map = reg.as_mut().ok_or("no watchers")?;
        if let Some(w) = map.remove(&id) {
            unsafe { libc::close(w.fd); }
        }
        Ok(())
    }

    /// Map inotify mask to a Node-style event name. Returns "change"
    /// for content/attribute modifications, "rename" for create/delete/move.
    pub fn event_kind(mask: u32) -> &'static str {
        if mask & (libc::IN_CREATE | libc::IN_DELETE | libc::IN_MOVED_FROM | libc::IN_MOVED_TO) != 0 {
            "rename"
        } else {
            "change"
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub mod inotify {
    pub struct WatchEvent { pub wd: i32, pub mask: u32, pub name: String }
    pub fn create() -> Result<u64, String> { Err("inotify only supported on Linux".to_string()) }
    pub fn raw_fd(_: u64) -> Result<i32, String> { Err("inotify only supported on Linux".to_string()) }
    pub fn add_watch(_: u64, _: &str) -> Result<i32, String> { Err("inotify only supported on Linux".to_string()) }
    pub fn read_events(_: u64) -> Result<Vec<WatchEvent>, String> { Ok(Vec::new()) }
    pub fn close(_: u64) -> Result<(), String> { Ok(()) }
    pub fn event_kind(_: u32) -> &'static str { "change" }
}
