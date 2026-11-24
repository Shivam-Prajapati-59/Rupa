//! `rupa --version` prints `rupa <version> (<commit>, <target>)` and touches nothing else.

use std::process::Command;

fn run_version(flag: &str, state_dir: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rupa"))
        .arg(flag)
        .env("RUPA_STATE_DIR", state_dir)
        .env("RUPA_CONFIG_DIR", state_dir.join("config"))
        .output()
        .expect("run rupa")
}

#[test]
fn version_prints_version_commit_and_target() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = run_version("--version", &tmp.path().join("state"));
    assert!(out.status.success(), "exit: {:?}", out.status);
    assert!(
        out.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let line = String::from_utf8(out.stdout).expect("utf-8");
    let line = line.trim_end();
    let prefix = format!("rupa {} (", env!("CARGO_PKG_VERSION"));
    let inner = line
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or_else(|| panic!("unexpected format: {line:?}"));
    let (commit, target) = inner
        .split_once(", ")
        .unwrap_or_else(|| panic!("want '<commit>, <target>': {line:?}"));

    assert!(
        commit == "unknown"
            || (commit.len() == 12 && commit.chars().all(|c| c.is_ascii_hexdigit())),
        "commit should be 12 hex chars or 'unknown': {commit:?}"
    );
    assert!(
        target.starts_with(std::env::consts::ARCH),
        "target for this CPU: {target:?}"
    );
}

#[test]
fn short_flag_works_too() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = run_version("-V", &tmp.path().join("state"));
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("rupa "));
}

#[test]
fn version_does_not_create_folders_or_logs() {
    // Printing a version must be side-effect free (it also runs on broken setups).
    let tmp = tempfile::tempdir().expect("tempdir");
    let state = tmp.path().join("state");
    let out = run_version("--version", &state);
    assert!(out.status.success());
    assert!(!state.exists(), "--version created {state:?}");
}
