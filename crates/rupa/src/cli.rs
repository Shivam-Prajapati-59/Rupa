//! Hand-parsed argv. One command so far (`config`), plus the global `--session`.
//!
//! `--version`/`-V` is handled in `main` before this module, so it stays side-effect free.

use rupa_platform::config::CliOverrides;

pub const USAGE: &str = "usage: rupa [--session <name>] [config <default|path|check>]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCommand {
    Default,
    Path,
    Check,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Config(ConfigCommand),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cli {
    /// `None` means "attach", the default role.
    pub command: Option<Command>,
    pub overrides: CliOverrides,
}

/// Parse argv without the program name. Errors are usage messages (exit code 2).
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
    let mut cli = Cli::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--session" => {
                let name = args.next().ok_or("--session needs a name")?;
                set_session(&mut cli, name)?;
            }
            _ if arg.starts_with("--session=") => {
                set_session(&mut cli, arg["--session=".len()..].to_string())?;
            }
            "config" => {
                let sub = args.next().ok_or("config needs default, path or check")?;
                let command = match sub.as_str() {
                    "default" => ConfigCommand::Default,
                    "path" => ConfigCommand::Path,
                    "check" => ConfigCommand::Check,
                    other => return Err(format!("unknown config subcommand `{other}`")),
                };
                if cli.command.is_some() {
                    return Err("only one command is allowed".to_string());
                }
                cli.command = Some(Command::Config(command));
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(cli)
}

fn set_session(cli: &mut Cli, name: String) -> Result<(), String> {
    if name.is_empty() {
        return Err("--session needs a name".to_string());
    }
    cli.overrides.session = Some(name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Cli, String> {
        parse(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn parses_config_subcommands() {
        for (name, expected) in [
            ("default", ConfigCommand::Default),
            ("path", ConfigCommand::Path),
            ("check", ConfigCommand::Check),
        ] {
            let cli = parse_args(&["config", name]).expect("valid");
            assert_eq!(cli.command, Some(Command::Config(expected)));
        }
    }

    #[test]
    fn parses_session_in_both_forms() {
        let spaced = parse_args(&["--session", "work", "config", "path"]).expect("valid");
        let joined = parse_args(&["--session=work", "config", "path"]).expect("valid");
        assert_eq!(spaced.overrides.session.as_deref(), Some("work"));
        assert_eq!(spaced, joined);
    }

    #[test]
    fn no_arguments_means_attach() {
        let cli = parse_args(&[]).expect("valid");
        assert_eq!(cli.command, None);
        assert_eq!(cli.overrides.session, None);
    }

    #[test]
    fn unknown_arguments_are_usage_errors() {
        assert!(parse_args(&["config"]).is_err(), "missing subcommand");
        assert!(parse_args(&["config", "nope"]).is_err(), "bad subcommand");
        assert!(parse_args(&["nope"]).is_err(), "bad argument");
        assert!(parse_args(&["--session"]).is_err(), "missing value");
        assert!(parse_args(&["--session="]).is_err(), "empty value");
        assert!(
            parse_args(&["config", "path", "config", "check"]).is_err(),
            "two commands"
        );
    }
}
