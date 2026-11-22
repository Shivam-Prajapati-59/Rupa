//! The runtime folder: where the daemon's socket lives while Rupa is running.
//!
//! Unlike config and state it is temporary and private (only the current user may use
//! it). Windows has no socket file: it uses a named pipe, so there the "runtime dir" only
//! holds a marker file.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// `sun_path` is 104 bytes on macOS and 108 on Linux. Staying under 100 leaves a margin.
pub const MAX_SOCKET_PATH_BYTES: usize = 100;

/// Which OS rules to apply. A value, not a `cfg!`, so tests can exercise every OS from one machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Host {
    Linux { uid: u32 },
    MacOs { uid: u32 },
    Windows,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("cannot find %LOCALAPPDATA%; set RUPA_RUNTIME_DIR")]
    NoLocalAppData,
    #[error(
        "socket path is {len} bytes but the limit is {max}: {}. Set RUPA_RUNTIME_DIR to a shorter folder",
        .path.display()
    )]
    SocketPathTooLong {
        len: usize,
        max: usize,
        path: PathBuf,
    },
    #[error("runtime folder {} is not safe to use: {reason}", .path.display())]
    Unsafe { path: PathBuf, reason: &'static str },
    #[error("cannot create runtime folder {}", .path.display())]
    Create {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl Host {
    #[cfg(windows)]
    pub fn current() -> Self {
        Host::Windows
    }

    #[cfg(target_os = "macos")]
    pub fn current() -> Self {
        Host::MacOs {
            uid: nix::unistd::Uid::current().as_raw(),
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn current() -> Self {
        Host::Linux {
            uid: nix::unistd::Uid::current().as_raw(),
        }
    }
}

/// Where the runtime folder is for this process, from the real environment.
pub fn runtime_dir_from_env() -> Result<PathBuf, RuntimeError> {
    runtime_dir(Host::current(), |key| std::env::var_os(key))
}

/// Pure: only decides the path (no folder is created). `env` is injected so tests don't
/// touch the real process environment.
pub fn runtime_dir(
    host: Host,
    env: impl Fn(&str) -> Option<OsString>,
) -> Result<PathBuf, RuntimeError> {
    let get = |key: &str| env(key).filter(|v| !v.is_empty());

    if let Some(dir) = get("RUPA_RUNTIME_DIR") {
        return Ok(PathBuf::from(dir));
    }
    match host {
        Host::Linux { uid } => {
            // Checked as text, not `is_absolute()`, so the answer is the same on any host OS.
            let xdg = get("XDG_RUNTIME_DIR").filter(|v| v.to_string_lossy().starts_with('/'));
            Ok(match xdg {
                Some(xdg) => Path::new(&xdg).join("rupa"),
                None => PathBuf::from(format!("/tmp/rupa-{uid}")),
            })
        }
        Host::MacOs { uid } => {
            // macOS gives every user a private $TMPDIR already; it is just long (see socket_path).
            let base = get("TMPDIR").map_or_else(|| PathBuf::from("/tmp"), PathBuf::from);
            Ok(base.join(format!("rupa-{uid}")))
        }
        Host::Windows => get("LOCALAPPDATA")
            .map(|base| Path::new(&base).join("rupa"))
            .ok_or(RuntimeError::NoLocalAppData),
    }
}

/// `<runtime_dir>/sessions/<session>/rupa.sock`, refusing paths the OS would reject.
pub fn socket_path(runtime_dir: &Path, session: &str) -> Result<PathBuf, RuntimeError> {
    let path = runtime_dir.join("sessions").join(session).join("rupa.sock");
    let len = path.as_os_str().len();
    if len >= MAX_SOCKET_PATH_BYTES {
        return Err(RuntimeError::SocketPathTooLong {
            len,
            max: MAX_SOCKET_PATH_BYTES,
            path,
        });
    }
    Ok(path)
}

/// Create `path` with mode 0700 and check it really is ours and private.
#[cfg(unix)]
pub fn ensure_private_dir(path: &Path) -> Result<(), RuntimeError> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};

    let create_err = |source| RuntimeError::Create {
        path: path.to_path_buf(),
        source,
    };
    let unsafe_err = |reason| RuntimeError::Unsafe {
        path: path.to_path_buf(),
        reason,
    };

    // An existing folder is fine: create is a no-op for it, and we verify it below.
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(create_err)?;

    // `symlink_metadata` does not follow links, so a link planted by someone else is seen as
    // a link and refused, instead of being quietly followed into their folder.
    let meta = std::fs::symlink_metadata(path).map_err(create_err)?;
    if !meta.file_type().is_dir() {
        return Err(unsafe_err("not a real folder (a symlink or a file)"));
    }
    if meta.uid() != nix::unistd::Uid::current().as_raw() {
        return Err(unsafe_err("owned by another user"));
    }
    if meta.permissions().mode() & 0o077 != 0 {
        return Err(unsafe_err("other users can enter it (needs mode 0700)"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let map: HashMap<String, OsString> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), OsString::from(v)))
            .collect();
        move |key| map.get(key).cloned()
    }

    const LINUX: Host = Host::Linux { uid: 1000 };
    const MAC: Host = Host::MacOs { uid: 501 };

    #[test]
    fn override_wins_on_every_os() {
        for host in [LINUX, MAC, Host::Windows] {
            let dir = runtime_dir(host, env(&[("RUPA_RUNTIME_DIR", "/x/rt")])).expect("override");
            assert_eq!(dir, PathBuf::from("/x/rt"), "{host:?}");
        }
    }

    #[test]
    fn empty_override_counts_as_unset() {
        let dir = runtime_dir(LINUX, env(&[("RUPA_RUNTIME_DIR", "")])).expect("fallback");
        assert_eq!(dir, PathBuf::from("/tmp/rupa-1000"));
    }

    #[test]
    fn linux_prefers_xdg_runtime_dir() {
        let dir = runtime_dir(LINUX, env(&[("XDG_RUNTIME_DIR", "/run/user/1000")])).expect("xdg");
        assert_eq!(dir, Path::new("/run/user/1000").join("rupa"));
    }

    #[test]
    fn linux_without_xdg_uses_tmp_with_the_uid() {
        let dir = runtime_dir(LINUX, env(&[])).expect("fallback");
        assert_eq!(dir, PathBuf::from("/tmp/rupa-1000"));
    }

    #[test]
    fn linux_ignores_a_relative_xdg_runtime_dir() {
        // The XDG spec requires an absolute path; a relative one would put the socket
        // wherever the current folder happens to be.
        let dir = runtime_dir(LINUX, env(&[("XDG_RUNTIME_DIR", "run/user")])).expect("fallback");
        assert_eq!(dir, PathBuf::from("/tmp/rupa-1000"));
    }

    #[test]
    fn macos_uses_tmpdir_with_the_uid() {
        let dir = runtime_dir(MAC, env(&[("TMPDIR", "/var/folders/ab/T/")])).expect("tmpdir");
        assert_eq!(dir, Path::new("/var/folders/ab/T/").join("rupa-501"));
    }

    #[test]
    fn macos_without_tmpdir_falls_back_to_tmp() {
        let dir = runtime_dir(MAC, env(&[])).expect("fallback");
        assert_eq!(dir, PathBuf::from("/tmp/rupa-501"));
    }

    #[test]
    fn windows_uses_a_folder_under_local_app_data_for_the_marker_file() {
        let dir = runtime_dir(
            Host::Windows,
            env(&[("LOCALAPPDATA", r"C:\Users\me\AppData\Local")]),
        )
        .expect("localappdata");
        assert_eq!(dir, Path::new(r"C:\Users\me\AppData\Local").join("rupa"));
    }

    #[test]
    fn windows_without_local_app_data_is_a_clear_error() {
        let err = runtime_dir(Host::Windows, env(&[])).expect_err("nothing to use");
        assert!(matches!(err, RuntimeError::NoLocalAppData));
    }

    /// A runtime dir of exactly `n` bytes; `socket_path` adds 24 more ("/sessions/main/rupa.sock").
    fn dir_of_len(n: usize) -> PathBuf {
        PathBuf::from(format!("/{}", "a".repeat(n - 1)))
    }

    #[test]
    fn socket_path_has_the_session_layout() {
        let p = socket_path(Path::new("/run/rupa"), "main").expect("short path");
        assert_eq!(p, Path::new("/run/rupa/sessions/main/rupa.sock"));
    }

    #[test]
    fn socket_path_of_99_bytes_is_accepted() {
        let p = socket_path(&dir_of_len(75), "main").expect("99 bytes is under the limit");
        assert_eq!(p.as_os_str().len(), 99);
    }

    #[test]
    fn socket_path_of_100_bytes_is_refused_with_a_fix() {
        let err = socket_path(&dir_of_len(76), "main").expect_err("100 bytes is too long");
        assert!(matches!(
            err,
            RuntimeError::SocketPathTooLong {
                len: 100,
                max: 100,
                ..
            }
        ));
        assert!(err.to_string().contains("RUPA_RUNTIME_DIR"), "{err}");
    }

    #[cfg(unix)]
    mod unix {
        use super::super::*;
        use std::fs::{self, Permissions};
        use std::os::unix::fs::{PermissionsExt, symlink};

        fn mode(p: &Path) -> u32 {
            fs::metadata(p).expect("exists").permissions().mode() & 0o777
        }

        #[test]
        fn creates_the_folder_private() {
            let tmp = tempfile::tempdir().expect("tempdir");
            let dir = tmp.path().join("rupa-test");
            ensure_private_dir(&dir).expect("create");
            assert_eq!(mode(&dir), 0o700);
        }

        #[test]
        fn accepts_an_existing_private_folder() {
            let tmp = tempfile::tempdir().expect("tempdir");
            let dir = tmp.path().join("rupa-test");
            ensure_private_dir(&dir).expect("first");
            ensure_private_dir(&dir).expect("second call is a no-op");
        }

        #[test]
        fn refuses_a_folder_other_users_can_enter() {
            let tmp = tempfile::tempdir().expect("tempdir");
            let dir = tmp.path().join("rupa-test");
            fs::create_dir(&dir).expect("create");
            fs::set_permissions(&dir, Permissions::from_mode(0o755)).expect("chmod");
            let err = ensure_private_dir(&dir).expect_err("0755 is not private");
            assert!(matches!(err, RuntimeError::Unsafe { .. }), "{err}");
        }

        #[test]
        fn refuses_a_symlink_planted_in_our_place() {
            // Another user could pre-create /tmp/rupa-<uid> as a link to a folder they control.
            let tmp = tempfile::tempdir().expect("tempdir");
            let target = tmp.path().join("elsewhere");
            fs::create_dir(&target).expect("target");
            let link = tmp.path().join("rupa-test");
            symlink(&target, &link).expect("symlink");
            let err = ensure_private_dir(&link).expect_err("a symlink is not our folder");
            assert!(matches!(err, RuntimeError::Unsafe { .. }), "{err}");
        }
    }
}
