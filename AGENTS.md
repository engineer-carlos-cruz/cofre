# AGENTS.md

## What this repo is

**Cofre**, a Rust TUI password manager. Spec-first: docs under `docs/` are the source of truth and implementation is driven task-by-task from them. Progress lives in `docs/specs/checklist-task.md`: only **T-SK-01** is `[x]` (1/77). `src/` is just the module skeleton — `main.rs` orchestration, `app.rs` (`Screen`/`AppState`), `terminal.rs` stub (no-op init/teardown), `ui/` (one placeholder frame), `errors.rs`. There is no real terminal loop, crypto, storage, CRUD, or UX yet.

Do not implement features ahead of their task: each `DEV-FASE-N.md` §1.2 lists what is out of scope, and business modules (`crypto`, `storage`, `models`, `password`, `clipboard`, `config`) belong to fases 2–4. Mark a task `[x]` only when its DoD passes.

## Commands (per-task DoD, run in this order)

The Rust toolchain **is installed** (`cargo` 1.97.x). No CI, no pre-commit hooks — verification is local:

```
cargo fmt --check          # must show zero diffs
cargo clippy -- -D warnings
cargo test                 # single test: cargo test <nombre_caso>
```

- As of this writing the skeleton **fails** `fmt --check` (formatting drift) and clippy (dead_code on unused `Screen` variants under `-D warnings`); only `cargo test` is green. This is pre-existing — fix it forward within the task you're doing instead of assuming you broke it.
- `cargo run` draws a single static `unlock` frame and exits (event loop arrives with T-SK-02/T-EVT-01).
- `Cargo.lock` is committed deliberately (deps pinned per task spec); update it, don't delete it.

## Conventions

- All docs and commit messages in **Spanish**. Commits are conventional; history shows `docs:`, `feat:`, `build:` prefixes.
- Every doc carries a status line (`Estado: borrador v1` / `definido` / `planificado`) — keep it on new docs.
- Tests: module `mod test_<modulo>`, cases `fn <comportamiento>_<condición>` (example in `errors.rs`). Test naming is Spanish.
- No `panic!` on any controllable condition. Decisions (screens, layout, exit confirmation, error mapping) are pure functions in `app.rs`/`errors.rs` testable without TTY; UI code only draws.
- Crates are added only when the active fase's `T-00-*` task says so — don't pre-add dependencies.

## Doc hierarchy (derivation chain)

```
docs/PRD.md                         product requirements (milestones M1–M4)
docs/SPEC.md                        technical spec (draft v1)
docs/specs/<fase-N>/DEV-FASE-N.md   phase definition (what + user stories + DoD)
docs/specs/<fase-N>/task.md         atomic task breakdown (how, derived from DEV)
docs/specs/checklist-task.md        global tracker of every T-* task
```

- `SPEC.md` is the technical source of truth (crypto, `.cofre` format, screens, shortcuts); `PRD.md` is product-level.
- When task.md files are added/renamed/removed, sync `checklist-task.md`.
- Each fase maps to one PRD milestone: F1 = skeleton, F2 = crypto/storage/CRUD, F3 = UX (generator, search, clipboard, auto-lock), F4 = polish (config, master password change, integration tests).
- Known stale spot: PRD (§ tech constraints) says the Rust toolchain is "not installed on this machine" — that predates the local install; trust the machine over that prose.

## Stack

Rust edition 2021, `ratatui` + `crossterm` (already pinned in `Cargo.toml`). Planned additions: F2 = `argon2` (Argon2id), `chacha20poly1305` (XChaCha20-Poly1305), `rand` (`OsRng`), `zeroize`, `serde`/`serde_json`, `uuid`, `time`; F3 = `arboard`; F4 = none.
