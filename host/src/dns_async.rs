// Π2.6.d.d — async DNS resolution.
//
// Pattern: per-request worker thread + eventfd wake. The worker calls
// std::net::ToSocketAddrs::to_socket_addrs (blocking gethostbyname)
// and writes the result into a per-thread completion queue. An
// eventfd is registered with the mio reactor; the worker writes(1)
// to the eventfd on completion, waking the eval loop. The JS layer
// drains the queue and resolves any pending Promises.
//
// Per-thread state (cargo test parallel isolation). Multiple
// in-flight requests share one eventfd; each request has a u32 id
// the worker stores in the result so the JS side knows which
// Promise to resolve.

#[cfg(target_os = "linux")]
pub mod async_dns {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::net::ToSocketAddrs;
    use std::os::unix::io::RawFd;
    use std::sync::mpsc::{Sender, Receiver, channel};
    use std::thread;

    pub struct Result {
        pub id: u32,
        pub addresses: Vec<String>,
        pub error: Option<String>,
    }

    pub struct Subsystem {
        pub fd: RawFd,
        pub tx: Sender<Result>,
        pub rx: Receiver<Result>,
        pub next_id: u32,
    }

    thread_local! {
        static SUBSYS: RefCell<Option<Subsystem>> = const { RefCell::new(None) };
    }

    fn ensure() -> RawFd {
        SUBSYS.with(|c| {
            let mut c = c.borrow_mut();
            if c.is_none() {
                let fd = unsafe { libc::eventfd(0, libc::EFD_NONBLOCK | libc::EFD_CLOEXEC) };
                let (tx, rx) = channel();
                *c = Some(Subsystem { fd, tx, rx, next_id: 1 });
            }
            c.as_ref().unwrap().fd
        })
    }

    pub fn raw_fd() -> RawFd {
        ensure()
    }

    pub fn submit(host: String, port: u16) -> u32 {
        let id = SUBSYS.with(|c| {
            let mut c = c.borrow_mut();
            let s = c.as_mut().unwrap();
            let id = s.next_id;
            s.next_id = s.next_id.wrapping_add(1);
            id
        });
        let (tx, fd) = SUBSYS.with(|c| {
            let c = c.borrow();
            let s = c.as_ref().unwrap();
            (s.tx.clone(), s.fd)
        });
        thread::spawn(move || {
            let addr = format!("{}:{}", host, port);
            let result = match addr.to_socket_addrs() {
                Ok(iter) => Result {
                    id,
                    addresses: iter.map(|sa| sa.ip().to_string()).collect(),
                    error: None,
                },
                Err(e) => Result {
                    id,
                    addresses: Vec::new(),
                    error: Some(e.to_string()),
                },
            };
            let _ = tx.send(result);
            let one: u64 = 1;
            unsafe {
                let _ = libc::write(fd, &one as *const u64 as *const _, 8);
            }
        });
        id
    }

    /// Drain completed results. Reads the eventfd counter to reset it,
    /// then drains the mpsc queue. Returns all results since the last
    /// drain. Per-result shape: (id, addresses_joined_by_comma, error_or_empty).
    pub fn drain() -> Vec<(u32, String, String)> {
        SUBSYS.with(|c| {
            let c = c.borrow();
            let Some(s) = c.as_ref() else { return Vec::new(); };
            // Consume eventfd counter.
            let mut buf = [0u8; 8];
            unsafe {
                let _ = libc::read(s.fd, buf.as_mut_ptr() as *mut _, 8);
            }
            let mut out = Vec::new();
            while let Ok(r) = s.rx.try_recv() {
                out.push((r.id, r.addresses.join(","), r.error.unwrap_or_default()));
            }
            out
        })
    }
}

#[cfg(not(target_os = "linux"))]
pub mod async_dns {
    pub fn raw_fd() -> i32 { -1 }
    pub fn submit(_: String, _: u16) -> u32 { 0 }
    pub fn drain() -> Vec<(u32, String, String)> { Vec::new() }
}
