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
- All fases 1–4 are fully defined (`DEV-FASE-N.md`, `Estado: definido`) and
  planned (`task.md`, `Estado: planificado`).
- Each fase maps to one PRD milestone: F1 = skeleton (no business logic),
  F2 = crypto/storage/CRUD, F3 = UX (generator, search, clipboard, auto-lock),
  F4 = Polish (config, master password change, final tests/docs). Don't scope
  features of a later fase into an earlier one; each `DEV-FASE-N.md` §1.2 lists
  what's excluded.
- Every task.md repeats the same conventions: no `panic` on any controllable
  condition, logic as pure functions testable without TTY (UI only draws;
  decisions live in `app.rs`).

## Planned stack (for writing specs, not code)

Rust edition 2021+, `ratatui` + `crossterm` (TUI), `argon2` (Argon2id KDF) +
`chacha20poly1305` (XChaCha20-Poly1305), `rand` `OsRng`, `arboard` clipboard.
Crates are introduced per fase: F1 = `crossterm` + `ratatui`; F2 adds
`argon2`, `chacha20poly1305`, `rand`, `zeroize`, `serde`/`serde_json`, `uuid`,
`time`; F3 adds `arboard`; F4 adds none. Task docs specify test conventions:
modules `mod test_<modulo>`, cases `fn <comportamiento>_<condición>`, per-task
DoD = `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
