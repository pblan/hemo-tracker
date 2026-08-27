# Native trusted-device credential proof

Date: 2026-08-28

Status: Platform integration implemented; signed-build behavior pending

## Purpose

This proof tests a small device-unlock key in Apple Keychain and Windows Credential Locker. It does not store the account vault in the credential store.

## Implemented boundary

The Rust API requires `SavedAccessApproval::Approved` before it saves a key. A declined request stores nothing. The API loads the key only into a Rust callback. It does not provide a method that returns raw key bytes to the webview.

Logout and device revocation delete the native credential. A missing item returns a normal `NotFound` result. The application can then require the account passphrase or recovery process.

The proof uses `apple-native-keyring-store` 1.0.2 with `WhenUnlockedThisDeviceOnly` and cloud synchronization disabled. It uses the Windows Runtime `PasswordVault` API through `windows` 0.62.2. Both implementations stay behind one private store interface.

The store keeps a base64 representation of one 32-byte key under a versioned service name and account identifier. Each owned encoded and decoded key buffer uses `zeroize`. This operation does not remove every possible memory copy.

## Verification

Install the stable Rust toolchain. Run the deterministic tests:

```sh
cargo test --locked --manifest-path proofs/native-credentials/Cargo.toml
```

Run the live native-store test on a disposable test account:

```sh
cargo test --locked --manifest-path proofs/native-credentials/Cargo.toml native_store_round_trip_and_revocation -- --ignored --exact
```

The live test saves, reads, verifies, and deletes one credential. A cleanup guard deletes the item after a test failure. If a process stops before cleanup, remove the item with service name `dev.hemo-tracker.device-unlock.v1` from Keychain Access or Windows Credential Manager. The path-filtered workflow compiles and tests the module on macOS and Windows. Its manual mode runs the live store test on both GitHub runner platforms.

## Signed-build gate

The proof is not complete until signed macOS and Windows test applications verify these cases:

- The first save follows explicit user approval.
- An upgrade with the same signing identity keeps access.
- A reinstall has the documented platform behavior.
- A changed signing identity cannot silently use the prior item.
- Logout and remote device revocation remove saved access.
- The macOS item does not synchronize to another device.
- The Windows interface states that Credential Locker can roam with the user's Microsoft account.

Record the certificate identities, bundle or application identity, entitlements, package type, operating-system version, and observed result. Do not record private keys or credential values.

## Proposed ADR input

Use the explicit protected data Keychain and `PasswordVault` integrations behind a project-owned Rust interface. Require application approval before save. Use a non-synchronizing `ThisDeviceOnly` Keychain item on macOS. Document Credential Locker roaming on Windows. Treat native credentials as a convenience unlock factor, not as account recovery or the only copy of an account key.
