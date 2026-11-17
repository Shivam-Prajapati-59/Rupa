# ADR-0002: The project is called Rupa and is licensed MIT OR Apache-2.0

- **Status:** accepted
- **Date:** 2026-11-15
- **Spec:** D13 (`TECH_SPEC.md` §0.4) and open decisions 1 and 4 in §12.1, owner slice P0-S6

> The spec numbers its first two ADRs as "own daemon" (ADR-0001) and "name, licence" (ADR-0002), and cites `ADR-0001` elsewhere by that meaning. This record keeps that numbering.

## Context

The project needed a working name and a licence before any code is published. herdr, the reference product, is Apache-2.0 **[V]**, and Rupa studies its design.

## Decision

- **Name: Rupa.** The binary is `rupa`, internal crates are `rupa-*`, and environment variables use the `RUPA_` prefix. Renaming later is one find-and-replace, which the spec is written to allow.
- **Licence: `MIT OR Apache-2.0`**, shipped as `LICENSE-MIT` and `LICENSE-APACHE`. `NOTICE` credits herdr for any adapted material.

## Consequences

- Any text or constant adapted from herdr must be marked in the source with its herdr path and version, and kept in `NOTICE` (risk R14).
- **Open:** availability of the name `rupa` on crates.io and as a command on common package managers has not been checked. Do that before the first public release.
