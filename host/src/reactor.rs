// Π2.6.c.a — single-process mio reactor.
//
// Per seed §A8.16 the registry lives behind a Mutex. Per the sockets
// pilot's lifetime model the underlying fd is owned by the sockets
// registry; mio's SourceFd is borrow-shaped, so we register fds by
// their sid (cast to mio::Token) without taking ownership. The eval
// loop calls poll_once() with a timeout when microtask + keep-alive +
// timer queues are quiescent.
//
// Scope of this round (Π2.6.c.a): expose register / deregister /
// poll_once / take_ready as JS-callable primitives via globalThis.
// __reactor.*. No consumer-side migration; the existing tryRead /
// idleSpin path in fetch stays operational. Π2.6.c.b migrates fetch.

use mio::{Events, Interest, Poll, Token};
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

pub struct Reactor {
    poll: Mutex<Poll>,
    ready: Mutex<Vec<u64>>,
    registered: Mutex<HashSet<u64>>,
}

static REACTOR: std::sync::OnceLock<Reactor> = std::sync::OnceLock::new();

fn reactor() -> &'static Reactor {
    REACTOR.get_or_init(|| Reactor {
        poll: Mutex::new(Poll::new().expect("mio Poll::new")),
        ready: Mutex::new(Vec::new()),
        registered: Mutex::new(HashSet::new()),
    })
}

#[cfg(unix)]
pub fn register_fd(sid: u64, fd: std::os::unix::io::RawFd) -> Result<(), String> {
    use mio::unix::SourceFd;
    let r = reactor();
    let poll = r.poll.lock().map_err(|e| e.to_string())?;
    let mut src = SourceFd(&fd);
    poll.registry()
        .register(&mut src, Token(sid as usize), Interest::READABLE)
        .map_err(|e| e.to_string())?;
    drop(poll);
    r.registered
        .lock()
        .map_err(|e| e.to_string())?
        .insert(sid);
    Ok(())
}

#[cfg(unix)]
pub fn deregister_fd(sid: u64, fd: std::os::unix::io::RawFd) -> Result<(), String> {
    use mio::unix::SourceFd;
    let r = reactor();
    let poll = r.poll.lock().map_err(|e| e.to_string())?;
    let mut src = SourceFd(&fd);
    // Ignore errors — fd may already have been closed by the sockets
    // pilot; deregister is best-effort.
    let _ = poll.registry().deregister(&mut src);
    drop(poll);
    r.registered
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&sid);
    Ok(())
}

#[cfg(not(unix))]
pub fn register_fd(_sid: u64, _fd: i32) -> Result<(), String> {
    Err("reactor: register_fd not supported on this platform".to_string())
}

#[cfg(not(unix))]
pub fn deregister_fd(_sid: u64, _fd: i32) -> Result<(), String> {
    Err("reactor: deregister_fd not supported on this platform".to_string())
}

/// Poll mio for readiness events with a timeout. Returns the number of
/// tokens that became readable; their ids accumulate in the ready
/// queue, drainable via take_ready().
///
/// timeout_ms < 0 → block until something readable.
/// timeout_ms == 0 → non-blocking poll.
/// timeout_ms > 0 → wait up to that many ms.
pub fn poll_once(timeout_ms: i64) -> Result<usize, String> {
    let r = reactor();
    let mut poll = r.poll.lock().map_err(|e| e.to_string())?;
    let timeout = if timeout_ms < 0 {
        None
    } else {
        Some(Duration::from_millis(timeout_ms as u64))
    };
    let mut events = Events::with_capacity(128);
    poll.poll(&mut events, timeout).map_err(|e| e.to_string())?;
    drop(poll);
    let mut ready = r.ready.lock().map_err(|e| e.to_string())?;
    let mut count = 0;
    for ev in events.iter() {
        if ev.is_readable() {
            ready.push(ev.token().0 as u64);
            count += 1;
        }
    }
    Ok(count)
}

pub fn take_ready() -> Vec<u64> {
    if let Ok(mut ready) = reactor().ready.lock() {
        std::mem::take(&mut *ready)
    } else {
        Vec::new()
    }
}

pub fn registered_count() -> usize {
    reactor()
        .registered
        .lock()
        .map(|r| r.len())
        .unwrap_or(0)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::os::unix::io::AsRawFd;

    #[test]
    fn readable_event_fires_on_socketpair() {
        // Bind a localhost listener, connect a client, register the
        // server-side fd with the reactor, write from the client, poll
        // and expect the server-side token in take_ready().
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut client = TcpStream::connect(addr).unwrap();
        // Accept loop (poll-mode since the listener is nonblocking).
        let mut server = loop {
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(5)),
            }
        };
        server.set_nonblocking(true).unwrap();
        let sid = 0xC0DE;
        register_fd(sid, server.as_raw_fd()).unwrap();
        // Write before polling.
        client.write_all(b"hi").unwrap();
        let _ = poll_once(500).unwrap();
        let ready = take_ready();
        assert!(ready.contains(&sid), "expected sid {} in ready, got {:?}", sid, ready);
        // Drain to leave the fd quiescent before deregister.
        let mut buf = [0u8; 16];
        let _ = server.read(&mut buf);
        deregister_fd(sid, server.as_raw_fd()).unwrap();
        assert_eq!(registered_count(), 0);
    }

    #[test]
    fn poll_zero_timeout_is_nonblocking() {
        // No fds registered + zero timeout → returns immediately with 0.
        let start = std::time::Instant::now();
        let n = poll_once(0).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(n, 0);
        assert!(elapsed < std::time::Duration::from_millis(50), "poll(0) blocked {}ms", elapsed.as_millis());
    }
}
