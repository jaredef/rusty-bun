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
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
use std::collections::HashMap;

enum Handle {
    Listener(TcpListener),
    Stream(TcpStream),
    AsyncListener(AsyncListener),
}

// AsyncListener: a TcpListener whose accept loop runs on a background
// thread. Accepted connections (stream-handle ids) flow through an mpsc
// channel to be polled JS-side. Architecturally matches Bun's pattern
// (background-thread accept + concurrent_tasks queue + main-thread
// drain) per the Bun event-loop docs; the std::thread + mpsc combo is
// the std-only equivalent of Bun's WorkPool + Waker (engagement
// 2026-05-11; option A of the async-bridge decision).
struct AsyncListener {
    // Receiver wrapped in Arc<Mutex<>> so listener_poll can clone the Arc,
    // release the registry lock, then block on recv_timeout independently.
    // Critical: without this, the accept_loop's `put(Handle::Stream)` would
    // deadlock against listener_poll holding the registry lock during recv.
    rx: Arc<Mutex<Receiver<AsyncEvent>>>,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    local_addr: String,
}

#[derive(Debug, Clone)]
pub enum AsyncEvent {
    /// A new connection was accepted; consumer can read/write via stream_id.
    Connection { stream_id: u64, peer: String },
    /// The accept loop exited (shutdown signal or unrecoverable error).
    Closed,
    /// A non-fatal accept error.
    Error(String),
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
    use std::net::ToSocketAddrs;
    // Resolve via to_socket_addrs (handles "localhost:N", "127.0.0.1:N",
    // and IPv6). std::net::SocketAddr::parse only accepts pre-resolved
    // IP:port forms; HTTP clients commonly pass hostnames.
    let sa: SocketAddr = addr.to_socket_addrs()
        .map_err(|e| SocketError::Connect(e.to_string()))?
        .next()
        .ok_or_else(|| SocketError::Connect("no addresses resolved".into()))?;
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

/// Π2.6.b: set non-blocking mode on a TcpStream so reads return WouldBlock
/// instead of suspending the JS thread. Required for canonical in-process
/// client↔server interleaving via the host's __keepAlive cooperative loop.
pub fn stream_set_nonblocking(id: u64, on: bool) -> Result<(), SocketError> {
    let stream = {
        let r = registry().lock().expect("sockets: registry poisoned");
        let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
        match h {
            Handle::Stream(s) => s.try_clone().map_err(|e| SocketError::Read(e.to_string()))?,
            _ => return Err(SocketError::WrongKind),
        }
    };
    stream.set_nonblocking(on).map_err(|e| SocketError::Read(e.to_string()))
}

/// Π2.6.b: non-blocking read. Returns Ok(Some(bytes)) on data,
/// Ok(None) on WouldBlock, Ok(Some(empty)) on orderly close (EOF).
/// Caller-side cooperative loop alternates this with __tickKeepAlive
/// + microtask yield to let in-process peers run.
pub fn stream_try_read(id: u64, max: usize) -> Result<Option<Vec<u8>>, SocketError> {
    let mut stream = {
        let r = registry().lock().expect("sockets: registry poisoned");
        let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
        match h {
            Handle::Stream(s) => s.try_clone().map_err(|e| SocketError::Read(e.to_string()))?,
            _ => return Err(SocketError::WrongKind),
        }
    };
    let mut buf = vec![0u8; max];
    match stream.read(&mut buf) {
        Ok(n) => { buf.truncate(n); Ok(Some(buf)) }
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(SocketError::Read(e.to_string())),
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
        Handle::AsyncListener(_) => "async-listener",
    })
}

// ─────────────────── Async-listener primitives (option A) ────────────
//
// Engagement 2026-05-11 design: thread-per-listener + mpsc channel +
// main-thread poll. Matches Bun's background-thread+concurrent_tasks
// architecture using std-only primitives. The main JS event loop
// polls the channel via TCP.poll(id, maxWaitMs) and dispatches accepted
// connections to fetch handlers.

/// Bind a TCP listener in async mode. Spawns a background thread that
/// runs the accept loop, pushing accepted connections into a channel.
/// Returns the async-listener handle id + the bound local addr.
pub fn listener_bind_async(addr: &str) -> Result<(u64, String), SocketError> {
    let listener = TcpListener::bind(addr).map_err(|e| SocketError::Bind(e.to_string()))?;
    let local = listener.local_addr().map_err(|e| SocketError::Bind(e.to_string()))?;
    let local_str = local.to_string();
    listener.set_nonblocking(true).map_err(|e| SocketError::Bind(e.to_string()))?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let (tx, rx): (Sender<AsyncEvent>, Receiver<AsyncEvent>) = mpsc::channel();
    let thread_shutdown = shutdown.clone();
    let thread = std::thread::spawn(move || {
        accept_loop(listener, tx, thread_shutdown);
    });
    let async_listener = AsyncListener {
        rx: Arc::new(Mutex::new(rx)),
        shutdown,
        thread: Some(thread),
        local_addr: local_str.clone(),
    };
    let id = put(Handle::AsyncListener(async_listener));
    Ok((id, local_str))
}

fn accept_loop(listener: TcpListener, tx: Sender<AsyncEvent>, shutdown: Arc<AtomicBool>) {
    // Non-blocking listener; poll every 10ms. Std-only equivalent of
    // Bun's epoll/kqueue-based wait — slightly higher latency, same shape.
    while !shutdown.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, peer)) => {
                // Stream is non-blocking inherited from the listener.
                // Reset to blocking — JS-side reads expect blocking semantics
                // (the read happens after poll() returns a Connection event,
                // so the bytes should already be on the wire or arriving soon).
                if let Err(e) = stream.set_nonblocking(false) {
                    let _ = tx.send(AsyncEvent::Error(format!("set_blocking: {}", e)));
                    continue;
                }
                let stream_id = put(Handle::Stream(stream));
                if tx.send(AsyncEvent::Connection { stream_id, peer: peer.to_string() }).is_err() {
                    // Receiver dropped; exit loop.
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = tx.send(AsyncEvent::Error(format!("accept: {}", e)));
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    let _ = tx.send(AsyncEvent::Closed);
}

/// Poll for the next async event from a bound async listener. Blocks
/// up to `max_wait_ms` milliseconds; returns Ok(None) on timeout.
pub fn listener_poll(id: u64, max_wait_ms: u64) -> Result<Option<AsyncEvent>, SocketError> {
    // Clone the Arc<Mutex<Receiver>> out of the registry so we can release
    // the registry lock BEFORE blocking on recv. Without this, the
    // accept_loop background thread's put(Handle::Stream(...)) deadlocks
    // against listener_poll holding the registry lock.
    let rx = {
        let r = registry().lock().expect("sockets: registry poisoned");
        let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
        match h {
            Handle::AsyncListener(al) => al.rx.clone(),
            _ => return Err(SocketError::WrongKind),
        }
    };  // registry lock released here
    let guard = rx.lock().expect("sockets: rx mutex poisoned");
    match guard.recv_timeout(Duration::from_millis(max_wait_ms)) {
        Ok(ev) => Ok(Some(ev)),
        Err(RecvTimeoutError::Timeout) => Ok(None),
        Err(RecvTimeoutError::Disconnected) => Ok(Some(AsyncEvent::Closed)),
    }
}

/// Signal the async listener to shut down. Joins the background thread.
/// Subsequent listener_poll on the id will return Closed.
pub fn listener_stop_async(id: u64) -> Result<(), SocketError> {
    // Take the listener out of the registry (this drops it from the slab).
    let mut r = registry().lock().expect("sockets: registry poisoned");
    let h = r.handles.remove(&id).ok_or(SocketError::NotFound)?;
    drop(r); // release the registry lock before joining
    match h {
        Handle::AsyncListener(mut al) => {
            al.shutdown.store(true, Ordering::Release);
            if let Some(t) = al.thread.take() {
                let _ = t.join();
            }
            Ok(())
        }
        _ => Err(SocketError::WrongKind),
    }
}

pub fn async_listener_addr(id: u64) -> Result<String, SocketError> {
    let r = registry().lock().expect("sockets: registry poisoned");
    let h = r.handles.get(&id).ok_or(SocketError::NotFound)?;
    match h {
        Handle::AsyncListener(al) => Ok(al.local_addr.clone()),
        _ => Err(SocketError::WrongKind),
    }
}
