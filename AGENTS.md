# AGENTS.md

## What this repo is

**Cofre**, a Rust TUI password manager. Spec-first: docs under `docs/` are the
source of truth and implementation is driven task-by-task from them. A **Fase 1
skeleton crate already exists** (`Cargo.toml` + `src/`), but it only covers
task **T-SK-01** (module structure + placeholder orchestration); the other 76 of
77 tasks are still planning docs. `src/terminal.rs` is a stub, and there is no
real terminal logic, crypto, storage, CRUD, or UX code yet.

The Rust toolchain is **not installed** (PRD §12), so the DoD commands
(`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`) cannot run
here. Follow only those documented commands; don't invent other `cargo` usage.

## Language convention

All docs and commit messages are in **Spanish**. Write new/edited docs and commit
messages in Spanish. Commit style is conventional: `docs: <short description>`
for doc changes, `feat: <short description>` for code.

## Doc hierarchy (derivation chain)

```
docs/PRD.md                         product requirements (milestones M1–M4)
docs/SPEC.md                        technical spec (draft v1)
docs/specs/<fase-N>/DEV-FASE-N.md   phase definition (what + user stories + DoD)
docs/specs/<fase-N>/task.md         atomic task breakdown (how, derived from DEV)
docs/specs/checklist-task.md        global task tracker (aggregates all task.md IDs)
```

- `checklist-task.md` is the single tracking list of every `T-*` task across all
  fases; when task.md files are added/renamed/removed, keep it in sync.
- Implementation progress is tracked here too: currently only **T-SK-01** is `[x]`.
  `src/` matches the module layout T-SK-01 defines; mark a task `[x]` only when
  its DoD passes. Don't implement features of a task before it (each task.md
  repeats the "no scope creep" rule).

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

## Stack

Rust edition 2021+, `ratatui` + `crossterm` (TUI), `argon2` (Argon2id KDF) +
`chacha20poly1305` (XChaCha20-Poly1305), `rand` `OsRng`, `arboard` clipboard.
Crates are introduced per fase: F1 = `crossterm` + `ratatui` (already pinned in
`Cargo.toml`); F2 adds `argon2`, `chacha20poly1305`, `rand`, `zeroize`,
`serde`/`serde_json`, `uuid`, `time`; F3 adds `arboard`; F4 adds none. Task docs
specify test conventions: modules `mod test_<modulo>`, cases
`fn <comportamiento>_<condición>`, per-task DoD = `cargo fmt --check`,
`cargo clippy -- -D warnings`, `cargo test`.
