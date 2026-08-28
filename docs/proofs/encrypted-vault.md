# Encrypted local vault proof

Date: 2026-08-27

Status: Supporting proof for accepted ADR 0006; specialist review pending

## Purpose

This proof tests the SQLCipher storage and failure paths used by the local
account vault. It is evidence for the implementation, not a substitute for the
independent specialist review required before real medical use.

## Proposed integration

The native vault manager owns the file path and database key. It creates the Rust account vault. The account vault owns the SQLCipher connection.

The webview boundary uses `VaultCommandFacade`. This facade accepts only typed add-report and list-report requests. It cannot accept SQL, a database key, a file path, a connection, or a cipher option. The proof-only probes have a separate `VaultProbe` type. Production Tauri commands must wrap only the command facade.

The proof uses these pinned direct dependencies:

| Component | Version | Use |
| --- | --- | --- |
| `rusqlite` | 0.40.1 | Typed Rust database access and backup support |
| `libsqlite3-sys` | 0.38.2 | Bundled SQLCipher build |
| SQLCipher Community Edition | 4.14.0 | Encrypted SQLite pages |
| `openssl-sys` | 0.9.117 | Rust OpenSSL binding |
| Vendored OpenSSL source | 3.6.3 | SQLCipher cryptographic provider |

The `bundled-sqlcipher-vendored-openssl` feature compiles SQLCipher and OpenSSL from the locked Cargo dependency graph. It does not depend on a system SQLite or OpenSSL installation.

The proof passes a raw random 32-byte database key to SQLCipher. The account key module must derive this key as a database purpose key. The proof does not use a passphrase as the SQLCipher key.

## Database settings

Each connection applies these settings before it runs a domain operation:

| Setting | Value | Reason |
| --- | --- | --- |
| `temp_store` | `MEMORY` | Prevent file-based temporary storage |
| `secure_delete` | `ON` | Overwrite deleted database content when possible |
| `journal_mode` | `WAL` | Support crash recovery and normal concurrent reads |
| `synchronous` | `FULL` | Prefer durability over write speed |
| `foreign_keys` | `ON` | Enforce relational constraints |

The bundled SQLCipher source defines `SQLITE_TEMP_STORE=2`. The runtime check also requires `temp_store` to report memory storage.

Schema changes run in one immediate transaction. The proof stores the schema version in `user_version`. It rejects an unknown schema version.

## Proof results

The test suite proves these behaviors:

- A typed lab report command migrates and reads the version 1 schema.
- The main file does not have the stock SQLite header.
- The main file, WAL file, shared-memory file, and rollback journal do not contain the test source text. The journal probe commits the text first. It then changes the same database page in an open transaction. This action forces the original encrypted page into the rollback journal.
- A stock `sqlite3` tool fails to read the vault when the tool is available.
- A wrong key fails with one safe error.
- A changed database page fails during open or an integrity check.
- SQLCipher page-HMAC checks and the SQLite integrity check pass for a valid vault.
- A subprocess termination during a transaction does not commit the partial lab report.
- A file backup after a truncating checkpoint remains encrypted.
- A restored backup opens with the original key and passes integrity checks.

The crash probe sends the database key and source text through a pipe. It does not put them in command-line arguments, standard output, or standard error.

## Backup and restore rule

The application must take an account-vault backup only after a successful truncating WAL checkpoint. It must copy the encrypted main file to a new file. It must not export plaintext SQLite data for a normal backup.

Restore must open the backup with the supplied database key. It must run the SQLCipher page-HMAC check and the SQLite integrity check before and after the copy. Restore must not replace the active vault until all checks pass.

Production code must add an atomic file replacement step and directory synchronization. It must retain the previous active vault until the restored vault opens successfully.

## Platform verification

The path-filtered security-proof workflow builds and tests the pinned SQLCipher stack on current GitHub macOS and Windows runners. These jobs do not run for routine TypeScript-only changes.

Both jobs require a stock `sqlite3` command. The Windows job installs the pinned Chocolatey SQLite 3.53.4 package. The test fails if the command is not available. The encrypted header and wrong-key tests also run on both systems.

## Native licenses

The proof includes these native licenses:

- `rusqlite` and `libsqlite3-sys`: MIT.
- SQLCipher Community Edition: BSD 3-Clause style license. Redistribution requires copyright and license notices in source or binary materials.
- SQLite: public domain.
- OpenSSL 3: Apache License 2.0.

The release application must provide a user-accessible third-party notices screen or linked document. It must include the complete SQLCipher and OpenSSL notices. The [SQLCipher license page](https://www.zetetic.net/sqlcipher/license/) states this user-access requirement.

## Commands

Run these commands from the repository root:

```sh
cargo test --locked --manifest-path proofs/encrypted-vault/Cargo.toml
cargo clippy --manifest-path proofs/encrypted-vault/Cargo.toml --all-targets -- -D warnings
```

## Limits and next decisions

The proof uses one small schema and one process. It does not prove multi-process use, large migrations, disk-full behavior, operating-system backup tools, swap behavior, hibernation, crash dumps, forensic deletion, or file-system snapshots.

The accepted V1 ADRs define the supported local format, database-key
generation, and atomic restore policy. Migration recovery and long-term backup
retention remain release-gate evidence. An independent security reviewer must
review SQLCipher settings, build provenance, key handling, sidecar behavior,
backup behavior, and failure errors.
