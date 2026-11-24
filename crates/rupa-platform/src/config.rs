//! `config.toml`: the built-in defaults, the user file, `RUPA_*` overrides and CLI flags.
//!
//! Everything is pure except [`Config::load`], which reads one file. Tests pass file contents
//! and a fake environment lookup, as `paths` does, so they never touch the process environment.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The file `rupa config default` prints; it is also the built-in default (a test pins that).
pub const DEFAULT_CONFIG_TOML: &str = r#"[server]
session = "default"
startup_per_agent_delay_ms = 100
max_clients = 16

[ui]
theme = "dark"                  # dark | light | high-contrast
sidebar_min_width = 18
sidebar_max_width = 36
mouse = true
render_min_interval_ms = 16

[keys]
prefix = ["ctrl+b"]            # a list; Claude Code binds ctrl+b to "background this command", so double-tap sends it through, or add e.g. "ctrl+space"
# new_tab = "prefix+c"

[terminal]
scrollback_lines = 2000         # P1-S8 may change the default
paste_limit_bytes = 1048576

[clipboard]
osc52_from_panes = "write"      # write | off
prefer_osc52 = false

[detect]
tail_rows = 24
tick_coalesce_ms = 100
idle_confirmations = 3
idle_confirm_interval_ms = 100
idle_confirm_cap_ms = 700
blocked_confirmations = 2
hook_stale_ms = 5000
blocked_clear_ms = 1500
manifest_dir = ""               # default: <config_dir>/manifests

[notify]
desktop = true
bell = true
osc = true
rate_limit_s = 30
muted = false

[worktrees]
directory = "~/.rupa/worktrees"
branch_prefix = "task/"
port_base = 21000
port_block = 20

[[agents]]
name = "claude"
command = "claude"
args = []
env = {}
resume = "claude --resume {id}"     # verified in P5-S6

[[agents]]
name = "codex"
command = "codex"
resume = "codex resume {id}"        # verify

[[agents]]
name = "opencode"
command = "opencode"
resume = "opencode --session {id}"  # verify
"#;

/// Everything in `config.toml`. Each section has `#[serde(default)]`, so a file only needs the
/// keys it changes; a section that is present keeps the defaults for its missing keys.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub ui: UiConfig,
    pub keys: KeysConfig,
    pub terminal: TerminalConfig,
    pub clipboard: ClipboardConfig,
    pub detect: DetectConfig,
    pub notify: NotifyConfig,
    pub worktrees: WorktreeConfig,
    /// A present `[[agents]]` list replaces the built-in one; it is not merged entry by entry.
    pub agents: Vec<AgentConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            ui: UiConfig::default(),
            keys: KeysConfig::default(),
            terminal: TerminalConfig::default(),
            clipboard: ClipboardConfig::default(),
            detect: DetectConfig::default(),
            notify: NotifyConfig::default(),
            worktrees: WorktreeConfig::default(),
            agents: default_agents(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct ServerConfig {
    pub session: String,
    pub startup_per_agent_delay_ms: u64,
    pub max_clients: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            session: "default".to_string(),
            startup_per_agent_delay_ms: 100,
            max_clients: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct UiConfig {
    pub theme: String,
    pub sidebar_min_width: u16,
    pub sidebar_max_width: u16,
    pub mouse: bool,
    pub render_min_interval_ms: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            sidebar_min_width: 18,
            sidebar_max_width: 36,
            mouse: true,
            render_min_interval_ms: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct KeysConfig {
    pub prefix: Vec<String>,
}

impl Default for KeysConfig {
    fn default() -> Self {
        Self {
            prefix: vec!["ctrl+b".to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct TerminalConfig {
    pub scrollback_lines: usize,
    pub paste_limit_bytes: usize,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: 2000,
            paste_limit_bytes: 1_048_576,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct ClipboardConfig {
    /// `write` or `off`. Kept as text: an unknown value is a later slice's validation, not a
    /// parse error (unknown input is ignored, not fatal).
    pub osc52_from_panes: String,
    pub prefer_osc52: bool,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            osc52_from_panes: "write".to_string(),
            prefer_osc52: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct DetectConfig {
    pub tail_rows: usize,
    pub tick_coalesce_ms: u64,
    pub idle_confirmations: u32,
    pub idle_confirm_interval_ms: u64,
    pub idle_confirm_cap_ms: u64,
    pub blocked_confirmations: u32,
    pub hook_stale_ms: u64,
    pub blocked_clear_ms: u64,
    /// Empty means `<config_dir>/manifests`.
    pub manifest_dir: String,
}

impl Default for DetectConfig {
    fn default() -> Self {
        Self {
            tail_rows: 24,
            tick_coalesce_ms: 100,
            idle_confirmations: 3,
            idle_confirm_interval_ms: 100,
            idle_confirm_cap_ms: 700,
            blocked_confirmations: 2,
            hook_stale_ms: 5000,
            blocked_clear_ms: 1500,
            manifest_dir: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct NotifyConfig {
    pub desktop: bool,
    pub bell: bool,
    pub osc: bool,
    pub rate_limit_s: u64,
    pub muted: bool,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            desktop: true,
            bell: true,
            osc: true,
            rate_limit_s: 30,
            muted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct WorktreeConfig {
    pub directory: String,
    pub branch_prefix: String,
    pub port_base: u16,
    pub port_block: u16,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            directory: "~/.rupa/worktrees".to_string(),
            branch_prefix: "task/".to_string(),
            port_base: 21000,
            port_block: 20,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(default)]
pub struct AgentConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    /// Unverified until P5-S6; `None` means "no resume template".
    pub resume: Option<String>,
}

fn default_agents() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            name: "claude".to_string(),
            command: "claude".to_string(),
            resume: Some("claude --resume {id}".to_string()),
            ..AgentConfig::default()
        },
        AgentConfig {
            name: "codex".to_string(),
            command: "codex".to_string(),
            resume: Some("codex resume {id}".to_string()),
            ..AgentConfig::default()
        },
        AgentConfig {
            name: "opencode".to_string(),
            command: "opencode".to_string(),
            resume: Some("opencode --session {id}".to_string()),
            ..AgentConfig::default()
        },
    ]
}

/// The config layers that come from the command line (currently just the global `--session`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliOverrides {
    pub session: Option<String>,
}

/// The result of merging all four layers, without touching the file system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub config: Config,
    /// Dotted paths of unknown keys, e.g. `ui.colour` or `agents[0].colour`.
    pub warnings: Vec<String>,
}

/// The result of [`Config::load`]: the config plus where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    pub config: Config,
    pub warnings: Vec<String>,
    /// `<config_dir>/config.toml`, whether or not it exists.
    pub path: PathBuf,
    /// Whether the file existed and was read.
    pub read: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("line {line}, column {column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("{path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Config {
    /// Merge the layers: built-in defaults, the file (when present), `RUPA_*`, then CLI flags.
    /// Pure: `file` is the file's contents and `env` is an injected lookup.
    pub fn resolve(
        file: Option<&str>,
        env: impl Fn(&str) -> Option<OsString>,
        cli: &CliOverrides,
    ) -> Result<Resolved, ConfigError> {
        let (mut config, warnings) = match file {
            Some(src) => {
                // Parse once as a table to find unknown keys, once as `Config` so type errors
                // keep their line and column.
                let table =
                    toml::from_str::<toml::Table>(src).map_err(|e| syntax_error(src, &e))?;
                let warnings = unknown_keys(&table);
                let config = toml::from_str::<Config>(src).map_err(|e| syntax_error(src, &e))?;
                (config, warnings)
            }
            None => (Config::default(), Vec::new()),
        };
        config.apply_env(env);
        config.apply_cli(cli);
        Ok(Resolved { config, warnings })
    }

    /// Read the config file (normally [`Paths::config_file`](crate::paths::Paths::config_file))
    /// and resolve every layer. A missing file means defaults; any other read error is reported.
    pub fn load(
        config_file: &Path,
        env: impl Fn(&str) -> Option<OsString>,
        cli: &CliOverrides,
    ) -> Result<Loaded, ConfigError> {
        let path = config_file.to_path_buf();
        let src = match std::fs::read_to_string(&path) {
            Ok(src) => Some(src),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(source) => return Err(ConfigError::Read { path, source }),
        };
        let resolved = Self::resolve(src.as_deref(), env, cli)?;
        Ok(Loaded {
            config: resolved.config,
            warnings: resolved.warnings,
            path,
            read: src.is_some(),
        })
    }

    /// `RUPA_*` overrides. Only variables that map to a config field are applied: `RUPA_SESSION`
    /// is the environment form of `--session` (§5.13, Appendix C).
    fn apply_env(&mut self, env: impl Fn(&str) -> Option<OsString>) {
        // A session name must be text: a non-UTF-8 value is ignored rather than failing the
        // whole config (unlike a path, which may hold any bytes).
        let text = |key| crate::paths::env_value(&env, key).and_then(|v| v.into_string().ok());
        if let Some(session) = text("RUPA_SESSION") {
            self.server.session = session;
        }
    }

    /// CLI flags win over everything else.
    fn apply_cli(&mut self, cli: &CliOverrides) {
        if let Some(session) = cli.session.as_ref().filter(|s| !s.is_empty()) {
            self.server.session.clone_from(session);
        }
    }
}

/// A `toml` error as a line and column. Type errors without a span fall back to 1:1.
fn syntax_error(src: &str, err: &toml::de::Error) -> ConfigError {
    let (line, column) = err
        .span()
        .map(|span| line_col(src, span.start))
        .unwrap_or((1, 1));
    ConfigError::Syntax {
        line,
        column,
        message: err.message().to_string(),
    }
}

fn line_col(src: &str, offset: usize) -> (usize, usize) {
    // `offset` comes from the parser, but never trust external input with a slice index.
    let before = src.get(..offset).unwrap_or(src);
    let line = before.matches('\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        // A "\r" from Windows "\r\n" line endings is not a visible column.
        .map_or(1, |line_text| {
            line_text.trim_end_matches('\r').chars().count() + 1
        });
    (line, column)
}

// shortcut: these lists mirror the structs by hand (serde has no "warn on unknown field"
// hook). The test `known_key_lists_match_the_structs` fails if they drift; upgrade to
// `serde_ignored` if §6.3 ever allows it.
const AGENT: &[&str] = &["name", "command", "args", "env", "resume"];

/// The keys each `[section]` accepts; `None` for a section that doesn't exist.
fn known_keys(section: &str) -> Option<&'static [&'static str]> {
    Some(match section {
        "server" => &["session", "startup_per_agent_delay_ms", "max_clients"],
        "ui" => &[
            "theme",
            "sidebar_min_width",
            "sidebar_max_width",
            "mouse",
            "render_min_interval_ms",
        ],
        "keys" => &["prefix"],
        "terminal" => &["scrollback_lines", "paste_limit_bytes"],
        "clipboard" => &["osc52_from_panes", "prefer_osc52"],
        "detect" => &[
            "tail_rows",
            "tick_coalesce_ms",
            "idle_confirmations",
            "idle_confirm_interval_ms",
            "idle_confirm_cap_ms",
            "blocked_confirmations",
            "hook_stale_ms",
            "blocked_clear_ms",
            "manifest_dir",
        ],
        "notify" => &["desktop", "bell", "osc", "rate_limit_s", "muted"],
        "worktrees" => &["directory", "branch_prefix", "port_base", "port_block"],
        _ => return None,
    })
}

/// Dotted paths of every key the structs do not know. Warnings, never errors (§5.13).
fn unknown_keys(table: &toml::Table) -> Vec<String> {
    let mut out = Vec::new();
    for (name, value) in table {
        if name == "agents" {
            // `[[agents]]` is an array of tables; report the index of the bad entry.
            if let Some(agents) = value.as_array() {
                for (i, agent) in agents.iter().enumerate() {
                    if let Some(agent) = agent.as_table() {
                        for key in agent.keys().filter(|k| !AGENT.contains(&k.as_str())) {
                            out.push(format!("agents[{i}].{key}"));
                        }
                    }
                }
            }
            continue;
        }
        let Some(known) = known_keys(name) else {
            out.push(name.clone());
            continue;
        };
        if let Some(section) = value.as_table() {
            for key in section.keys().filter(|k| !known.contains(&k.as_str())) {
                out.push(format!("{name}.{key}"));
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// A fake environment built from `(key, value)` pairs.
    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let map: HashMap<String, OsString> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), OsString::from(v)))
            .collect();
        move |key| map.get(key).cloned()
    }

    fn resolve(
        src: Option<&str>,
        env: impl Fn(&str) -> Option<OsString>,
    ) -> Result<Resolved, ConfigError> {
        Config::resolve(src, env, &CliOverrides::default())
    }

    #[test]
    fn defaults_match_the_spec() {
        let c = Config::default();
        assert_eq!(c.server.session, "default");
        assert_eq!(c.server.startup_per_agent_delay_ms, 100);
        assert_eq!(c.server.max_clients, 16);
        assert_eq!(c.ui.theme, "dark");
        assert_eq!(c.ui.sidebar_min_width, 18);
        assert_eq!(c.ui.sidebar_max_width, 36);
        assert!(c.ui.mouse);
        assert_eq!(c.ui.render_min_interval_ms, 16);
        assert_eq!(c.keys.prefix, vec!["ctrl+b".to_string()]);
        assert_eq!(c.terminal.scrollback_lines, 2000);
        assert_eq!(c.terminal.paste_limit_bytes, 1_048_576);
        assert_eq!(c.clipboard.osc52_from_panes, "write");
        assert!(!c.clipboard.prefer_osc52);
        assert_eq!(c.detect.tail_rows, 24);
        assert_eq!(c.detect.tick_coalesce_ms, 100);
        assert_eq!(c.detect.hook_stale_ms, 5000);
        assert_eq!(c.notify.rate_limit_s, 30);
        assert!(!c.notify.muted);
        assert_eq!(c.worktrees.port_base, 21000);
        assert_eq!(c.worktrees.port_block, 20);
        let names: Vec<&str> = c.agents.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["claude", "codex", "opencode"]);
    }

    #[test]
    fn default_config_toml_is_the_built_in_default() {
        let resolved =
            resolve(Some(DEFAULT_CONFIG_TOML), env(&[])).expect("the template must parse");
        assert!(
            resolved.warnings.is_empty(),
            "warnings: {:?}",
            resolved.warnings
        );
        assert_eq!(resolved.config, Config::default());
    }

    /// Keys of a value after a round trip through TOML, sorted.
    fn serialized_keys(value: &impl serde::Serialize) -> toml::Table {
        let text = toml::to_string(value).expect("serialize");
        toml::from_str(&text).expect("re-parse")
    }

    fn sorted(keys: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
        let mut v: Vec<String> = keys.into_iter().map(|k| k.as_ref().to_string()).collect();
        v.sort();
        v
    }

    /// Adding a struct field without adding it to `known_keys` would make Rupa warn about
    /// a valid setting. This compares the lists against the structs themselves.
    #[test]
    fn known_key_lists_match_the_structs() {
        let table = serialized_keys(&Config::default());
        for (section, value) in &table {
            if section == "agents" {
                continue;
            }
            let actual = sorted(value.as_table().expect("a section is a table").keys());
            let known =
                sorted(known_keys(section).unwrap_or_else(|| panic!("[{section}] missing")));
            assert_eq!(
                actual, known,
                "known_keys(\"{section}\") drifted from the struct"
            );
        }
        // `resume` is an Option and is skipped when None, so fill every field.
        let agent = AgentConfig {
            resume: Some(String::new()),
            ..AgentConfig::default()
        };
        assert_eq!(sorted(serialized_keys(&agent).keys()), sorted(AGENT));
    }

    #[test]
    fn file_values_override_defaults_and_keep_the_rest() {
        let src = "[server]\nsession = \"work\"\n\n[ui]\ntheme = \"light\"\n";
        let c = resolve(Some(src), env(&[])).expect("valid").config;
        assert_eq!(c.server.session, "work");
        assert_eq!(c.ui.theme, "light");
        assert_eq!(c.server.max_clients, 16, "untouched defaults stay");
        assert_eq!(c.ui.sidebar_min_width, 18);
    }

    #[test]
    fn env_overrides_the_file() {
        let c = resolve(
            Some("[server]\nsession = \"file\"\n"),
            env(&[("RUPA_SESSION", "env")]),
        )
        .expect("valid")
        .config;
        assert_eq!(c.server.session, "env");
    }

    #[test]
    fn cli_overrides_env() {
        let cli = CliOverrides {
            session: Some("cli".to_string()),
        };
        let c = Config::resolve(
            Some("[server]\nsession = \"file\"\n"),
            env(&[("RUPA_SESSION", "env")]),
            &cli,
        )
        .expect("valid")
        .config;
        assert_eq!(c.server.session, "cli");
    }

    #[test]
    fn empty_env_value_counts_as_unset() {
        let c = resolve(
            Some("[server]\nsession = \"file\"\n"),
            env(&[("RUPA_SESSION", "")]),
        )
        .expect("valid")
        .config;
        assert_eq!(c.server.session, "file");
    }

    #[test]
    fn unknown_keys_warn_and_are_ignored() {
        let resolved = resolve(Some("[ui]\ncolour = \"red\"\n"), env(&[]))
            .expect("unknown keys are not fatal");
        assert_eq!(resolved.warnings, vec!["ui.colour".to_string()]);
        assert_eq!(resolved.config, Config::default());
    }

    #[test]
    fn unknown_agent_keys_warn_too() {
        let src = "[[agents]]\nname = \"x\"\ncommand = \"x\"\ncolour = \"red\"\n";
        let resolved = resolve(Some(src), env(&[])).expect("valid");
        assert_eq!(resolved.warnings, vec!["agents[0].colour".to_string()]);
    }

    #[test]
    fn syntax_error_reports_line_and_column() {
        let err =
            resolve(Some("[ui]\ntheme = \"dark\n"), env(&[])).expect_err("unterminated string");
        match err {
            ConfigError::Syntax {
                line,
                column,
                message,
            } => {
                assert_eq!(
                    (line, column),
                    (2, 14),
                    "the end of the unterminated string"
                );
                assert!(!message.is_empty());
            }
            other => panic!("want a syntax error, got {other:?}"),
        }
    }

    #[test]
    fn syntax_error_column_ignores_windows_line_endings() {
        // Files saved on Windows end lines with "\r\n"; the invisible "\r" is not a column.
        let err =
            resolve(Some("[ui]\r\ntheme = \"dark\r\n"), env(&[])).expect_err("unterminated string");
        assert!(
            matches!(
                err,
                ConfigError::Syntax {
                    line: 2,
                    column: 14,
                    ..
                }
            ),
            "{err}"
        );
    }

    #[test]
    fn missing_file_means_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        let loaded = Config::load(
            &dir.path().join("config.toml"),
            env(&[]),
            &CliOverrides::default(),
        )
        .expect("a missing file is fine");
        assert!(!loaded.read);
        assert_eq!(loaded.path, dir.path().join("config.toml"));
        assert_eq!(loaded.config, Config::default());
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn load_reads_config_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("config.toml"), "[ui]\ntheme = \"light\"\n")
            .expect("write config");
        let loaded = Config::load(
            &dir.path().join("config.toml"),
            env(&[]),
            &CliOverrides::default(),
        )
        .expect("valid");
        assert!(loaded.read);
        assert_eq!(loaded.config.ui.theme, "light");
    }
}
