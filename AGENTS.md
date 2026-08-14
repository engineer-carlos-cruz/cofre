# AGENTS.md

## What this repo is

Documentation-only repository for **Cofre**, a Rust TUI password manager. There is
**no source code yet** — only planning/spec docs. The Rust toolchain is not
installed (PRD §12) and there are no Cargo/build/test/lint commands to run. Do not
invent or run `cargo` commands.

## Language convention

All docs and commit messages are in **Spanish**. Write new/edited docs and commit
messages in Spanish. Commit style is conventional: `docs: <short description>`.

## Doc hierarchy (derivation chain)

```
docs/PRD.md                         product requirements (milestones M1–M4)
docs/SPEC.md                        technical spec (draft v1)
docs/specs/<fase-N>/DEV-FASE-N.md   phase definition (what + user stories + DoD)
docs/specs/<fase-N>/task.md         atomic task breakdown (how, derived from DEV)
```

- `SPEC.md` is the technical source of truth (crypto, `.cofre` format, screens,
  shortcuts); `PRD.md` is product-level.
- Each doc carries a status line (e.g. `Estado: borrador v1`, `definido`,
  `planificado`). Keep this convention for new docs.
- Fase 1 (`fase-1/`) is fully defined (`DEV-FASE-1.md`) and planned (`task.md`).
  Fase 2 (`fase-2/`) exists only as empty stub files — fill them following the
  `fase-1/` format when defined.
- Fase 1 explicitly excludes crypto/storage/CRUD/generator/clipboard/auto-lock
  (those land in M2+); don't scope business features into fase 1.

## Planned stack (for writing specs, not code)

Rust edition 2021+, `ratatui` + `crossterm` (TUI), `argon2` (Argon2id KDF) +
`chacha20poly1305` (XChaCha20-Poly1305), `rand` `OsRng`, `arboard` clipboard.
Task docs specify test conventions: modules `mod test_<modulo>`, cases
`fn <comportamiento>_<condición>`, per-task DoD = `cargo fmt --check`,
`cargo clippy -- -D warnings`, `cargo test`.
