//! `rupa`: argv dispatch for the client, daemon and CLI roles.

mod cli;
mod logging;
mod panic_hook;

use std::process::ExitCode;

use rupa_platform::config::{Config, DEFAULT_CONFIG_TOML};
use rupa_platform::paths::Paths;

use crate::cli::{Cli, Command, ConfigCommand};

/// `rupa 0.0.0 (a1b2c3d4e5f6, x86_64-pc-windows-msvc)`. Commit and target come from build.rs.
const VERSION_LINE: &str = concat!(
    "rupa ",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("RUPA_COMMIT"),
    ", ",
    env!("RUPA_TARGET"),
    ")"
);

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("rupa: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<u8> {
    // shortcut: hand-parsed argv; switch to clap when a command needs options, help or
    // completions. `--version` is handled before any setup so it works (and changes nothing)
    // even on a broken machine: no folders, no log file.
    if matches!(std::env::args().nth(1).as_deref(), Some("--version" | "-V")) {
        println!("{VERSION_LINE}");
        return Ok(0);
    }

    let cli = match cli::parse(std::env::args().skip(1)) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("rupa: {message}");
            eprintln!("{}", cli::USAGE);
            return Ok(2);
        }
    };

    // `config default` only prints the built-in file, so it works even when no directory can be
    // resolved (a broken home, a fresh machine).
    if cli.command == Some(Command::Config(ConfigCommand::Default)) {
        print!("{DEFAULT_CONFIG_TOML}");
        return Ok(0);
    }

    let paths = Paths::from_env()?;

    if cli.command == Some(Command::Config(ConfigCommand::Path)) {
        println!("{}", paths.config_file().display());
        return Ok(0);
    }

    let rupa_log = std::env::var("RUPA_LOG").ok();
    let _log = logging::init(&paths.logs_dir(), rupa_log.as_deref())?;
    panic_hook::install(paths.state_dir.clone());

    if cli.command == Some(Command::Config(ConfigCommand::Check)) {
        return Ok(check(&paths, &cli));
    }

    let config = match Config::load(
        &paths.config_file(),
        |key| std::env::var_os(key),
        &cli.overrides,
    ) {
        Ok(loaded) => {
            for warning in &loaded.warnings {
                tracing::warn!(key = %warning, "unknown config key");
            }
            loaded.config
        }
        Err(err) => {
            // A typo in the file must not brick the daemon: report it and run on the defaults.
            // `rupa config check` is where an invalid file is an error (exit 1).
            tracing::error!(
                path = %paths.config_file().display(),
                error = %err,
                "config is invalid; using defaults"
            );
            // Drop only the broken file: `RUPA_SESSION` and `--session` still apply.
            Config::resolve(None, |key| std::env::var_os(key), &cli.overrides)
                .map(|resolved| resolved.config)
                .unwrap_or_default()
        }
    };

    tracing::info!(
        version = VERSION_LINE,
        session = %config.server.session,
        "rupa starting"
    );
    Ok(0)
}

/// `rupa config check`: a CLI command, so warnings also go to stdout. Exit 1 on an invalid file.
fn check(paths: &Paths, cli: &Cli) -> u8 {
    let path = paths.config_file();
    match Config::load(&path, |key| std::env::var_os(key), &cli.overrides) {
        Ok(loaded) => {
            for warning in &loaded.warnings {
                println!("warning: unknown key `{warning}`");
                tracing::warn!(key = %warning, "unknown config key");
            }
            if loaded.read {
                println!("ok: {}", loaded.path.display());
            } else {
                println!("ok: {} (not found; using defaults)", loaded.path.display());
            }
            0
        }
        Err(err) => {
            eprintln!("rupa: {}: {err}", path.display());
            1
        }
    }
}
