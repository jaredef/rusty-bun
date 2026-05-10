// Consumer-regression suite for Bun.spawn.

use rusty_bun_spawn::*;

// ────────── Bun.serve dev workflow — shell out to a build tool ──────────
//
// Source: many Bun-using dev workflows shell out to compile or transpile
// before reload. Pattern: `Bun.spawnSync(["tsc", "--noEmit"])`.

#[test]
fn consumer_dev_workflow_check_command_exit_code() {
    let r = spawn_sync(&["sh", "-c", "true"], Default::default()).unwrap();
    assert_eq!(r.exit_code, 0);
    assert!(r.success);

    let r = spawn_sync(&["sh", "-c", "false"], Default::default()).unwrap();
    assert!(!r.success);
}

// ────────── Test runner — capture stdout from a child for snapshot ──────
//
// Source: Bun's own test infrastructure spawns child processes for
// integration tests; expect captured stdout to be byte-equal.

#[test]
fn consumer_test_runner_byte_equal_stdout() {
    let payload = "exact contents that should round-trip";
    let r = spawn_sync(
        &["sh", "-c", &format!("printf '%s' '{}'", payload)],
        Default::default(),
    ).unwrap();
    assert_eq!(r.stdout, payload.as_bytes());
}

// ────────── Build script — env propagation ──────────
//
// Source: build scripts pass NODE_ENV / BUN_ENV to children. Consumer
// expectation: env values propagate exactly.

#[test]
fn consumer_build_script_env_propagation() {
    let mut env = std::collections::HashMap::new();
    env.insert("NODE_ENV".into(), "production".into());
    env.insert("PATH".into(), std::env::var("PATH").unwrap_or_default());
    let r = spawn_sync(&["sh", "-c", "echo $NODE_ENV"], SpawnOptions {
        env: Some(env),
        ..Default::default()
    }).unwrap();
    assert_eq!(r.stdout, b"production\n");
}

// ────────── CLI tools — stdin piping ──────────
//
// Source: `Bun.spawnSync(["jq", "."], { stdin: jsonText })` is the canonical
// CLI-tool-piping pattern.

#[test]
fn consumer_cli_jq_style_stdin_pipe() {
    let r = spawn_sync(&["cat"], SpawnOptions {
        stdin: StdinInput::Text(r#"{"key":"value"}"#.into()),
        ..Default::default()
    }).unwrap();
    assert_eq!(r.stdout, br#"{"key":"value"}"#.to_vec());
}

// ────────── Process supervision — kill long-running ──────────
//
// Source: dev servers spawn long-running children (file watchers, build
// daemons) and rely on `proc.kill()` to cleanly shut them down on reload.

#[test]
fn consumer_dev_server_kills_child_on_reload() {
    let mut proc = spawn(&["sh", "-c", "sleep 30"], SpawnOptions {
        stdout: StdioMode::Null,
        stderr: StdioMode::Null,
        ..Default::default()
    }).unwrap();
    let pid = proc.pid();
    assert!(pid > 0);
    proc.kill().unwrap();
    let r = proc.wait().unwrap();
    assert!(!r.success);
}

// ────────── Bun docs — basic spawnSync example ──────────
//
// Source: https://bun.sh/docs/api/spawn — canonical example
//   `const result = Bun.spawnSync(["echo", "Hello"]);`
//   `result.stdout.toString() === "Hello\n"`

#[test]
fn consumer_bun_docs_basic_spawnsync() {
    let r = spawn_sync(&["sh", "-c", "echo Hello"], Default::default()).unwrap();
    let stdout_text = String::from_utf8(r.stdout).unwrap();
    assert_eq!(stdout_text, "Hello\n");
}

// ────────── Build tool — cwd directs operation context ──────────
//
// Source: tooling like prettier / eslint runs in a specific cwd to resolve
// config files. Consumer relies on cwd being honored.

#[test]
fn consumer_lint_tool_runs_in_cwd() {
    let r = spawn_sync(&["pwd"], SpawnOptions {
        cwd: Some(std::path::PathBuf::from("/tmp")),
        ..Default::default()
    }).unwrap();
    let out = String::from_utf8(r.stdout).unwrap();
    let trimmed = out.trim();
    assert!(trimmed == "/tmp" || trimmed == "/private/tmp",
        "expected /tmp pwd, got {}", trimmed);
}

// ────────── Error handling — nonzero exit signals failure ──────────
//
// Source: virtually every CI/CD pipeline. Consumer expectation: exit_code
// is precisely the value the child exited with.

#[test]
fn consumer_ci_pipeline_exit_code_precision() {
    for code in [0, 1, 2, 42, 127, 255] {
        let r = spawn_sync(
            &["sh", "-c", &format!("exit {}", code)],
            Default::default(),
        ).unwrap();
        assert_eq!(r.exit_code, code,
            "expected exit code {} got {}", code, r.exit_code);
    }
}
