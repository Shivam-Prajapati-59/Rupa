# ADR-0001: Rupa runs its own daemon and owns the PTYs; no tmux dependency

- **Status:** accepted
- **Date:** 2026-11-15
- **Spec:** D1 (`TECH_SPEC.md` §0.4), owner slice P0-S6, validated by Gate G2

## Context

Rupa has to keep agent processes alive when the UI closes. The cheapest way would be to run every pane inside tmux. tmux does not exist on Windows and is not installed on the author's primary machine **[V]**. herdr and zellij both own their PTYs rather than delegating to a multiplexer **[V]**. Owning the terminals is also what lets Rupa read each pane's screen to infer agent state (D6) and keep server-authoritative screen state for every client (D4).

## Options considered

- **Drive tmux (including tmux under WSL).** Rejected: no native Windows story, and screen contents and size negotiation sit behind a second process we cannot control.
- **Own daemon with ConPTY / `openpty` via `portable-pty`.** Chosen.

## Decision

A background `rupa` daemon owns every PTY and its emulator. Clients, including the TUI and the CLI, are thin and attach over a local socket.

## Consequences

- Detach and reattach, daemon start-up, detached spawn on Windows, and child cleanup are ours to build and to prove (Phase 2, Gate G2: detach/reattach 20 times, scripted in CI).
- ConPTY quirks are ours to handle (TECH_SPEC §5.1.1).
- If all you need is persistence, tmux is simpler. The README says so (TECH_SPEC §2.3).
