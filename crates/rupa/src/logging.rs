//! File logging. Rupa never logs to stdout or stderr: in TUI mode the screen belongs to the
//! UI, and one stray line would smear over it. Logs go to `<state_dir>/logs/`.
//!
//! Logs hold ids, states and rule names, never pane contents (they may contain secrets).

use std::path::Path;

use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;

/// Level used when `RUPA_LOG` is unset or unreadable.
pub const DEFAULT_LEVEL: &str = "info";

/// Keep the returned guard alive for the whole program: dropping it flushes the last lines.
#[must_use = "dropping the guard stops logging"]
pub struct LogGuard(#[allow(dead_code)] WorkerGuard);

/// Install the file logger for this process. `logs_dir` is `Paths::logs_dir()`; `rupa_log`
/// is the value of `RUPA_LOG`.
pub fn init(logs_dir: &Path, rupa_log: Option<&str>) -> anyhow::Result<LogGuard> {
    let (subscriber, guard) = build(logs_dir, rupa_log)?;
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(LogGuard(guard))
}

/// How many daily log files to keep before the oldest is deleted.
const KEEP_DAYS: usize = 7;

fn build(
    logs_dir: &Path,
    rupa_log: Option<&str>,
) -> anyhow::Result<(impl Subscriber + Send + Sync, WorkerGuard)> {
    std::fs::create_dir_all(logs_dir)?;

    // One file per day (rupa.2026-11-22.log), keeping a week: logs never fill the disk.
    let file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("rupa")
        .filename_suffix("log")
        .max_log_files(KEEP_DAYS)
        .build(logs_dir)?;
    // A background thread does the disk writes, so logging never stalls the caller.
    let (writer, guard) = tracing_appender::non_blocking(file);

    // A bad RUPA_LOG value falls back to the default instead of stopping Rupa
    // (unknown input is ignored, not fatal).
    let filter = rupa_log
        .and_then(|v| EnvFilter::try_new(v).ok())
        .unwrap_or_else(|| EnvFilter::new(DEFAULT_LEVEL));

    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_env_filter(filter)
        .finish();
    Ok((subscriber, guard))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Everything written to `<state>/logs/`, across all files.
    fn read_logs(state: &Path) -> String {
        let Ok(entries) = fs::read_dir(state.join("logs")) else {
            return String::new();
        };
        entries
            .map(|e| fs::read_to_string(e.expect("entry").path()).expect("read log"))
            .collect()
    }

    /// Log through a subscriber built for `state`, then flush by dropping the guard.
    fn log_with(state: &Path, rupa_log: Option<&str>, emit: impl FnOnce()) {
        let (subscriber, guard) = build(&state.join("logs"), rupa_log).expect("build logger");
        tracing::subscriber::with_default(subscriber, emit);
        drop(guard);
    }

    #[test]
    fn writes_into_state_logs() {
        let state = tempfile::tempdir().expect("tempdir");
        log_with(state.path(), None, || {
            tracing::info!(pane = 7, "pane started")
        });
        let logs = read_logs(state.path());
        assert!(logs.contains("pane started"), "logs were: {logs:?}");
        assert!(logs.contains("pane=7"), "fields are kept: {logs:?}");
    }

    #[test]
    fn default_level_hides_debug() {
        let state = tempfile::tempdir().expect("tempdir");
        log_with(state.path(), None, || tracing::debug!("noisy detail"));
        assert!(!read_logs(state.path()).contains("noisy detail"));
    }

    #[test]
    fn rupa_log_debug_shows_debug() {
        let state = tempfile::tempdir().expect("tempdir");
        log_with(state.path(), Some("debug"), || {
            tracing::debug!("noisy detail")
        });
        assert!(read_logs(state.path()).contains("noisy detail"));
    }

    #[test]
    fn log_files_have_no_color_codes() {
        // ANSI colors are for screens. In a file they are junk like "\x1b[32m".
        let state = tempfile::tempdir().expect("tempdir");
        log_with(state.path(), None, || tracing::info!("plain"));
        assert!(!read_logs(state.path()).contains('\x1b'));
    }
}
