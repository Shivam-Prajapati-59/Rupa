# Run from PowerShell or cmd on Windows, not Git Bash (see docs/dev-setup.md).
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
    @just --list

# The gate every slice must pass before merge: fmt, clippy -D warnings, tests.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

test:
    cargo test --workspace

fmt:
    cargo fmt --all

run *ARGS:
    cargo run -p rupa -- {{ARGS}}
