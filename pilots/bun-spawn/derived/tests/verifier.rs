// Verifier for the Bun.spawn pilot. Tests actually invoke child processes.

use rusty_bun_spawn::*;

// ════════════════════ SPAWN_SYNC ════════════════════

#[test]
fn cd_spawn_sync_echo_captures_stdout() {
    let r = spawn_sync(&["sh", "-c", "echo hello"], Default::default()).unwrap();
    assert!(r.success);
    assert_eq!(r.exit_code, 0);
    assert_eq!(r.stdout, b"hello\n".to_vec());
}

#[test]
fn cd_spawn_sync_exit_code_nonzero() {
    let r = spawn_sync(&["sh", "-c", "exit 7"], Default::default()).unwrap();
    assert_eq!(r.exit_code, 7);
    assert!(!r.success);
}

#[test]
fn cd_spawn_sync_stderr_captured() {
    let r = spawn_sync(&["sh", "-c", "echo error >&2"], Default::default()).unwrap();
    assert_eq!(r.stderr, b"error\n".to_vec());
}

#[test]
fn cd_spawn_sync_stdin_text_passed_through() {
    let r = spawn_sync(&["cat"], SpawnOptions {
        stdin: StdinInput::Text("input data".into()),
        ..Default::default()
    }).unwrap();
    assert_eq!(r.stdout, b"input data".to_vec());
}

#[test]
fn cd_spawn_sync_stdin_bytes_passed_through() {
    let r = spawn_sync(&["cat"], SpawnOptions {
        stdin: StdinInput::Bytes(vec![0u8, 1, 2, 3, 0xFF]),
        ..Default::default()
    }).unwrap();
    assert_eq!(r.stdout, vec![0u8, 1, 2, 3, 0xFF]);
}

#[test]
fn spec_spawn_sync_cwd_set() {
    let r = spawn_sync(&["pwd"], SpawnOptions {
        cwd: Some(std::path::PathBuf::from("/tmp")),
        ..Default::default()
    }).unwrap();
    let out = String::from_utf8(r.stdout).unwrap();
    assert!(out.starts_with("/tmp") || out.starts_with("/private/tmp"),
        "expected /tmp prefix, got {}", out);
}

#[test]
fn spec_spawn_sync_env_set() {
    let mut env = std::collections::HashMap::new();
    env.insert("MYVAR".into(), "thevalue".into());
    let r = spawn_sync(&["sh", "-c", "echo $MYVAR"], SpawnOptions {
        env: Some(env),
        ..Default::default()
    }).unwrap();
    assert_eq!(r.stdout, b"thevalue\n".to_vec());
}

#[test]
fn spec_spawn_sync_env_clear_when_set() {
    // When env is set, parent env should NOT be inherited per Bun docs.
    // We use `env` directly (not via shell) because shells auto-initialize
    // PATH from a built-in default if PATH is unset, masking the env-clear.
    let mut env = std::collections::HashMap::new();
    env.insert("ONLY_THIS".into(), "x".into());
    let r = spawn_sync(&["env"], SpawnOptions {
        env: Some(env),
        ..Default::default()
    }).unwrap();
    let out = String::from_utf8(r.stdout).unwrap();
    assert!(out.contains("ONLY_THIS=x"));
    // Child env should NOT contain inherited variables like PATH.
    assert!(!out.contains("PATH="),
        "child should not inherit PATH; got {}", out);
}

#[test]
fn spec_spawn_sync_stdout_null_discards() {
    let r = spawn_sync(&["sh", "-c", "echo hello"], SpawnOptions {
        stdout: StdioMode::Null,
        ..Default::default()
    }).unwrap();
    assert!(r.stdout.is_empty());
    assert!(r.success);
}

#[test]
fn spec_spawn_sync_empty_args_errors() {
    let r = spawn_sync(&[], Default::default());
    assert!(matches!(r, Err(SpawnError::EmptyArgs)));
}

// ════════════════════ SPAWN (ASYNC-SHAPED) ════════════════════

#[test]
fn cd_spawn_returns_handle_with_pid() {
    let proc = spawn(&["sh", "-c", "echo async"], Default::default()).unwrap();
    assert!(proc.pid() > 0);
    let r = proc.wait().unwrap();
    assert!(r.success);
    assert_eq!(r.stdout, b"async\n".to_vec());
}

#[test]
fn spec_spawn_wait_collects_exit_code() {
    let proc = spawn(&["sh", "-c", "exit 42"], Default::default()).unwrap();
    let r = proc.wait().unwrap();
    assert_eq!(r.exit_code, 42);
    assert!(!r.success);
}

#[test]
fn spec_spawn_wait_collects_stderr() {
    let proc = spawn(&["sh", "-c", "echo err >&2; exit 1"], Default::default()).unwrap();
    let r = proc.wait().unwrap();
    assert_eq!(r.exit_code, 1);
    assert_eq!(r.stderr, b"err\n".to_vec());
}

#[test]
fn spec_spawn_kill_terminates() {
    let mut proc = spawn(&["sh", "-c", "sleep 30"], SpawnOptions {
        stdout: StdioMode::Null,
        stderr: StdioMode::Null,
        ..Default::default()
    }).unwrap();
    proc.kill().unwrap();
    let r = proc.wait().unwrap();
    // Killed processes have non-zero exit; exact code depends on platform
    // (signal-killed children expose -1 from std + signal_code).
    assert!(!r.success);
}

#[test]
fn spec_spawn_stdin_text_then_wait() {
    let proc = spawn(&["cat"], SpawnOptions {
        stdin: StdinInput::Text("piped".into()),
        ..Default::default()
    }).unwrap();
    let r = proc.wait().unwrap();
    assert_eq!(r.stdout, b"piped".to_vec());
}

#[test]
fn spec_spawn_empty_args_errors() {
    let r = spawn(&[], Default::default());
    assert!(matches!(r, Err(SpawnError::EmptyArgs)));
}

// ════════════════════ EDGE CASES ════════════════════

#[test]
fn spec_spawn_sync_unknown_program_errors() {
    let r = spawn_sync(&["/nonexistent/program/path"], Default::default());
    assert!(matches!(r, Err(SpawnError::Io(_))));
}

#[test]
fn spec_spawn_sync_multiline_stdout_preserved() {
    let r = spawn_sync(&["sh", "-c", "echo line1; echo line2; echo line3"], Default::default()).unwrap();
    let s = String::from_utf8(r.stdout).unwrap();
    assert_eq!(s, "line1\nline2\nline3\n");
}

#[test]
fn spec_spawn_sync_binary_stdout() {
    // Output non-UTF-8 bytes via printf with octal escapes (POSIX-portable).
    let r = spawn_sync(&["sh", "-c", r"printf '\000\001\377'"], Default::default()).unwrap();
    assert_eq!(r.stdout, vec![0u8, 1, 0xFF]);
}
