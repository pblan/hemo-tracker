# Security tooling for a Tauri desktop client

Date: 2026-08-27

## Purpose

This note reviews maintained security tooling for a Tauri 2 client on macOS and Windows.

This note separates documented facts from project recommendations. It does not make a product decision.

## Documented facts

### Tauri security boundary

Tauri 2 uses explicit plugin permissions and capabilities. Each plugin adds a security surface. The application must give each window only the permissions that it needs. The official [plugin list](https://v2.tauri.app/plugin/) includes Stronghold, SQL, updater, deep-link, opener, HTTP, and single-instance plugins.

The official Stronghold plugin stores secrets with the IOTA Stronghold engine. It supports macOS and Windows. The plugin requires a function that derives exactly 32 bytes from a password. Its standard builder uses Argon2 and a salt file. The plugin exposes a JavaScript API. See the [Tauri Stronghold documentation](https://v2.tauri.app/plugin/stronghold/).

Stronghold is an encrypted secret store. The Tauri documentation does not define it as an encrypted relational database. It also does not define the complete account key hierarchy for this project.

The official SQL plugin gives the webview access to databases through `sqlx`. Its documentation does not state that SQLite data is encrypted. Do not treat the SQL plugin as encrypted storage without a separate proof.

### Password-based key wrapping

Argon2id is a memory-hard password key derivation function. The RustCrypto [`argon2` crate](https://docs.rs/argon2/latest/argon2/) has a specific key derivation API. Its default algorithm is Argon2id version 19.

OWASP recommends Argon2id for password storage. Its current minimum example uses 19 MiB of memory, two iterations, and one lane. OWASP also gives equivalent parameter sets. See the [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html).

NIST SP 800-132 defines password-based derivation for keys that protect stored data. NIST plans to revise this publication. See [NIST SP 800-132](https://csrc.nist.gov/pubs/sp/800/132/final).

The application must use a unique random salt for each wrapped key. It must store the algorithm, version, salt, and cost parameters with the wrapped key. This data permits a later parameter upgrade.

### Authenticated encryption

The RustCrypto [`chacha20poly1305` crate](https://docs.rs/chacha20poly1305/latest/chacha20poly1305/) implements ChaCha20-Poly1305 and XChaCha20-Poly1305 authenticated encryption. Its documentation states that one NCC Group audit found no significant issues. A nonce must be unique for each message under one key.

Authenticated encryption detects a changed ciphertext. Associated data can bind a ciphertext to an object identifier, schema version, account identifier, or object type. Associated data is not secret.

RustCrypto [`zeroize`](https://docs.rs/zeroize/latest/zeroize/) prevents the compiler from removing its memory overwrite. It cannot remove all secret copies. It does not protect against hardware leakage, process compromise, swap, or a debugger.

### Secure random values

The Rust [`getrandom` crate](https://docs.rs/getrandom/latest/getrandom/) gets random bytes from operating-system sources. It fails instead of returning known insecure bytes. Higher-level RustCrypto APIs can use the operating-system random source for keys and nonces.

Do not generate keys, salts, nonces, PKCE verifiers, or recovery codes in application JavaScript when the Rust core can generate them.

### Operating-system credential storage

Apple states that Keychain is the correct location for small secrets, passwords, and cryptographic keys. Keychain supports access-control rules and user-presence requirements. See [Storing Keys in the Keychain](https://developer.apple.com/documentation/Security/storing-keys-in-the-keychain) and [Keychain data protection](https://support.apple.com/guide/security/keychain-data-protection-secb0694df1a/web).

Microsoft states that Credential Locker stores credentials for Windows applications. Microsoft limits it to 20 credentials per application and says not to use it for large data blobs. The user must opt in before the application saves a password. See [Credential locker for Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/security/credential-locker).

The maintained Rust [`keyring` ecosystem](https://docs.rs/keyring/latest/keyring/) can access native macOS and Windows stores. Its current documentation tells applications that need exact store control to use `keyring-core` and explicit store crates. This API recently changed its package structure. An integration proof is necessary.

An operating-system credential store can protect a small device-unlock key. It cannot protect plaintext after the application retrieves that key. Its behavior also depends on the signed application identity and the operating-system account.

### Encrypted SQLite

SQLite does not encrypt a database by default. SQLCipher is a maintained SQLite fork that adds full database encryption and other security functions. SQLCipher uses AES-256. It supports a raw 32-byte database key. See the [SQLCipher project documentation](https://www.zetetic.net/sqlcipher/documentation/) and [SQLCipher source mirror](https://github.com/sqlcipher/sqlcipher).

SQLCipher is a C library. A Rust and Tauri application must select, build, and package a compatible SQLite binding. The application must prove this build on each target. It must also prove migrations, backups, temporary files, write-ahead logs, crash recovery, and wrong-key behavior.

The official Tauri SQL plugin does not document SQLCipher support. A custom Rust database module can keep the database key out of the webview API. This module can use SQLCipher through a reviewed Rust binding.

### Google OAuth for an installed application

Google classifies installed applications as public clients. They cannot keep a client secret. Google supports Authorization Code with PKCE for desktop applications. It requires a unique high-entropy verifier for each request. Google recommends the `S256` challenge. The application opens the system browser and receives the result on a local redirect. See [OAuth 2.0 for iOS and desktop apps](https://developers.google.com/identity/protocols/oauth2/native-app).

RFC 8252 requires native applications to use an external user agent. It requires PKCE support. For a loopback callback, it permits HTTP because the request stays on the device. The client must bind only to a loopback IP address, use an ephemeral port, open it only for the request, and close it after the callback. The RFC does not recommend `localhost`; it recommends a loopback IP literal. See [RFC 8252](https://www.rfc-editor.org/rfc/rfc8252).

The OAuth request must use a random `state` value. The client must compare it before it accepts the authorization response. The client must validate the issuer, audience, signature, expiry, and nonce of an OpenID Connect ID token. Google authentication must not serve as the encryption passphrase.

The official Tauri deep-link plugin supports macOS and Windows. Tauri warns that a user can submit a false deep link. The application must validate every callback. See the [Tauri deep-link documentation](https://v2.tauri.app/plugin/deep-linking/).

### Signed applications and updates

The official Tauri updater requires a signature for every update. The application embeds the public update key. The release process keeps the private key secret. Tauri also enforces TLS for production update endpoints unless a dangerous option disables it. See the [Tauri updater documentation](https://v2.tauri.app/plugin/updater/).

An updater signature does not replace platform code signing. Apple notarization and code signing establish the macOS application identity. Authenticode signing reduces Windows SmartScreen warnings and establishes the Windows publisher identity. See [Tauri distribution](https://v2.tauri.app/distribute/), [macOS code signing](https://v2.tauri.app/distribute/sign/macos/), and [Windows code signing](https://v2.tauri.app/distribute/sign/windows/).

The update signing key must not exist on the self-hosted data server. A server administrator who controls both update content and its signing key can replace the trusted client.

## Candidate simple stack

The following items are recommendations for a proof. They are not final decisions.

1. Keep all key operations and decrypted database access in Rust commands. Do not expose raw keys to the webview.
2. Generate account keys, device keys, salts, nonces, recovery codes, PKCE verifiers, and OAuth state with the operating-system random source.
3. Derive a key-encryption key from the user passphrase with RustCrypto Argon2id. Measure the cost on the slowest supported device. Start at or above the current OWASP minimum.
4. Wrap a random account data key with an audited authenticated-encryption implementation. Store a versioned envelope. Bind its account identifier and envelope version as associated data.
5. Use a separate random key for each purpose. Do not use one key for database encryption, object encryption, and key wrapping. Derive or wrap purpose-specific keys with a documented hierarchy.
6. Store only a small device-unlock key in Apple Keychain or Windows Credential Locker. Require explicit device approval. Use the explicit native `keyring-core` store for each platform if its proof succeeds.
7. Test SQLCipher behind a narrow Rust database module. Give the webview typed domain commands instead of SQL access. Keep the official Tauri SQL plugin out of the clinical vault unless a SQLCipher integration proof succeeds.
8. Encrypt each attachment with an authenticated streaming file format. Do not design an ad hoc chunk format. Evaluate a maintained libsodium secretstream binding or the `age` file format in a separate proof. A single-buffer AEAD is not suitable for large files.
9. Use the system browser for Google OAuth. Use Authorization Code with PKCE `S256`, random `state`, and a temporary loopback listener. Store refresh tokens in the encrypted vault or the native credential store. Do not embed a client secret.
10. Use the Tauri updater with its mandatory signature check. Also sign and notarize macOS releases and sign Windows releases. Keep release signing on an isolated release system.
11. Use `zeroize` or a higher-level secret wrapper for owned Rust secret buffers. Do not claim that this removes all memory exposure.
12. Deny webview access to the filesystem, shell, SQL, Stronghold, and network by default. Add the minimum Tauri capabilities for each window.

## Maturity and risk summary

| Area | Candidate | Maturity signal | Main risk |
| --- | --- | --- | --- |
| Tauri integration | Official Tauri 2 plugins | Official and cross-platform | A broad capability can expose a sensitive operation to the webview. |
| Secret store | Tauri Stronghold | Official Tauri plugin | It is not a relational vault. The project still needs a key hierarchy and recovery design. |
| Password KDF | RustCrypto Argon2id | Maintained implementation with a key derivation API | Fixed example parameters can be too weak or too slow on target devices. |
| Record encryption | RustCrypto XChaCha20-Poly1305 | Maintained and audited once | Nonce reuse or incorrect associated data breaks the design. |
| Native secret store | Apple Keychain and Windows Credential Locker through `keyring-core` | Native stores are mature; the Rust facade is maintained | Platform semantics differ. Package APIs have changed. |
| Local database | SQLCipher behind a custom Rust module | Long-running maintained project | Cross-platform build, license, migrations, WAL files, and backup behavior need proof. |
| OAuth | Google installed-app flow and RFC 8252 | Standard provider flow | A false callback, bad token validation, or token leakage can compromise the account session. |
| Updates | Tauri updater plus platform signing | Official tooling | A leaked release key defeats the trusted-client boundary. |
| Memory cleanup | RustCrypto `zeroize` | Maintained narrow utility | It cannot remove every copy or stop a live process attacker. |

## Required proofs before an ADR

### Key lifecycle proof

Build a command-line proof before the user interface exists.

- Create an account data key.
- Wrap and unwrap it with a passphrase-derived key.
- Change the passphrase without re-encrypting clinical data.
- Recover it with a recovery key.
- Reject a wrong passphrase and changed envelope.
- Rotate a purpose-specific key.
- Confirm that logs and errors contain no secret material.

Ask an independent security reviewer to review the envelope format, key hierarchy, recovery design, nonce rules, and domain separation.

### Native credential-store proof

Test signed development builds on macOS and Windows.

- Save and retrieve a device-unlock key.
- Test application upgrade and reinstall behavior.
- Test a changed signing identity.
- Test logout and device revocation.
- Determine whether the store roams or backs up the item.
- Confirm the user-consent behavior on both systems.

### SQLCipher proof

- Build one pinned SQLCipher version for Apple silicon, Intel macOS if supported, and Windows x64.
- Confirm that a stock SQLite tool cannot read the file.
- Inspect the main file, WAL file, shared-memory file, journal, temporary directory, and crash dump path.
- Run schema migrations and interrupted-write tests.
- Test backup, restore, corruption, and wrong-key behavior.
- Record all native dependencies and licenses.

### Attachment encryption proof

- Select a documented streaming format.
- Test multi-gigabyte input without a plaintext temporary file.
- Detect changed, truncated, reordered, and duplicated chunks.
- Bind the account, object identifier, and format version.
- Test interrupted encryption and decryption.

### OAuth proof

- Register the correct Google desktop client type.
- Open the system browser.
- Bind an ephemeral listener to `127.0.0.1` or `[::1]` only.
- Use PKCE `S256`, `state`, and an OpenID Connect nonce.
- Reject a reused code, false callback, wrong state, wrong issuer, wrong audience, and expired token.
- Close the listener after one result or a short timeout.
- Confirm account linking rules for an open-registration server.

### Release-chain proof

- Sign and notarize a macOS build.
- Sign a Windows build.
- Publish a Tauri-signed test update through HTTPS.
- Reject an unsigned update, a changed artifact, an old manifest, and a wrong platform artifact.
- Document offline recovery after loss of the update signing key.
- Keep the update private key outside the self-hosted data server.

## Security review points

A specialist review is required before the project stores real medical data. The review must cover these items:

- Threat model and administrator boundary.
- Key hierarchy and recovery.
- Encrypted object and attachment formats.
- SQLCipher integration and plaintext artifacts.
- Tauri commands, capabilities, content security policy, and webview input handling.
- OAuth callback and token validation.
- Update and release-key custody.
- Logs, telemetry, crash reports, clipboard, screenshots, exports, and temporary files.

No library can make a compromised client device safe. No encrypted local database can protect data while the trusted client displays it. The project must state these limits in its threat model.
