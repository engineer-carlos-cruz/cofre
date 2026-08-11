# PRD — Cofre

> Terminal password manager (TUI).

## 1. Overview

**Cofre** is a command-line password manager with a TUI (Text User Interface)
written in Rust. It stores credentials encrypted on disk and unlocks with a
master password. It is designed for personal, offline-first use: no cloud
services, no servers, a single binary.

## 2. Goals

- Store credentials securely (encryption at rest).
- Provide a fast, comfortable terminal experience.
- Ship as a single self-contained, portable binary with no runtime dependencies.

## 3. Non-goals (out of scope)

- Cloud sync / multi-device (possible future extension).
- Browser autofill / desktop app integrations.
- Sharing vaults between users.

## 4. Tech stack

| Layer | Choice | Notes |
|---|---|---|
| Language | Rust (edition 2021+) | Requires installing the toolchain (`rustup`) |
| TUI | `ratatui` + `crossterm` | Cross-platform terminal backend |
| Crypto | `argon2` (KDF) + `chacha20poly1305` | Master password → encryption key |
| Storage | Encrypted binary `.cofre` file | Own format with version + salt + nonce + MAC |
| Secure RNG | `rand` (`OsRng`) | For salts, nonces, and password generation |
| Clipboard | `arboard` | Copy credentials with automatic clearing |

## 5. MVP features

### 5.1 Unlock & security

- Master password requested at startup (hidden input, no echo).
- Key derivation with **Argon2id** (random per-vault salt).
- Vault encryption with **XChaCha20-Poly1305** (authenticated encryption).
- Master password and derived key are never stored on disk.
- **Auto-lock** after N minutes of inactivity (configurable).

### 5.2 Entry management (CRUD)

Each entry contains:

- Title
- Username
- Password
- URL (optional)
- Notes (optional)
- Tags (optional, multiple)

Operations: create, edit, delete, list, view details.

### 5.3 Password generator

- Configurable length (default 20).
- Configurable charset: lowercase, uppercase, digits, symbols.
- Option to **avoid ambiguous characters** (0/O, 1/l/I).
- "One of each class" option to guarantee at least one character of each type.

### 5.4 Search & filtering

- Incremental search by title, username, URL, or tag.
- Filter by tag.

### 5.5 Clipboard

- Copy password / username with a single key.
- **Automatic clearing** of the clipboard after N seconds (default 15) and on
  app exit.

## 6. Main commands / screens

| Screen | Purpose |
|---|---|
| `unlock` | Ask for the master password (or create a new vault) |
| `list` | Entries with search and filters |
| `detail` | View credential, copy, edit, delete |
| `form` | Create/edit an entry |
| `generator` | Generate and save a password |
| `settings` | Change master password, auto-lock, clipboard timeout |

## 7. Keyboard shortcuts (initial proposal)

| Key | Action |
|---|---|
| `↑/↓` or `j/k` | Navigate list |
| `/` | Search |
| `Enter` | Open detail |
| `c` | Copy password |
| `n` | New entry |
| `e` | Edit entry |
| `d` | Delete entry |
| `g` | Open generator |
| `q` / `Esc` | Go back / quit |
| `Ctrl+L` | Lock vault |

*(Shortcuts will be confirmed in the UX document during implementation.)*

## 8. Project structure (proposal)

```text
cofre/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point, terminal init
│   ├── app.rs             # App state, screen navigation
│   ├── ui/                # TUI components (ratatui)
│   ├── crypto.rs          # Argon2id + XChaCha20-Poly1305 + format
│   ├── storage.rs         # Read/write of the .cofre file
│   ├── models.rs          # Entry, Vault, config
│   ├── password.rs        # Generator and strength analysis
│   └── config.rs          # User configuration
└── docs/
    └── PRD.md             # This document
```

## 9. Vault file format

```text
┌─────────────────────────────┐
│ Header: magic "COFRE1"      │
│ + format version (u8)       │
├─────────────────────────────┤
│ Argon2id salt (16 B)        │
│ XChaCha nonce (24 B)        │
├─────────────────────────────┤
│ Encrypted payload (JSON)    │
│ + Poly1305 tag (16 B)       │
└─────────────────────────────┘
```

- The JSON payload contains the entries, the configuration, and the last
  modification date.
- The format version enables future migrations.

## 10. MVP acceptance criteria

- [ ] Create a new vault with a master password.
- [ ] Open an existing vault with the correct password and reject the wrong one.
- [ ] Create, list, search, edit, and delete entries.
- [ ] Generate passwords with the configured options.
- [ ] Copy credentials to the clipboard with automatic clearing.
- [ ] Auto-lock on inactivity.
- [ ] The `.cofre` file cannot be read without the master password.

## 11. Possible future extensions

- **TOTP / 2FA** codes generated in the TUI.
- Weak or reused password detection.
- Import/Export in CSV and Keepass (kdbx) formats.
- Backup via Git or encrypted files.
- **FIDO2** support as a second unlock factor.
- Recovery kit (backup codes).
- Multiple vaults / profiles.
- Automated security tests and format auditing.

## 12. Environment requirements

- Rust toolchain via `rustup` (currently not installed on this machine).
- ANSI-compatible terminal (Linux/macOS; Windows with crossterm support).

## 13. Suggested milestones

1. **M1 — Skeleton**: `cargo new`, ratatui terminal, basic navigation.
2. **M2 — Crypto & storage**: `.cofre` format, create/open vault, CRUD.
3. **M3 — UX**: search, generator, clipboard, auto-lock.
4. **M4 — Polish**: configuration, clear errors, tests and documentation.
