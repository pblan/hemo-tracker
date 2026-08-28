---
status: accepted
---

# Use versioned local cryptographic formats

## Context

The local account vault needs stable key, database, and source-file formats before product data depends on them. Proofs 2, 4, and 5 verified the selected formats on macOS and Windows.

## Decision

Create one random 256-bit account data key for each local account vault. Do not derive this key from the passphrase. Create one separate random 256-bit recovery key.

Wrap the account data key in two versioned XChaCha20-Poly1305 envelopes. The passphrase envelope uses Argon2id version 19 with 65,536 KiB memory, three iterations, one lane, a random 16-byte salt, and a 32-byte output. The recovery envelope uses the recovery key. Each envelope uses a new random 24-byte nonce.

Encode envelope binary fields as unpadded Base64URL. Authenticate the format, version, local account identifier, cipher, protector type, and passphrase parameters as additional data. Use the recovery-key text format `HTRK1-<base64url>`. Its payload contains the 32-byte recovery key and the first four bytes of its SHA-256 digest.

Use HKDF-SHA-256 to derive separate database and source-file purpose keys. Bind the local account identifier, purpose, format version, and key generation. V1 does not derive a synchronization-manifest key.

Use SQLCipher Community Edition 4.14.0 through `rusqlite` 0.40.1 and `libsqlite3-sys` 0.38.2 with vendored OpenSSL 3.6.3. Use a raw 32-byte database purpose key. Require in-memory temporary storage, secure deletion, WAL mode, full synchronization, and foreign-key enforcement. Keep the file path, key, SQL, connection, and cipher options behind the Rust account vault module.

Encrypt each source file with libsodium secretstream XChaCha20-Poly1305 through `libsodium-rs` 0.2.4 and `libsodium-sys-stable` 1.24.0. Version 1 uses the magic bytes `HEMOSRC` plus one zero byte, one version byte, a 24-byte secretstream header, encrypted data frames, and one final encrypted metadata frame. Use at most 1 MiB of plaintext per data frame.

Derive a separate file key from the source-file purpose key for each random 256-bit opaque object identifier. Authenticate the local account identifier, opaque object identifier, format version, frame type, and frame index. Encrypt the original file name, media type, SHA-256 checksum, and plaintext byte count in the final frame.

Write source ciphertext to an opaque partial path. Synchronize it and rename it only after the final frame succeeds. Never create a plaintext temporary file.

## Consequences

A passphrase change creates a new passphrase envelope. It does not re-encrypt clinical data. Recovery-key replacement creates a new recovery envelope. A purpose-key generation change applies only to new writes until a verified migration completes.

The webview receives typed domain results. It does not receive raw keys, SQL, vault paths, source-file paths, or cipher configuration.

An independent security review must approve the key hierarchy, envelope encoding, nonce rules, SQLCipher build, source-file container, recovery flow, and failure handling before V1 stores real medical data.
