# Developer setup

Written for the primary target, Windows 11 (decision D12). Linux and macOS need only a Rust toolchain and a C compiler.

## Windows

1. **Install the C++ build tools** (Visual Studio 2022 Build Tools, or Visual Studio 2026 Community as the author did) with the *Desktop development with C++* workload (MSVC v143 and the Windows 11 SDK). Without it nothing links: the machine this project started on had no `link.exe`, and the only other C toolchain was 32-bit MinGW, which does not work for the default Rust target. **[V]** `cargo check` and `clippy` work without it; `cargo build`, `cargo test` and `cargo run` do not.
2. **Build from PowerShell or cmd, not Git Bash.** Git Bash puts GNU `link` (`C:\Program Files\Git\usr\bin\link.exe`) ahead of MSVC's `link.exe`, which breaks every build script. **[V]**
3. Rust: `rustup` with the `stable` toolchain. `rust-toolchain.toml` pins the channel and the `rustfmt` and `clippy` components, so rustup installs them on first use.
4. Optional but expected by the workflow: [`just`](https://github.com/casey/just) (`winget install Casey.Just`) and `cargo install cargo-nextest`.

## Commands

```powershell
just check   # fmt --check, clippy -D warnings, test: the gate before every merge
just test
just fmt
just run -- --help
```

Without `just`, run the three lines of the `check` recipe in `justfile` by hand.

## Agent skills

Project-scoped skills live in `.claude/skills/` and are committed; `skills-lock.json` records their sources and hashes. See `AGENTS.md` for what each one is for. To restore or update them:

```powershell
$env:DISABLE_TELEMETRY = "1"; $env:DO_NOT_TRACK = "1"   # the skills CLI sends install telemetry otherwise
npx skills experimental_install      # restore exactly what skills-lock.json records
npx skills update --project          # fetch newer versions; review the diff before committing
```

Skills run with full agent permissions: read what changed in `.claude/skills/` before committing an update.

## Reference clone of herdr

The spec cites herdr source paths (for example `src/detect/manifests/*.toml`, `vendor/patches/portable-pty/`). Keep a read-only clone at `reference/herdr/src/` in the repo root, at commit `ee5bfb5` (herdr 0.9.5). It is gitignored. Never edit it, never copy from it without the attribution rule in `NOTICE`.

```powershell
git clone https://github.com/herdrdev/herdr reference/herdr/src
git -C reference/herdr/src checkout ee5bfb5
```

## Scratch locations

- `target/`: build output (ignored).
- `captures/`: raw terminal recordings (ignored, may contain secrets). Only redacted files may go to `tests/golden/`.
- `spikes/pty-capture/`: the never-built capture spike, excluded from the workspace. P1-S1 moves its sources over the `tools/pty-capture` stub (do not nest the directory).
