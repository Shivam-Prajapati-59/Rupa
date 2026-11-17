# Documentation index

| Document | What it is |
|---|---|
| [`TECH_SPEC.md`](../TECH_SPEC.md) | **The build contract.** Goals, architecture, design, phases and slices, tests, risks. If code and spec disagree, fix both in the same change (§12.3). |
| [`progress.md`](progress.md) | One checkbox per slice (P0-S1 to P8-S9), with the gate slices marked. |
| [`dev-setup.md`](dev-setup.md) | Toolchain, the Windows build caveats, skills restore, the local herdr reference clone. |
| [`ideas.md`](ideas.md) | Our own ideas beyond herdr, with the conditions for each. |
| [`adr/`](adr/) | Architecture decision records. Start from [`0000-template.md`](adr/0000-template.md). |

## Spec map

| If you want to… | Read |
|---|---|
| understand the product and its limits | [§1 goals](../TECH_SPEC.md), §2, §3 |
| understand the design | §4 architecture, §5 detailed design |
| start a slice | §8 (slices), then the §5.x it links, and Appendix A for manifests |
| see what is proven and what is guessed | evidence tags in §0.3, assumptions in §12.4, audit in Appendix G |
| argue with a decision | decision register in §0.4, then its ADR |

## ADRs

| ADR | Decision |
|---|---|
| [0001](adr/0001-own-daemon-not-tmux.md) | Own daemon owns the PTYs; no tmux dependency (D1) |
| [0002](adr/0002-name-and-licence.md) | Name: Rupa. Licence: MIT OR Apache-2.0 (D13) |

Planned by the spec: ADR-0003 emulator choice (P1-S4, Gate G1), ADR-0004 resize debounce (P1-S8), ADR-0005 status reconciler (P4-S10, Gate G3).

## Reference material

The herdr clone, the eight research reports and the superseded first plan live in `reference/herdr/`, which is **gitignored and local only** (see [`../reference/README.md`](../reference/README.md)). It is study material, not instructions.
