# Architecture

## System shape

Hemo Tracker V1 is a local desktop application with an encrypted account vault. It has no runtime server dependency.

The desktop client owns all clinical behavior. It decrypts data, validates measurements, normalizes units, searches the account vault, and creates plots. The user unlocks the account vault with a passphrase.

## Technology baseline

The desktop client uses Tauri 2, React 19, TypeScript, Vite, Chakra UI, TanStack Query, Zod, and Bun tooling. Tests use Vitest and Playwright.

Rust owns key operations, decrypted database access, source-file encryption, native credential storage, and other security-sensitive native functions. The webview does not receive raw encryption keys or SQL access.

The plot adapter uses uPlot as accepted in ADR 0004.

## Main modules

### Account vault module

The account vault module is the main client seam. Its interface exposes domain operations. It does not expose SQL, encryption keys, file paths, or cipher options.

It owns these behaviors:

- Lab report storage.
- Measurement storage and validation.
- Analyte definition storage.
- Personal target range storage.
- Encrypted source-file storage.
- Search and filtering.
- Export and backup.
- Entity versions for synchronization.

A SQLCipher adapter implements local storage. Tests use an in-memory adapter only when it exercises the same interface rules.

### Key module

The key module creates and unwraps account, device, database, and source-file keys. It owns versioned key envelopes, recovery, passphrase change, random values, and domain separation.

Candidate tools include RustCrypto Argon2id, XChaCha20-Poly1305, `getrandom`, and `zeroize`. The security proof must select exact formats and parameters before an ADR accepts them.

### Normalization module

The normalization module receives a source measurement and an analyte definition. It returns one of three results:

- A safe normalized value.
- A value that needs a reviewed analyte-specific rule.
- A blocked conversion with a reason.

It uses UCUM for commensurable units. It never changes source facts.

### Trend module

The trend module builds display-ready series. It selects comparable measurements, calculates range segments, adds personal target ranges, and marks comparability boundaries.

The plot adapter receives engine-neutral points, intervals, boundaries, and theme tokens. Feature code must not use uPlot or ECharts option objects.

### Deferred synchronization module

The synchronization module transfers encrypted objects and manifests. It compares entity versions. It preserves both versions when the same entity changed on two devices.

The module does not inspect clinical content on the server. The trusted client resolves conflicts.

### Deferred identity module

The identity module uses Google OAuth Authorization Code with PKCE. The desktop client uses the system browser and a temporary loopback callback. The server checks the Google identity against its email whitelist.

Identity grants access to account ciphertext. It does not decrypt the account vault.

## Data ownership

One local account vault owns these encrypted entities:

- Lab reports.
- Source-file manifests.
- Measurements.
- Analyte definitions.
- Personal target ranges.
- User settings.

## Source and derived data

Source data includes source files, source value text, source unit text, source reference intervals, source flags, source labels, and report metadata.

Derived data includes parsed numeric values, normalized values, comparability decisions, plot series, and overview summaries.

The application can recalculate derived data when an analyte definition changes. It must not change source data during recalculation.

## Local storage

The client uses an encrypted local SQLite vault. It encrypts each source file before it writes the file to persistent storage. V1 does not store a device-unlock key. The user supplies the passphrase after lock or restart.

## Deferred synchronization

Each synchronized entity has an opaque identifier and a version. The client sends only encrypted entity content and permitted operations metadata.

If two devices change different entities, synchronization applies both changes. If two devices change the same entity version, the server keeps both encrypted candidates. A trusted client asks the user to select one candidate.

## Local distribution

V1 creates unsigned local packages for macOS and Windows. The user guides must explain the operating-system warning and the lack of a verified publisher identity. Signed distribution and updates are deferred.

## Security verification

The project must complete these proofs before it stores real medical data:

- Key creation, wrapping, recovery, passphrase change, and rotation.
- SQLCipher files, journals, temporary files, backups, and crash behavior.
- Streaming encryption for large source files.
- Log, telemetry, crash-report, clipboard, export, and screenshot review.

An independent specialist must review the local threat model, key hierarchy, encrypted formats, Tauri capabilities, backups, exports, and unsigned distribution limits.
