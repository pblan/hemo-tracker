# Local V1 threat model

## Scope

This threat model covers the unsigned local V1 desktop application on macOS and Windows. It covers the locked account vault, encrypted source files, encrypted backups, recovery material, decrypted exports, and the Tauri process boundary.

Google identity, a server, synchronization, trusted-device unlock, application signing, and signed updates are outside the V1 boundary.

## Protected assets

- Account data keys and derived purpose keys.
- Passphrases and recovery keys.
- Lab reports, measurements, analyte definitions, and personal target ranges.
- Source files and their private metadata.
- Encrypted backups and decrypted exports.

## Trust boundaries

The Rust process is trusted while it handles keys, SQLCipher connections, and source-file encryption. The webview is less trusted. It can request typed domain operations, but it cannot request SQL, raw keys, arbitrary file paths, shell commands, or cipher options.

The local file system, operating-system backup system, and release download location are not trusted with plaintext clinical content. A user-selected decrypted export is an explicit exception.

## Protected threats

V1 is designed to protect locked clinical content against another local user who can read application files, a copied disk or backup, accidental source-file disclosure from application storage, and use of a wrong passphrase or recovery key.

Authenticated encryption and SQLCipher integrity checks detect changed encrypted content within their documented limits. Atomic replacement rules protect an active account vault from an invalid restore.

## Accepted limits

V1 does not protect clinical content against these threats:

- Malware, a debugger, or an administrator on the unlocked computer.
- Screen capture or observation while the application displays data.
- Plaintext that another application reads from a user-selected decrypted export.
- Swap, hibernation, crash dumps, or hardware leakage that copies process memory.
- Loss of every local copy, encrypted backup, passphrase, and recovery key.
- Replacement of the unsigned application or installer.
- Metadata disclosure from ciphertext size, opaque file count, local timestamps, exports, and operating-system activity.

## Required controls

- Clear decrypted application state on lock.
- Keep raw keys and SQL out of the webview API.
- Keep clinical content out of logs, telemetry, crash reports, notifications, clipboard defaults, and screenshot fixtures.
- Remove temporary decrypted export data after success or failure.
- Require an explicit warning and confirmation before decrypted export.
- Verify encrypted backups before atomic restore.
- State the unsigned publisher limit in installation and release documentation.
- Complete an independent specialist review before use with real medical data.
