// Π2.6.d.c — signal delivery to JS via signalfd + mio reactor.
//
// Per-thread signalfd registry. JS process.on('SIGINT', fn) registers
// a handler; first registration for a signal mask blocks the signal
// at the thread level (so default action doesn't fire) and creates
// a signalfd that becomes readable when the signal is delivered.
// The reactor wakes the eval loop; the consumer's __tickKeepAlive
// pump reads the siginfo struct and dispatches handlers.
//
// Per-thread because cargo test runs parallel — global signal masks
// across threads collide; signalfd's per-thread scope (via
// pthread_sigmask) gives clean isolation.

#[cfg(target_os = "linux")]
pub mod sigfd {
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::os::unix::io::RawFd;

    pub struct SignalSubsystem {
        pub fd: RawFd,
        pub signals: HashSet<i32>,
    }

    thread_local! {
        static SUBSYS: RefCell<Option<SignalSubsystem>> = const { RefCell::new(None) };
    }

    /// Map Node-style signal name to signum.
    pub fn signum_for(name: &str) -> Option<i32> {
        Some(match name {
            "SIGHUP" => libc::SIGHUP,
            "SIGINT" => libc::SIGINT,
            "SIGQUIT" => libc::SIGQUIT,
            "SIGILL" => libc::SIGILL,
            "SIGABRT" => libc::SIGABRT,
            "SIGFPE" => libc::SIGFPE,
            "SIGKILL" => libc::SIGKILL,
            "SIGUSR1" => libc::SIGUSR1,
            "SIGSEGV" => libc::SIGSEGV,
            "SIGUSR2" => libc::SIGUSR2,
            "SIGPIPE" => libc::SIGPIPE,
            "SIGALRM" => libc::SIGALRM,
            "SIGTERM" => libc::SIGTERM,
            "SIGCHLD" => libc::SIGCHLD,
            "SIGCONT" => libc::SIGCONT,
            "SIGSTOP" => libc::SIGSTOP,
            "SIGTSTP" => libc::SIGTSTP,
            "SIGTTIN" => libc::SIGTTIN,
            "SIGTTOU" => libc::SIGTTOU,
            _ => return None,
        })
    }

    /// Add a signal to the per-thread signalfd. Creates the signalfd
    /// on first call. Returns the fd (caller registers with reactor
    /// after the first call; subsequent calls return the same fd).
    /// Returns (fd, is_new_fd).
    pub fn add_signal(signum: i32) -> Result<(RawFd, bool), String> {
        SUBSYS.with(|cell| {
            let mut cell = cell.borrow_mut();
            let mut sig_set: libc::sigset_t = unsafe { std::mem::zeroed() };
            unsafe {
                libc::sigemptyset(&mut sig_set);
                if let Some(ref sub) = *cell {
                    for &s in &sub.signals { libc::sigaddset(&mut sig_set, s); }
                }
                libc::sigaddset(&mut sig_set, signum);
                // Block the signal so default action doesn't fire +
                // signalfd can deliver via the fd.
                libc::pthread_sigmask(libc::SIG_BLOCK, &sig_set, std::ptr::null_mut());
            }
            // Create or update signalfd. signalfd() returns the existing
            // fd if you pass it as the first arg; -1 creates new.
            let existing = cell.as_ref().map(|s| s.fd).unwrap_or(-1);
            let flags = libc::SFD_NONBLOCK | libc::SFD_CLOEXEC;
            let fd = unsafe { libc::signalfd(existing, &sig_set, flags) };
            if fd < 0 {
                return Err(format!("signalfd: {}", std::io::Error::last_os_error()));
            }
            let is_new = existing < 0;
            let mut signals = cell.as_ref().map(|s| s.signals.clone()).unwrap_or_default();
            signals.insert(signum);
            *cell = Some(SignalSubsystem { fd, signals });
            Ok((fd, is_new))
        })
    }

    /// Drain queued siginfo structs from the signalfd. Returns the
    /// signum of each delivered signal. WouldBlock → empty.
    pub fn read_signals() -> Result<Vec<i32>, String> {
        let fd = SUBSYS.with(|c| c.borrow().as_ref().map(|s| s.fd).unwrap_or(-1));
        if fd < 0 { return Ok(Vec::new()); }
        // struct signalfd_siginfo is 128 bytes on Linux. First u32 is ssi_signo.
        const SIGINFO_SIZE: usize = 128;
        let mut out = Vec::new();
        loop {
            let mut buf = [0u8; SIGINFO_SIZE * 8];
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::WouldBlock { break; }
                return Err(format!("signalfd read: {}", err));
            }
            if n == 0 { break; }
            let n = n as usize;
            let count = n / SIGINFO_SIZE;
            for i in 0..count {
                let ssi_signo = u32::from_ne_bytes(
                    buf[i * SIGINFO_SIZE..i * SIGINFO_SIZE + 4].try_into().unwrap());
                out.push(ssi_signo as i32);
            }
            if n < buf.len() { break; }
        }
        Ok(out)
    }

    pub fn fd_or_none() -> Option<RawFd> {
        SUBSYS.with(|c| c.borrow().as_ref().map(|s| s.fd))
    }
}

/// libc::kill wrapper exposed via process.kill.
#[cfg(unix)]
pub fn kill(pid: i32, signum: i32) -> Result<(), String> {
    let rc = unsafe { libc::kill(pid as libc::pid_t, signum) };
    if rc != 0 {
        Err(format!("kill: {}", std::io::Error::last_os_error()))
    } else {
        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
pub mod sigfd {
    use std::os::raw::c_int;
    pub fn signum_for(_: &str) -> Option<c_int> { None }
    pub fn add_signal(_: c_int) -> Result<(c_int, bool), String> { Err("signalfd only on Linux".to_string()) }
    pub fn read_signals() -> Result<Vec<c_int>, String> { Ok(Vec::new()) }
    pub fn fd_or_none() -> Option<c_int> { None }
}
