# Rupa

A terminal agent runtime. A background daemon owns real terminals running AI coding agents (Claude Code, Codex, OpenCode, or any CLI); a TUI attaches to it and detaches without disturbing them. On top sits the point of the project: **agent awareness**. Rupa tells you which agent is working, which is blocked waiting on you and why, and which is done.

> **Status: pre-alpha, specification complete, no usable code yet.** The repository is a Cargo workspace skeleton. Nothing runs yet. The plan is in [`TECH_SPEC.md`](TECH_SPEC.md); progress is in [`docs/progress.md`](docs/progress.md).

## What it will do (V1)

- Run any terminal agent in real PTY panes that survive the UI closing.
- Report agent state (`working`, `blocked`, `done`, `idle`) with a visible reason and confidence, and never say "idle" while work is going on.
- One inbox of agents that need you, with a one-key jump.
- One git worktree per task, created and cleaned up by one command.
- A local JSON API and CLI so scripts, and agents, can drive it.
- Cheap when idle, and private: no telemetry, no account, no network calls unless you run `rupa update --check`.

Targets Windows, Linux and macOS, with Windows 11 as the primary development platform.

## What this is not

Rupa is not a better multiplexer. It is a **supervision layer** that happens to own its terminals. tmux can keep your agents alive; it cannot tell you which one is waiting for you and why, group them by git worktree, or let a script wait until an agent is blocked. If you only need persistence, use tmux.

Also out of scope for V1: plugins, SSH or multi-machine, a GUI, kanban planning, sandboxing, cost tracking. See [`TECH_SPEC.md` §1.2](TECH_SPEC.md).

## Repository layout

| Path | What |
|---|---|
| `crates/rupa` | the single binary: client, daemon and CLI selected by argv |
| `crates/rupa-*` | `proto`, `platform`, `pty`, `screen`, `detect`, `model`, `server`, `client`, `tui` (layering in [TECH_SPEC §4.5](TECH_SPEC.md)) |
| `tools/pty-capture`, `tests/fake-agent` | capture/replay tool and scripted fake agent (stubs until P1-S1 and P2-S12) |
| `docs/` | spec index, ADRs, progress: start at [`docs/README.md`](docs/README.md) |
| `AGENTS.md` | rules for coding agents and contributors |

## Building

Windows needs the Visual Studio Build Tools and a PowerShell or cmd shell (not Git Bash). Full steps: [`docs/dev-setup.md`](docs/dev-setup.md).

```powershell
just check    # fmt --check, clippy -D warnings, test
```

## Licence

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Rupa studies the design of [herdr](https://github.com/herdrdev/herdr) (Apache-2.0); see [`NOTICE`](NOTICE).
