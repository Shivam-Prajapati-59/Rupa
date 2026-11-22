//! Crash politely: on a panic, give the user their terminal back first, then write a crash
//! report to `<state_dir>/crash-<ms>-<pid>.txt`. The report stays on this machine (no upload).

use std::backtrace::Backtrace;
use std::io::Write;
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::Command;
use crossterm::cursor::Show;
use crossterm::terminal::LeaveAlternateScreen;

/// Install the hook for the whole process. Call once, early in `main`.
pub fn install(state_dir: PathBuf) {
    std::panic::set_hook(Box::new(move |info| {
        // 1. The user's terminal matters more than our report, so it goes first: if
        //    anything below fails, they still get a working shell back.
        restore_terminal();

        // 2. The report: to a file if we can, otherwise straight to stderr so it isn't lost.
        let report = crash_report(info);
        tracing::error!(location = ?info.location(), "panic");
        match write_crash_file(&state_dir, &report) {
            Ok(path) => eprintln!("rupa crashed. Details: {}", path.display()),
            Err(err) => eprintln!("rupa crashed (crash file not written: {err})\n{report}"),
        }
    }));
}

/// Undo what the TUI does to the terminal: raw mode, alternate screen, hidden cursor.
/// Unconditional and best effort: undoing a mode that was never on is harmless, and a
/// panic hook has nowhere to report its own errors.
fn restore_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(restore_sequence().as_bytes());
    let _ = stdout.flush();
}

/// The escape codes, written as plain text instead of through crossterm's `execute!`: on
/// Windows `execute!` may switch to console API calls, which do nothing when stdout is
/// not a console (tests). Plain codes behave the same everywhere.
fn restore_sequence() -> String {
    let mut s = String::new();
    let _ = LeaveAlternateScreen.write_ansi(&mut s);
    let _ = Show.write_ansi(&mut s);
    s
}

fn crash_report(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "(no message)".to_string());
    let location = info
        .location()
        .map_or_else(|| "(unknown)".to_string(), ToString::to_string);

    format!(
        "Rupa crashed.\n\
         version:  {}\n\
         os:       {} {}\n\
         message:  {message}\n\
         location: {location}\n\
         \n\
         backtrace:\n{}\n",
        crate::VERSION_LINE,
        std::env::consts::OS,
        std::env::consts::ARCH,
        Backtrace::force_capture(),
    )
}

/// `crash-<unix ms>-<pid>.txt`: the pid keeps two processes crashing in the same
/// millisecond (daemon and client) from overwriting each other.
fn write_crash_file(state_dir: &Path, report: &str) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(state_dir)?;
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let path = state_dir.join(format!("crash-{ms}-{}.txt", std::process::id()));
    std::fs::write(&path, report)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Output};

    const CHILD_ENV: &str = "RUPA_TEST_PANIC_CHILD";
    const LEAVE_ALT_SCREEN: &str = "\x1b[?1049l";
    const SHOW_CURSOR: &str = "\x1b[?25h";

    /// Not a real test on its own: does nothing unless the parent test below started us.
    #[test]
    fn child_installs_hook_and_panics() {
        if std::env::var_os(CHILD_ENV).is_none() {
            return;
        }
        let state = std::env::var_os("RUPA_STATE_DIR").expect("parent sets RUPA_STATE_DIR");
        install(PathBuf::from(state));
        // Deliberate: this panic is what the parent test inspects.
        panic!("forced test panic");
    }

    /// Re-run this test binary, executing only the child test above.
    fn run_child(state_dir: &std::path::Path) -> Output {
        Command::new(std::env::current_exe().expect("path of this test binary"))
            .args([
                "--exact",
                "panic_hook::tests::child_installs_hook_and_panics",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_ENV, "1")
            .env("RUPA_STATE_DIR", state_dir)
            .output()
            .expect("run child")
    }

    fn crash_files(dir: &std::path::Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        entries
            .map(|e| e.expect("entry").path())
            .filter(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                name.starts_with("crash-") && name.ends_with(".txt")
            })
            .collect()
    }

    /// The P0-S4 "Done when": a forced panic leaves a crash file and a sane terminal.
    #[test]
    fn forced_panic_leaves_a_crash_file_and_restores_the_terminal() {
        let state = tempfile::tempdir().expect("tempdir");
        let out = run_child(state.path());
        let stdout = String::from_utf8_lossy(&out.stdout);

        assert!(!out.status.success(), "the child must die from its panic");
        assert!(
            stdout.contains(LEAVE_ALT_SCREEN),
            "alternate screen left: {stdout:?}"
        );
        assert!(stdout.contains(SHOW_CURSOR), "cursor shown: {stdout:?}");

        let files = crash_files(state.path());
        assert_eq!(files.len(), 1, "exactly one crash file: {files:?}");
        let report = std::fs::read_to_string(&files[0]).expect("read crash file");
        assert!(report.contains("forced test panic"), "message: {report}");
        assert!(report.contains("panic_hook.rs"), "location: {report}");
        assert!(
            report.contains(env!("CARGO_PKG_VERSION")),
            "version: {report}"
        );
    }

    /// Restore comes first: even when the crash file can't be written, the terminal is
    /// fixed and the report still reaches the user on stderr.
    #[test]
    fn terminal_is_restored_even_if_the_crash_file_cannot_be_written() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let not_a_dir = tmp.path().join("a-file");
        std::fs::write(&not_a_dir, "").expect("create a file where the folder should be");

        let out = run_child(&not_a_dir);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);

        assert!(
            stdout.contains(LEAVE_ALT_SCREEN),
            "restored anyway: {stdout:?}"
        );
        assert!(stdout.contains(SHOW_CURSOR), "restored anyway: {stdout:?}");
        assert!(
            stderr.contains("forced test panic"),
            "report on stderr: {stderr}"
        );
    }
}
