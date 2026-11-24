//! `rupa config default | path | check`, run through the real binary.

use std::path::Path;
use std::process::{Command, Output};

fn run(tmp: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rupa"))
        .args(args)
        .env("RUPA_CONFIG_DIR", tmp.join("config"))
        .env("RUPA_STATE_DIR", tmp.join("state"))
        .env("RUPA_RUNTIME_DIR", tmp.join("run"))
        .env_remove("RUPA_LOG")
        .env_remove("RUPA_SESSION")
        .output()
        .expect("run rupa")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn config_default_prints_the_file_and_creates_nothing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = run(tmp.path(), &["config", "default"]);
    assert!(out.status.success(), "exit: {:?}", out.status);
    assert!(out.stderr.is_empty(), "stderr: {}", stderr(&out));

    let text = stdout(&out);
    assert!(text.starts_with("[server]"), "stdout: {text:?}");
    assert!(text.contains("[[agents]]"), "stdout: {text:?}");
    assert!(
        !tmp.path().join("config").exists(),
        "printing the default must not create the config dir"
    );
    assert!(
        !tmp.path().join("state").exists(),
        "printing the default must not create the state dir"
    );
}

#[test]
fn config_path_prints_the_file_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = run(tmp.path(), &["config", "path"]);
    assert!(out.status.success(), "exit: {:?}", out.status);
    let expected = tmp.path().join("config").join("config.toml");
    assert_eq!(stdout(&out).trim_end(), expected.display().to_string());
    assert!(!expected.exists(), "`config path` only reports the path");
}

#[test]
fn config_check_warns_on_unknown_keys_but_exits_zero() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("config")).expect("config dir");
    std::fs::write(
        tmp.path().join("config").join("config.toml"),
        "[ui]\ncolour = \"red\"\n",
    )
    .expect("write config");

    let out = run(tmp.path(), &["config", "check"]);
    assert!(out.status.success(), "exit: {:?}", out.status);
    let text = stdout(&out);
    assert!(
        text.contains("warning: unknown key `ui.colour`"),
        "stdout: {text:?}"
    );
    assert!(text.contains("ok: "), "stdout: {text:?}");
}

#[test]
fn config_check_reports_a_syntax_error_with_line_and_column() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("config")).expect("config dir");
    std::fs::write(
        tmp.path().join("config").join("config.toml"),
        "[ui]\ntheme = \"dark\n",
    )
    .expect("write config");

    let out = run(tmp.path(), &["config", "check"]);
    assert_eq!(out.status.code(), Some(1), "exit: {:?}", out.status);
    let err = stderr(&out);
    assert!(err.contains("line 2, column 14"), "stderr: {err:?}");
}

#[test]
fn an_invalid_file_does_not_stop_startup() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("config")).expect("config dir");
    std::fs::write(
        tmp.path().join("config").join("config.toml"),
        "[ui]\ntheme = \"dark\n",
    )
    .expect("write config");

    let out = run(tmp.path(), &[]);
    assert!(out.status.success(), "exit: {:?}", out.status);
    assert!(out.stdout.is_empty(), "stdout: {}", stdout(&out));
    assert!(out.stderr.is_empty(), "stderr: {}", stderr(&out));

    let logs = read_logs(&tmp.path().join("state"));
    assert!(logs.contains("config is invalid"), "logs: {logs:?}");
    assert!(logs.contains("line 2"), "logs: {logs:?}");
}

#[test]
fn an_invalid_file_still_keeps_the_cli_session() {
    // Falling back to defaults must drop only the broken file, not `--session`.
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("config")).expect("config dir");
    std::fs::write(
        tmp.path().join("config").join("config.toml"),
        "[ui]\ntheme = \"dark\n",
    )
    .expect("write config");

    let out = run(tmp.path(), &["--session=work"]);
    assert!(out.status.success(), "exit: {:?}", out.status);

    let logs = read_logs(&tmp.path().join("state"));
    assert!(logs.contains("config is invalid"), "logs: {logs:?}");
    assert!(logs.contains("session=work"), "logs: {logs:?}");
}

#[test]
fn cli_session_beats_the_environment() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = Command::new(env!("CARGO_BIN_EXE_rupa"))
        .args(["--session", "cli-session"])
        .env("RUPA_CONFIG_DIR", tmp.path().join("config"))
        .env("RUPA_STATE_DIR", tmp.path().join("state"))
        .env("RUPA_RUNTIME_DIR", tmp.path().join("run"))
        .env("RUPA_SESSION", "env-session")
        .env_remove("RUPA_LOG")
        .output()
        .expect("run rupa");
    assert!(out.status.success(), "exit: {:?}", out.status);

    let logs = read_logs(&tmp.path().join("state"));
    assert!(logs.contains("session=cli-session"), "logs: {logs:?}");
    assert!(!logs.contains("session=env-session"), "logs: {logs:?}");
}

#[test]
fn unknown_arguments_exit_with_the_usage_code() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = run(tmp.path(), &["config", "nope"]);
    assert_eq!(out.status.code(), Some(2), "exit: {:?}", out.status);
    let err = stderr(&out);
    assert!(err.contains("unknown config subcommand"), "stderr: {err:?}");
    assert!(err.contains("usage: rupa"), "stderr: {err:?}");
}

fn read_logs(state: &Path) -> String {
    let Ok(entries) = std::fs::read_dir(state.join("logs")) else {
        return String::new();
    };
    entries
        .map(|entry| std::fs::read_to_string(entry.expect("entry").path()).expect("read log"))
        .collect()
}
