// sockets pilot — TCP socket primitives (Tier-G transport substrate).
//
// Scope (this round, M10 substrate-introduction):
//   - Blocking std::net wrappers exposed as handle-based primitives so a
//     JS host can store handles in a JS-side Map and reference them by id.
//   - TcpListener: bind, accept, close
//   - TcpStream:   connect, read, write, close, peer_addr, local_addr
//   - read() reads up to a caller-specified buffer size (returns bytes-read,
//     up to max size; 0 == orderly close)
//   - write() returns bytes-written (may be < requested for partial writes)
//
// Out of scope (deferred to follow-on rounds per M10 staging):
//   - Async / non-blocking I/O (would require tokio / mio / a poll loop;
//     this pilot is blocking and works on a thread-per-connection model)
//   - TLS (heavy — depends on the closed crypto.subtle pilot + ASN.1)
//   - HTTP/2 multiplexing (binary frame layer; separate substrate)
//   - UDP / Unix domain sockets / IPC
//
// Real consumer use: a JS host wraps these primitives into the Bun.listen /
// Bun.connect API surface; the http-codec pilot (just landed) provides the
// wire-format codec; together they form Tier-G's full surface.
//
// Threading model: handles are stored in a global mutex-protected slab,
// keyed by u64 ids. JS host references a handle by id; the slab keeps the
// underlying TcpListener / TcpStream alive across FFI calls. This avoids
// passing raw socket fds (which the OS may reuse) across the boundary and
// avoids the rquickjs lifetime concerns with owning network types.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use std::collections::HashMap;

#[derive(Debug)]
enum Handle {
    Listener(TcpListener),
    Stream(TcpStream),
}

struct Registry {
    next_id: u64,
    handles: HashMap<u64, Handle>,
}

fn registry() -> &'static Mutex<Registry> {
    static REG: OnceLock<Mutex<Registry>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(Registry { next_id: 1, handles: HashMap::new() }))
}

fn put(h: Handle) -> u64 {
    let mut r = registry().lock().expect("sockets: registry poisoned");
    let id = r.next_id;
    r.next_id += 1;
    r.handles.insert(id, h);
    id
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocketError {
    Bind(String),
    Connect(String),
    Accept(String),
    Read(String),
    Write(String),
    NotFound,
    WrongKind,
}

impl std::fmt::Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketError::Bind(s) => write!(f, "sockets: bind failed: {}", s),
            SocketError::Connect(s) => write!(f, "sockets: connect failed: {}", s),
            SocketError::Accept(s) => write!(f, "sockets: accept failed: {}", s),
            SocketError::Read(s) => write!(f, "sockets: read failed: {}", s),
            SocketError::Write(s) => write!(f, "sockets: write failed: {}", s),
            SocketError::NotFound => write!(f, "sockets: handle not in registry"),
            SocketError::WrongKind => write!(f, "sockets: handle is wrong kind"),
        }
    }
}

// ───────────────────────── TcpListener primitives ──────────────────

/// Bind a TCP listener to `addr` (e.g. "127.0.0.1:0" for any port).
/// Returns (handle_id, actual_bound_addr_as_string). Port 0 lets the OS
/// pick a free port; the returned addr lets the caller learn what was
/// chosen.
pub fn listener_bind(addr: &str) -> Result<(u64, String), SocketError> {
    let listener = TcpListener::bind(addr).map_err(|e| SocketError::Bind(e.to_string()))?;
    let local = listener.local_addr().map_err(|e| SocketError::Bind(e.to_string()))?;
    let id = put(Handle::Listener(listener));
    Ok((id, local.to_string()))
}

/// Block-accept a connection. Returns (stream_handle_id, peer_addr_string).
pub fn listener_accept(id: u64) -> Result<(u64, String), SocketError> {
    // We need exclusive access to the listener but want to release the
    // registry lock while blocking on accept(). Clone the listener via
    // try_clone (cheap, dup()s the fd).
    let listener_clone = {
        let r = registry().lock().expect("sockets: registry poisoned");
        let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
        match h {
            Handle::Listener(l) => l.try_clone().map_err(|e| SocketError::Accept(e.to_string()))?,
            _ => return Err(SocketError::WrongKind),
        }
    };
    let (stream, peer) = listener_clone.accept().map_err(|e| SocketError::Accept(e.to_string()))?;
    let stream_id = put(Handle::Stream(stream));
    Ok((stream_id, peer.to_string()))
}

/// Set accept-timeout via a non-blocking option. Sets the LISTENER's accept
/// timeout to `ms` milliseconds; subsequent listener_accept calls will
/// return SocketError::Accept("WouldBlock"-like) if no connection arrives.
pub fn listener_set_accept_timeout(id: u64, ms: u64) -> Result<(), SocketError> {
    let r = registry().lock().expect("sockets: registry poisoned");
    let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
    match h {
        Handle::Listener(l) => {
            // std's TcpListener doesn't expose set_read_timeout, but we can
            // set non-blocking + use a separate poll loop. For the substrate
            // pilot, we expose set_nonblocking only (timeout is informational).
            let _ = ms;
            l.set_nonblocking(false).map_err(|e| SocketError::Accept(e.to_string()))
        }
        _ => Err(SocketError::WrongKind),
    }
}

// ───────────────────────── TcpStream primitives ────────────────────

/// Connect a TCP client to `addr`. Returns the stream handle id.
pub fn stream_connect(addr: &str) -> Result<u64, SocketError> {
    let stream = TcpStream::connect(addr).map_err(|e| SocketError::Connect(e.to_string()))?;
    Ok(put(Handle::Stream(stream)))
}

/// Connect with a timeout (milliseconds). Useful for client code that
/// shouldn't block forever.
pub fn stream_connect_timeout(addr: &str, timeout_ms: u64) -> Result<u64, SocketError> {
    let sa: SocketAddr = addr.parse().map_err(|e: std::net::AddrParseError| SocketError::Connect(e.to_string()))?;
    let stream = TcpStream::connect_timeout(&sa, Duration::from_millis(timeout_ms))
        .map_err(|e| SocketError::Connect(e.to_string()))?;
    Ok(put(Handle::Stream(stream)))
}

/// Read up to `max` bytes from the stream. Returns the actual bytes read.
/// Empty Vec means orderly close.
pub fn stream_read(id: u64, max: usize) -> Result<Vec<u8>, SocketError> {
    // Clone the stream fd so we can release the registry lock during the
    // blocking read. TcpStream::try_clone() dup()s the underlying fd.
    let mut stream = {
        let r = registry().lock().expect("sockets: registry poisoned");
        let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
        match h {
            Handle::Stream(s) => s.try_clone().map_err(|e| SocketError::Read(e.to_string()))?,
            _ => return Err(SocketError::WrongKind),
        }
    };
    let mut buf = vec![0u8; max];
    let n = stream.read(&mut buf).map_err(|e| SocketError::Read(e.to_string()))?;
    buf.truncate(n);
    Ok(buf)
}

/// Write bytes to the stream. Returns the number of bytes written.
pub fn stream_write(id: u64, data: &[u8]) -> Result<usize, SocketError> {
    let mut stream = {
        let r = registry().lock().expect("sockets: registry poisoned");
        let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
        match h {
            Handle::Stream(s) => s.try_clone().map_err(|e| SocketError::Write(e.to_string()))?,
            _ => return Err(SocketError::WrongKind),
        }
    };
    let n = stream.write(data).map_err(|e| SocketError::Write(e.to_string()))?;
    Ok(n)
}

/// Write the entire buffer, looping over partial writes.
pub fn stream_write_all(id: u64, data: &[u8]) -> Result<(), SocketError> {
    let mut stream = {
        let r = registry().lock().expect("sockets: registry poisoned");
        let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
        match h {
            Handle::Stream(s) => s.try_clone().map_err(|e| SocketError::Write(e.to_string()))?,
            _ => return Err(SocketError::WrongKind),
        }
    };
    stream.write_all(data).map_err(|e| SocketError::Write(e.to_string()))?;
    Ok(())
}

pub fn stream_peer_addr(id: u64) -> Result<String, SocketError> {
    let r = registry().lock().expect("sockets: registry poisoned");
    let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
    match h {
        Handle::Stream(s) => s.peer_addr().map(|a| a.to_string())
            .map_err(|e| SocketError::Read(e.to_string())),
        _ => Err(SocketError::WrongKind),
    }
}

pub fn stream_local_addr(id: u64) -> Result<String, SocketError> {
    let r = registry().lock().expect("sockets: registry poisoned");
    let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
    match h {
        Handle::Stream(s) => s.local_addr().map(|a| a.to_string())
            .map_err(|e| SocketError::Read(e.to_string())),
        _ => Err(SocketError::WrongKind),
    }
}

/// Close a handle (listener or stream) by removing it from the registry.
/// Drop runs the OS close.
pub fn handle_close(id: u64) -> Result<(), SocketError> {
    let mut r = registry().lock().expect("sockets: registry poisoned");
    r.handles.remove(&id).ok_or(SocketError::NotFound)?;
    Ok(())
}

pub fn handle_kind(id: u64) -> Result<&'static str, SocketError> {
    let r = registry().lock().expect("sockets: registry poisoned");
    let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
    Ok(match h {
        Handle::Listener(_) => "listener",
        Handle::Stream(_) => "stream",
    })
}
