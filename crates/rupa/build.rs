//! Runs before `rupa` is compiled: stamps the git commit and the target triple into the
//! binary as `RUPA_COMMIT` and `RUPA_TARGET`, for `rupa --version` and crash reports.

use std::process::Command;

fn main() {
    // Cargo sets TARGET for build scripts, e.g. "x86_64-pc-windows-msvc".
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=RUPA_TARGET={target}");

    // No git (a source tarball, a CI checkout without history): say so, don't fail the build.
    let commit = git(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=RUPA_COMMIT={commit}");

    // Re-run when the checked-out commit changes, not on every edit.
    println!("cargo:rerun-if-changed=build.rs");
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        println!("cargo:rerun-if-changed={git_dir}/HEAD");
        println!("cargo:rerun-if-changed={git_dir}/logs/HEAD");
    }
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    Some(text.trim().to_string()).filter(|s| !s.is_empty())
}
