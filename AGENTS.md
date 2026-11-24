# AGENTS.md

Instructions for coding agents (and humans) working on **Rupa**. `CLAUDE.md` only points here. Read this file first, then the spec section for your slice.

## 1. What this is

Rupa is a terminal agent runtime in Rust: a background **daemon owns real PTYs** running AI coding agents (Claude Code, Codex, OpenCode, any CLI); a **TUI/CLI client attaches** over a local socket; an **agent-awareness layer** says which agent is working, blocked on you (and why), or done. One binary, `rupa`, selects client, daemon or CLI by argv. Windows 11 is the primary dev target; Linux and macOS are CI-gated from day one. The reference product is herdr (`reference/herdr/src/`, local and read-only).

**Non-goals for V1** (full list: spec §1.2): plugin system, SSH/multi-machine, GUI/web/mobile, kanban/task planner, sandboxing, ACP/MCP modes, Kitty graphics/keyboard protocols, cost/token tracking, auto-update. If a task needs one of these, stop and ask. New scope goes into spec §7.2 first (§12.3).

## 2. The spec is the contract

- **`TECH_SPEC.md` (repo root) is authoritative.** The old plan and research live in the gitignored `reference/herdr/` (may be absent): evidence, not instruction (r4's crate versions are stale).
- **Slice IDs** look like `P2-S3` = phase 2, slice 3 (§8). A slice is the smallest mergeable increment: it compiles, has a test, and passes its **Done when**. Sizes: XS 0.25 d, S 0.5 d, M 1 d, L 2 d; anything bigger than L must be split first.
- **Gates G0..G6** (§9.4) block the next phase. G0 CI green on 3 OSes · **G1** emulator ADR-0003 with real captures · **G2** detach/reattach ×20, scripted in CI · **G3** SC2 status metrics on the golden set · GB hooks beat the screen guess · G4 50 worktree cycles clean · G5 scripted fleet · **G6** SC1–SC8 + dogfood week. Never skip, soften or "pass with a TODO". Success criteria in §1.3 are never loosened without an ADR.
- **Evidence tags:** **[V]** verified in source/docs or on this machine · **[R]** reported by research, unverified · **[A]** assumption or initial tuning value that a named slice must validate. Do not promote a tag without evidence; when you verify an **[A]**, update the spec (§12.4 lists them).
- **Spec bugs are slice-sized work.** If reality contradicts the spec, fix the code *and* the spec in the same change. If a §0.4 decision (D1..D14) changes, write an ADR first (§12.3).
- The spec lists dependencies in §6.3. **Do not add one that isn't there** without a spec change; add with `cargo add` and never copy a version from any document.

## 3. Workflow (one slice at a time)

1. Pick the next unticked slice in `docs/progress.md` whose dependencies are done. Work **one slice**, never two. Branch `p<phase>-s<slice>-<slug>` (e.g. `p2-s3-daemon-spawn`).
2. Read the slice and the §5.x it links. Do only its listed tasks.
3. **TDD:** write the failing test named in **Done when** first (use the `tdd` skill). Then implement tasks in order.
4. Tick each task checkbox in `TECH_SPEC.md` as it goes green; **commit after each green task** (small commits).
5. Run `just check` (fmt, clippy `-D warnings`, tests) before every push. CI must be green on Windows, Linux and macOS before merge.
6. Update the spec/ADRs if reality disagreed. Tick the slice in `docs/progress.md`.
7. **Phase end:** after the gate passes, ask the user to run `/improve-codebase-architecture` (see §9) and run `ponytail-debt`.

If a task takes longer than ~90 minutes, split it and write the split down. If you are blocked on a decision that is the user's (§12.1), ask; don't guess.

## 4. Architecture invariants (spec §4.6; violating one is a bug)

1. **State is separate from runtime.** `AppState`/`PaneState` are plain data constructible without a PTY (`AppState::test_new()`); `PaneRuntime` owns PTYs, threads and channels.
2. **Render is pure.** `compute_view()` does geometry and mutation; `render(&ViewModel)` only draws and returns a hit map.
3. **Platform isolation.** All OS-specific code lives in `rupa-platform`. Elsewhere `#[cfg(target_os)]` appears only on imports, struct fields and match arms. `unsafe` is denied workspace-wide; only `rupa-platform` may `#![allow(unsafe_code)]`.
4. **Detection is decoupled.** The status engine reads a *snapshot of the live bottom rows*, never the parser, a viewport or scrolled state.
5. **One actor owns mutable state.** Everyone else sends messages.
6. **No blocking on the actor or in render.** Slow work (git, process probes, clipboard) goes to workers with an operation id; a stale completion is ignored.
7. **Every queue is bounded** and has a documented overflow behaviour.
8. **Every side effect is verified before it is announced** (no "Copied" without a confirmed write; no "installed" without re-reading the file).
9. **Wire compatibility is a test:** hand-written fixture bytes for hello, welcome, errors.
10. **Unknown input is ignored, not fatal:** unknown JSON fields, manifest keys, escape sequences.

**Crate layering** (arrows = "may depend on"; spec §4.5): `rupa` → `server`, `tui`, `client`, `platform` · `server` → `model`, `detect`, `pty`, `screen`, `platform`, `proto` · `tui` → `client`, `model`, `screen` · `pty` → `platform` · `client` → `proto`, `platform` · `detect` → `screen`, `proto` · `model` → `proto`. **Pure crates (no I/O): `proto`, `screen`, `detect`, `model`.** Don't add an edge that isn't in the diagram.

## 5. Rust conventions

- **Edition 2024, stable toolchain** (`rust-toolchain.toml`), MSRV 1.94. Format with the repo `rustfmt.toml`; clippy is `-D warnings`.
- **Errors:** `thiserror` enums in library crates; `anyhow` only in the `rupa` binary, `tools/`, `tests/`. Wire errors use stable codes from `rupa-proto`. **No `unwrap()` outside tests** (the `clippy::unwrap_used` lint enforces it). `expect("invariant: …")` only for a provable invariant, with the reason in the message. No `panic!`/`todo!`/`dbg!` in committed code; no indexing that can panic on external input.
- **Logging:** `tracing` with a file appender in `<state_dir>/logs/`, level from `RUPA_LOG`. **Never write to stdout/stderr in TUI or daemon mode.** Logs hold ids, states and rule names, **never pane contents** (they may contain secrets).
- **Async:** blocking PTY I/O runs on std threads (reader, writer, exit watcher per pane); everything else is tokio. Never `block_on` inside the runtime or hold a `std::sync::Mutex` guard across `.await`; hold the screen mutex only to parse a chunk or copy rows out. Use `spawn_blocking` for blocking work. Channels are bounded; state the overflow policy next to each one. Make cancellation explicit (shutdown tokens), and never leave detached tasks that outlive their owner.
- **Style:** small modules with narrow interfaces; prefer deep modules over many shallow ones (use the `codebase-design` vocabulary). No speculative abstraction, options or config nobody asked for. Comments say *why*. A known shortcut gets `// shortcut: <limit>, <when to upgrade>`.
- **Paths and IDs:** ids are monotonic integers, never reused in a daemon boot. Runtime paths come from `rupa-platform::paths` (honour `RUPA_CONFIG_DIR`, `RUPA_STATE_DIR`, `RUPA_RUNTIME_DIR` so tests are isolated). Socket path must stay under 100 bytes.

## 6. Windows constraints (read before your first build)

- **Build and test from PowerShell or cmd, not Git Bash.** Git Bash puts GNU `link.exe` ahead of MSVC and breaks build scripts. **MSVC Build Tools** (C++ workload + Windows 11 SDK) are required to link; `cargo check`/`clippy`/`fmt` work without them. Setup: `docs/dev-setup.md`. Do not install system software yourself; report it.
- **ConPTY quirks** (spec §5.1.1): send Ctrl-C as `0x03` through the PTY, never `GenerateConsoleCtrlEvent` on the shared console; reads can end mid-UTF-8 codepoint, so parse incrementally; resolve programs with `is_file()` and handle `.cmd`/`.exe` shims; keep the original quoting of raw `cmd.exe /c` command tails; send multi-line paste as **one bracketed paste** (otherwise Claude Code submits at the first newline); don't leak daemon handles into children; never resize PTYs to a fallback size when no client is attached.
- The **daemon**, not a client, goes in the Job Object (`KILL_ON_JOB_CLOSE`); doing it in a client would kill agents on detach. Detached daemon spawn and the WMI fallback are P2-S3.
- Default shell resolution on Windows: `pwsh` → `powershell` → `cmd` (only Windows PowerShell 5.1 exists on the author's machine). Line endings are LF everywhere (`.gitattributes`).

## 7. Testing

- `just check` is the merge gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. No `just`? Run those three lines.
- Layers (spec §10): **L1** pure unit/property tests (`proptest`; no PTYs, milliseconds) · **L2** in-process `TestServer` with in-memory client writers (assert on decoded messages, not bytes) · **L3** real binary under a PTY with `tests/fake-agent` (isolated `RUPA_RUNTIME_DIR`, graceful daemon stop, watchdog for strays) · **L4** golden-capture detection (labelled intervals; each manifest rule needs a positive and a negative fixture) · **L5** manual release checklist.
- Every behaviour change has a test that fails without it. New parsers, state machines and codecs get property or fixture tests. Never weaken or delete a test to get green; never mark a test `ignore` without a linked reason and slice.
- Detection changes must keep the SC2 evaluation passing (recall<sub>blocked</sub> ≥ 0.98, FIR ≤ 0.01, p95 detect ≤ 1.0 s).

## 8. Commits and branches

- **Conventional Commits with the slice ID:** `feat(pty): spawn and resize via portable-pty (P1-S2)`. Types: `feat fix test refactor docs build ci chore`. Scope = crate name without `rupa-` (`pty`, `server`, `detect`…) or `repo`/`docs`/`ci`.
- Subject ≤ 72 chars, imperative. Body says *why*. Mention gate or ADR numbers when relevant. One logical change per commit; commits after green tasks, not at the end.
- Branch `p<phase>-s<slice>-<slug>`. No force-push to `main`. Don't commit unless asked; when asked, never skip hooks.
- A PR carries: slice ID, the **Done when** evidence, spec/ADR edits, and for gate slices the evidence the gate names.

## 9. Installed skills (`.claude/skills/`, project-scoped, committed)

| Skill | Source | Use it when |
|---|---|---|
| `tdd` | mattpocock/skills | Every slice: red-green-refactor, tests through the public interface. |
| `diagnosing-bugs` | mattpocock/skills | Anything failing, flaky or slow, especially ConPTY/detach/timing bugs: build a reproduction loop before guessing. |
| `codebase-design` | mattpocock/skills | Designing or reviewing a module's interface, deciding where a seam goes. Shared vocabulary: module, interface, depth, seam, adapter. |
| `improve-codebase-architecture` | mattpocock/skills | **At the end of each phase, after its gate passes**, to find deepening refactors. User-invoked only (it disables model invocation), so ask the user to run `/improve-codebase-architecture`. It writes its HTML report to the OS temp dir, reads `docs/adr/` so it won't re-litigate them, and uses `GLOSSARY.md`. |
| `grilling`, `grill-me` | mattpocock/skills | Stress-testing a plan or design before building it; `grill-me` is user-invoked. Use on any §12.1 open decision or new scope. |
| `domain-modeling` | mattpocock/skills | Keeping `GLOSSARY.md` and ADRs current (format: `.claude/skills/domain-modeling/ADR-FORMAT.md`). Seed `GLOSSARY.md` from spec Appendix D, created lazily. |
| `rust-best-practices` | apollographql/skills | Writing or reviewing Rust: ownership, error handling, clippy, testing, performance. Where it disagrees with this file or the spec, **this file and the spec win**. |
| `rust-async-patterns` | wshobson/agents | Tokio code in `rupa-server`/`rupa-client`: tasks, channels, cancellation, async traits. |
| `ponytail` | dietrichgebert/ponytail | A "smallest complete change" reminder when a slice starts growing. It never overrides the slice's listed tasks, **Done when** tests, or the invariants in §4. |
| `ponytail-review`, `ponytail-audit`, `ponytail-debt` | dietrichgebert/ponytail | Review a diff before merge; audit the repo at phase end; list `shortcut:` debt comments (`ponytail-debt`). |

Skills are third-party instructions that run with full agent permissions. Review the diff in `.claude/skills/` and `skills-lock.json` before committing any update (`npx skills update --project`, with `DISABLE_TELEMETRY=1`). Restore with `npx skills experimental_install`.

## 10. Documentation rules

- `TECH_SPEC.md` stays in sync with the code. Tick task boxes there and slice boxes in `docs/progress.md`. Update **§12.4** when an **[A]** is validated, and **Appendix G** when a decision changes.
- Record decisions in `docs/adr/NNNN-slug.md` (copy `0000-template.md`) when a §0.4 decision changes, when a gate decides something (ADR-0003 emulator, ADR-0004 resize debounce, ADR-0005 status reconciler), or when a hard-to-reverse, surprising trade-off is made. ADR-0001 is "own daemon" and ADR-0002 is "name and licence": the spec cites those numbers.
- Update `docs/README.md` when adding docs; keep relative links working. `README.md` must never claim a feature that doesn't run.
- Adapting anything from herdr: mark it in source with the herdr path and version, and keep it covered in `NOTICE`.

## 11. Do NOT

- **Don't touch `reference/`.** It is gitignored, read-only study material about herdr. Study it; don't copy code from it without attribution (above).
- **No telemetry, analytics, crash upload or network calls** in Rupa. The only permitted network call is the user-run `rupa update --check` (D11, SC8).
- **Never commit secrets or captures.** `captures/` is ignored because recordings can contain tokens; only redacted files go to `tests/golden/`. No `.env`, keys or tokens in the repo, logs or fixtures. Redact before sharing any capture.
- Don't commit build output (`target/`), crash dumps or logs. Do commit `Cargo.lock` and `skills-lock.json`.
- Don't use `tmux`/`screen` or shell/Python hook scripts as a dependency: hooks are `rupa hook <agent>` (D7); the daemon owns the PTYs (ADR-0001).
- Don't work outside the current slice, add features nobody asked for, or silently change the spec, the protocol or a success criterion.
- Don't `git push`, force-push, rewrite history or delete branches unless explicitly asked.
