//! Runs the real `rupa` binary: it must log to a file and keep the screen untouched.

use std::process::Command;

#[test]
fn startup_logs_to_a_file_and_prints_nothing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = Command::new(env!("CARGO_BIN_EXE_rupa"))
        .env("RUPA_CONFIG_DIR", tmp.path().join("config"))
        .env("RUPA_STATE_DIR", tmp.path().join("state"))
        .env("RUPA_RUNTIME_DIR", tmp.path().join("run"))
        .env_remove("RUPA_LOG")
        .output()
        .expect("run rupa");

    assert!(out.status.success(), "exit: {:?}", out.status);
    assert!(
        out.stdout.is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let logs: String = std::fs::read_dir(tmp.path().join("state").join("logs"))
        .expect("logs folder exists")
        .map(|e| std::fs::read_to_string(e.expect("entry").path()).expect("read"))
        .collect();
    assert!(logs.contains("rupa starting"), "logs: {logs:?}");
}
