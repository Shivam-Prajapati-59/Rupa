//! Where Rupa's files live on each OS.
//!
//! - `config_dir`: files the user writes (`config.toml`).
//! - `state_dir`: files Rupa writes (logs, crash files, saved sessions).
//!
//! Each can be overridden with `RUPA_CONFIG_DIR` / `RUPA_STATE_DIR`, which is how tests
//! get isolated folders.

pub mod runtime;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// Resolved directories for one Rupa process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum PathsError {
    #[error("cannot find a home directory; set RUPA_CONFIG_DIR and RUPA_STATE_DIR")]
    NoHome,
}

impl Paths {
    /// Resolve from the real environment and the OS conventions.
    pub fn from_env() -> Result<Self, PathsError> {
        Self::resolve(
            |key| std::env::var_os(key),
            ProjectDirs::from("", "", "rupa"),
        )
    }

    /// Resolve from a given environment lookup and OS defaults. Pure: tests pass a fake
    /// `env` instead of mutating the process environment (which races between tests).
    pub fn resolve(
        env: impl Fn(&str) -> Option<OsString>,
        defaults: Option<ProjectDirs>,
    ) -> Result<Self, PathsError> {
        let config_dir = match non_empty(env("RUPA_CONFIG_DIR")) {
            Some(dir) => dir,
            None => defaults
                .as_ref()
                .ok_or(PathsError::NoHome)?
                .config_dir()
                .to_path_buf(),
        };
        let state_dir = match non_empty(env("RUPA_STATE_DIR")) {
            Some(dir) => dir,
            None => default_state_dir(defaults.as_ref().ok_or(PathsError::NoHome)?),
        };
        Ok(Self {
            config_dir,
            state_dir,
        })
    }
}

/// An empty variable (`RUPA_STATE_DIR=`) means "unset", the usual shell convention.
fn non_empty(value: Option<OsString>) -> Option<PathBuf> {
    value.filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// `directories` has a real state dir only on Linux (`~/.local/state/rupa`). On Windows
/// and macOS it returns `None`, so we use the local (non-roaming) data dir: state is
/// per-machine and must not sync to other PCs through a roaming profile.
fn default_state_dir(dirs: &ProjectDirs) -> PathBuf {
    choose_state_dir(dirs.config_dir(), dirs.state_dir(), dirs.data_local_dir())
}

/// The decision itself, on plain paths so it can be tested with made-up values.
fn choose_state_dir(config: &Path, state: Option<&Path>, data_local: &Path) -> PathBuf {
    let chosen = state.unwrap_or(data_local);
    if chosen == config {
        // macOS: one "Application Support/rupa" serves config and data. Give state a
        // subfolder so wiping state never takes the user's config.toml with it.
        chosen.join("state")
    } else {
        chosen.to_path_buf()
    }
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

    fn os_defaults() -> Option<ProjectDirs> {
        ProjectDirs::from("", "", "rupa")
    }

    #[test]
    fn overrides_win_over_os_defaults() {
        let p = Paths::resolve(
            env(&[
                ("RUPA_CONFIG_DIR", "/x/conf"),
                ("RUPA_STATE_DIR", "/x/state"),
            ]),
            os_defaults(),
        )
        .expect("overrides need no home dir");
        assert_eq!(p.config_dir, PathBuf::from("/x/conf"));
        assert_eq!(p.state_dir, PathBuf::from("/x/state"));
    }

    #[test]
    fn overrides_work_even_without_a_home_dir() {
        let p = Paths::resolve(
            env(&[
                ("RUPA_CONFIG_DIR", "/x/conf"),
                ("RUPA_STATE_DIR", "/x/state"),
            ]),
            None,
        )
        .expect("overrides need no home dir");
        assert_eq!(p.state_dir, PathBuf::from("/x/state"));
    }

    #[test]
    fn empty_override_counts_as_unset() {
        let p = Paths::resolve(env(&[("RUPA_STATE_DIR", "")]), os_defaults())
            .expect("this machine has a home dir");
        assert_ne!(p.state_dir, PathBuf::new());
    }

    #[test]
    fn defaults_follow_os_conventions_and_name_rupa() {
        let p = Paths::resolve(env(&[]), os_defaults()).expect("this machine has a home dir");
        for dir in [&p.config_dir, &p.state_dir] {
            assert!(dir.is_absolute(), "{dir:?} should be absolute");
            assert!(
                dir.components().any(|c| c.as_os_str() == "rupa"),
                "{dir:?} should name rupa"
            );
        }
    }

    #[test]
    fn state_gets_its_own_folder_when_the_os_shares_one_with_config() {
        // macOS: directories returns the same "Application Support/rupa" for both.
        let shared = Path::new("/Users/me/Library/Application Support/rupa");
        let state = choose_state_dir(shared, None, shared);
        assert_ne!(state, shared, "state must not share config's folder");
        assert_eq!(state, shared.join("state"));
    }

    #[test]
    fn state_uses_the_os_state_dir_when_there_is_one() {
        // Linux: a real, separate state dir exists.
        let state = choose_state_dir(
            Path::new("/home/me/.config/rupa"),
            Some(Path::new("/home/me/.local/state/rupa")),
            Path::new("/home/me/.local/share/rupa"),
        );
        assert_eq!(state, Path::new("/home/me/.local/state/rupa"));
    }

    #[test]
    fn state_falls_back_to_local_data_when_it_is_separate_from_config() {
        // Windows: no state dir; Local data differs from Roaming config.
        let state = choose_state_dir(
            Path::new(r"C:\Users\me\AppData\Roaming\rupa\config"),
            None,
            Path::new(r"C:\Users\me\AppData\Local\rupa\data"),
        );
        assert_eq!(state, Path::new(r"C:\Users\me\AppData\Local\rupa\data"));
    }

    #[test]
    fn no_home_and_no_override_is_a_clear_error() {
        let err = Paths::resolve(env(&[]), None).expect_err("nothing to resolve from");
        assert!(matches!(err, PathsError::NoHome));
    }
}
