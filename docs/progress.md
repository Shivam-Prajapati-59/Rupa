# Progress

Tick a slice here when it is merged and its **Done when** passes (TECH_SPEC §8.2 step 6). Slice tasks themselves are ticked in `TECH_SPEC.md`. Phase gates (G0 to G6) are defined in §9.4; a gate is not passed until the evidence the spec names is committed.

Generated from the slice headers in `TECH_SPEC.md` §8 on 2026-11-15. If a slice is added, split or renamed in the spec, update it here in the same change.

**Current status:** Phase 0 code is done (P0-S1, S2, S4, S5, S6); only P0-S3 (CI) is open. `just check` passes on Windows and in WSL (Linux). GitHub Actions is not used for now (the account is on the free plan and its billing is locked), so `.github/workflows/ci.yml` was removed. **Gate G0 (CI green on all three OSes) is therefore on hold**: until CI returns, every slice is checked locally with `just check` on Windows and the same three commands in WSL, and macOS is untested. Bring CI back (restore the workflow from git history) before any release.


## Phase 0 — Foundations

- [x] **P0-S1** Toolchain and dev environment (S)
- [x] **P0-S2** Workspace skeleton (S)
- [ ] **P0-S3** CI matrix (S). On hold: GitHub Actions billing is locked. Blocks Gate G0.
- [x] **P0-S4** Paths, logging, panic hook (S). Green on Windows and Linux (WSL); macOS untested until CI returns.
- [x] **P0-S5** Config loader (S). Green on Windows and Linux (WSL).
- [x] **P0-S6** Repo hygiene (XS)

**Phase 0 architecture review (2026-11-22, `/improve-codebase-architecture`):** A (deepen `Paths` into the one "where" module: `config_file()`, `logs_dir()`, `socket_path()`, one "empty = unset" rule) **done**. B (a `Startup` module out of `main.rs`) **do at the start of Phase 2**, when daemon and attach roles arrive. C (config key lists; guarded by a drift test) and D (config types in `rupa-platform`) **deferred** until real friction.

## Phase 1 — Terminal core and the fidelity gate

- [ ] **P1-S1** Capture and replay tool (M)
- [ ] **P1-S2** `rupa-pty` crate (M)
- [ ] **P1-S3** `rupa-screen` crate (L)
- [ ] **P1-S4** Fidelity review and emulator decision (L) **GATE G1**
- [ ] **P1-S5** Process-tree cleanup (S)
- [ ] **P1-S6** Single-pane prototype TUI (M)
- [ ] **P1-S7** Key encoder (M)
- [ ] **P1-S8** Budget measurements (S)

## Phase 2 — Daemon, protocol, attach

- [ ] **P2-S1** Protocol crate (M)
- [ ] **P2-S2** Local IPC transport (M)
- [ ] **P2-S3** Daemon lifecycle (L)
- [ ] **P2-S4** Session actor and event hub (M)
- [ ] **P2-S5** Pane runtime (M)
- [ ] **P2-S6** Attach and render pipeline (L)
- [ ] **P2-S7** Client input path (M)
- [ ] **P2-S8** Geometry controller (M)
- [ ] **P2-S9** Walking skeleton (M) **GATE G2**
- [ ] **P2-S10** Test harness (S)
- [ ] **P2-S11** Client frame consumer (M)
- [ ] **P2-S12** Fake agent (M)

## Phase 3 — Multiplexer UI

- [ ] **P3-S1** Layout tree (M)
- [ ] **P3-S2** Workspace, tab, pane model (M)
- [ ] **P3-S3** Chrome: sidebar, tab bar, status line (L)
- [ ] **P3-S4** Prefix state machine and keybindings (M)
- [ ] **P3-S5** Mouse (M)
- [ ] **P3-S6** Scrollback, copy mode, search (L)
- [ ] **P3-S7** Clipboard (M)
- [ ] **P3-S8** Profiles and launch (M)
- [ ] **P3-S9** Navigators (S)
- [ ] **P3-S10** Theme (S)
- [ ] **P3-S11** Structure commands end to end (L)

## Phase 4 — Status engine (the product)

- [ ] **P4-S1** Detection types and rollup (S)
- [ ] **P4-S2** Manifest engine (L)
- [ ] **P4-S3** Bundled manifests and golden regression (L)
- [ ] **P4-S4** Process identification (M)
- [ ] **P4-S5** OSC signal ingestion (M)
- [ ] **P4-S6** Reconciler (M)
- [ ] **P4-S7** Generic prompt patterns (S)
- [ ] **P4-S8** Inbox and jump (M)
- [ ] **P4-S9** Notifications (M)
- [ ] **P4-S10** Calibration harness (M) **GATE G3**

## Phase 5 — Agent integrations

- [ ] **P5-S1** Report API and CLI (M)
- [ ] **P5-S2** `rupa hook <agent>` mappers (M)
- [ ] **P5-S3** Claude installer (L)
- [ ] **P5-S4** Codex installer (M)
- [ ] **P5-S5** OpenCode plugin (M)
- [ ] **P5-S6** Resume references (M)
- [ ] **P5-S7** `rupa doctor` (M)

## Phase 6 — Worktrees and tasks

- [ ] **P6-S1** Git runner (M)
- [ ] **P6-S2** Worktree operations (L)
- [ ] **P6-S3** Task commands and linkage (L)
- [ ] **P6-S4** Per-task ports and env (S)
- [ ] **P6-S5** Git status chips (M)
- [ ] **P6-S6** Diff popup (S)

## Phase 7 — Control API and CLI

- [ ] **P7-S1** Method registry and schema (M)
- [ ] **P7-S2** Structure CLI (L)
- [ ] **P7-S3** Pane I/O primitives (L)
- [ ] **P7-S4** Events (M)
- [ ] **P7-S5** Agent primitives (M)
- [ ] **P7-S6** CLI polish (S)

## Phase 8 — Persistence, hardening, release

- [ ] **P8-S1** Snapshot schema and atomic writes (M)
- [ ] **P8-S2** Restore flow (L)
- [ ] **P8-S3** Resource governor (M)
- [ ] **P8-S4** Crash isolation (M)
- [ ] **P8-S5** Security pass (S)
- [ ] **P8-S6** Packaging and release (L)
- [ ] **P8-S7** Documentation (M)
- [ ] **P8-S8** Dogfood week (L)
- [ ] **P8-S9** Dogfood fixes and V1.0 gate (L) **GATE G6**
