// Bun.spawn pilot — subprocess management.
//
// Inputs:
//   AUDIT — pilots/bun-spawn/AUDIT.md
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md
//           (Bun.spawn: 79 + 13 clauses)
//   REF   — Bun docs at https://bun.sh/docs/api/spawn
//
// Tier-2 ecosystem-only. Derivation wraps std::process::Command. Pilot scope
// is the synchronous + simple-async API; streaming stdio with ReadableStream
// integration deferred (would compose with the Streams pilot).

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdioMode {
    /// Capture into a Vec<u8> via a pipe.
    Pipe,
    /// Inherit the parent's stdio handle.
    Inherit,
    /// Discard.
    Null,
}

impl StdioMode {
    fn into_stdio(self) -> Stdio {
        match self {
            StdioMode::Pipe => Stdio::piped(),
            StdioMode::Inherit => Stdio::inherit(),
            StdioMode::Null => Stdio::null(),
        }
    }
}

/// Bytes or string supplied as stdin to the child.
#[derive(Debug, Clone)]
pub enum StdinInput {
    None,
    Bytes(Vec<u8>),
    Text(String),
    Inherit,
    Null,
}

#[derive(Debug, Clone)]
pub struct SpawnOptions {
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub stdin: StdinInput,
    pub stdout: StdioMode,
    pub stderr: StdioMode,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            cwd: None,
            env: None,
            stdin: StdinInput::None,
            stdout: StdioMode::Pipe,
            stderr: StdioMode::Pipe,
        }
    }
}

/// Result of `Bun::spawn_sync` — collects stdout/stderr/exit-code.
#[derive(Debug, Clone)]
pub struct SyncSubprocess {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
    pub signal_code: Option<String>,
    pub success: bool,
}

/// Async-shaped subprocess handle returned by `Bun::spawn`. Consumer calls
/// `.wait()` to collect the result.
pub struct Subprocess {
    child: Child,
    pid: u32,
    captured_stdout: bool,
    captured_stderr: bool,
}

impl std::fmt::Debug for Subprocess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subprocess")
            .field("pid", &self.pid)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ExitStatus {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
    pub signal_code: Option<String>,
    pub success: bool,
}

#[derive(Debug)]
pub enum SpawnError {
    EmptyArgs,
    Io(std::io::Error),
}

impl From<std::io::Error> for SpawnError {
    fn from(e: std::io::Error) -> Self { SpawnError::Io(e) }
}

/// `Bun.spawn(args, options)` — async-shaped. Returns a Subprocess handle.
/// CD: `expect(proc.pid).toBeNumber()`, `expect(proc.exited).toBeDefined()`
pub fn spawn(args: &[&str], options: SpawnOptions) -> Result<Subprocess, SpawnError> {
    if args.is_empty() { return Err(SpawnError::EmptyArgs); }
    let mut cmd = build_command(args, &options)?;
    let captured_stdout = matches!(options.stdout, StdioMode::Pipe);
    let captured_stderr = matches!(options.stderr, StdioMode::Pipe);
    let mut child = cmd.spawn()?;
    let pid = child.id();

    // If stdin is a piped input, write it then close.
    match &options.stdin {
        StdinInput::Bytes(b) => {
            if let Some(mut stdin) = child.stdin.take() { stdin.write_all(b)?; }
        }
        StdinInput::Text(s) => {
            if let Some(mut stdin) = child.stdin.take() { stdin.write_all(s.as_bytes())?; }
        }
        _ => {}
    }

    Ok(Subprocess { child, pid, captured_stdout, captured_stderr })
}

/// `Bun.spawnSync(args, options)` — synchronous. Blocks until the child
/// exits and returns collected stdio + exit code.
pub fn spawn_sync(args: &[&str], options: SpawnOptions) -> Result<SyncSubprocess, SpawnError> {
    if args.is_empty() { return Err(SpawnError::EmptyArgs); }
    let mut cmd = build_command(args, &options)?;

    // For spawn_sync with byte/text stdin we must spawn manually + write
    // stdin + wait_with_output to gather both pipes.
    let captured_stdout = matches!(options.stdout, StdioMode::Pipe);
    let captured_stderr = matches!(options.stderr, StdioMode::Pipe);
    let mut child = cmd.spawn()?;
    match &options.stdin {
        StdinInput::Bytes(b) => {
            if let Some(mut stdin) = child.stdin.take() { stdin.write_all(b)?; }
        }
        StdinInput::Text(s) => {
            if let Some(mut stdin) = child.stdin.take() { stdin.write_all(s.as_bytes())?; }
        }
        _ => {}
    }
    let output = child.wait_with_output()?;
    let exit_code = output.status.code().unwrap_or(-1);
    Ok(SyncSubprocess {
        stdout: if captured_stdout { output.stdout } else { Vec::new() },
        stderr: if captured_stderr { output.stderr } else { Vec::new() },
        exit_code,
        signal_code: signal_code_of(&output.status),
        success: output.status.success(),
    })
}

impl Subprocess {
    pub fn pid(&self) -> u32 { self.pid }

    /// `proc.exited` — has the child exited yet. Pilot polls non-blocking.
    pub fn exited(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    /// `proc.kill(signal?)` — terminate the child. Pilot ignores the
    /// signal argument (std::process only exposes a generic kill).
    pub fn kill(&mut self) -> Result<(), SpawnError> {
        Ok(self.child.kill()?)
    }

    /// `proc.exited` Promise → `wait()`. Block until child exits + collect
    /// stdio + exit code.
    pub fn wait(self) -> Result<ExitStatus, SpawnError> {
        let captured_stdout = self.captured_stdout;
        let captured_stderr = self.captured_stderr;
        let output = self.child.wait_with_output()?;
        Ok(ExitStatus {
            stdout: if captured_stdout { output.stdout } else { Vec::new() },
            stderr: if captured_stderr { output.stderr } else { Vec::new() },
            exit_code: output.status.code().unwrap_or(-1),
            signal_code: signal_code_of(&output.status),
            success: output.status.success(),
        })
    }
}

fn build_command(args: &[&str], options: &SpawnOptions) -> Result<Command, SpawnError> {
    let mut cmd = Command::new(args[0]);
    if args.len() > 1 { cmd.args(&args[1..]); }
    if let Some(cwd) = &options.cwd { cmd.current_dir(cwd); }
    if let Some(env) = &options.env {
        cmd.env_clear();
        for (k, v) in env { cmd.env(k, v); }
    }
    cmd.stdin(match &options.stdin {
        StdinInput::None | StdinInput::Null => Stdio::null(),
        StdinInput::Inherit => Stdio::inherit(),
        StdinInput::Bytes(_) | StdinInput::Text(_) => Stdio::piped(),
    });
    cmd.stdout(options.stdout.into_stdio());
    cmd.stderr(options.stderr.into_stdio());
    Ok(cmd)
}

fn signal_code_of(status: &std::process::ExitStatus) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.signal().map(|s| format!("SIG{}", s))
    }
    #[cfg(not(unix))]
    {
        let _ = status;
        None
    }
}
