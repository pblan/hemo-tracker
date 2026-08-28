---
status: accepted
---

# Use versioned local cryptographic formats

## Context

The local account vault needs stable key, database, and source-file formats before product data depends on them. Proof 2 verified the key lifecycle on Linux. Proofs 4 and 5 verified SQLCipher and source-file encryption on macOS and Windows.

## Decision

Create one random 256-bit account data key for each local account vault. Do not derive this key from the passphrase. Create one separate random 256-bit recovery key.

Wrap the account data key in two versioned XChaCha20-Poly1305 envelopes. The format identifier is `hemo-tracker-key-envelope`, the numeric version is `1`, and the cipher identifier is `xchacha20-poly1305`. The passphrase envelope uses Argon2id version 19 with 65,536 KiB memory, three iterations, one lane, a random 16-byte salt, and a 32-byte output. The recovery envelope uses the recovery key. Each envelope uses a new random 24-byte nonce.

Encode the envelope as a JSON object with the fields `format`, `version`, `account_id`, `protector`, `cipher`, `nonce`, and `ciphertext`. Encode binary fields as unpadded Base64URL. The passphrase protector is `{"type":"passphrase","kdf":"argon2id-v19","memory_kib":65536,"iterations":3,"lanes":1,"salt":"<base64url>"}`. The recovery protector is `{"type":"recovery-key"}`. JSON field order is not significant.

Build envelope additional data as UTF-8 bytes from `format + NUL + version-decimal + NUL + account_id + NUL + cipher + NUL + protector`. The passphrase protector text is `passphrase:argon2id-v19:65536:3:1:<salt-base64url>`. The recovery protector text is `recovery-key`. Account identifiers must not contain NUL. Use the recovery-key text format `HTRK1-<base64url>`. Its payload contains the 32-byte recovery key and the first four bytes of its SHA-256 digest.

Use HKDF-SHA-256 to derive separate database and source-file purpose keys. Use the UTF-8 local account identifier as the salt. Use exactly `hemo-tracker/v1/database/generation/<generation-decimal>` and `hemo-tracker/v1/source-files/generation/<generation-decimal>` as the respective `info` values. V1 does not derive a synchronization-manifest key. Local account identifiers must not contain NUL and key generation starts at `0`.

Use SQLCipher Community Edition 4.14.0 through `rusqlite` 0.40.1 and `libsqlite3-sys` 0.38.2 with vendored OpenSSL 3.6.3. Use a raw 32-byte database purpose key. Require in-memory temporary storage, secure deletion, WAL mode, full synchronization, and foreign-key enforcement. Keep the file path, key, SQL, connection, and cipher options behind the Rust account vault module.

Encrypt each source file with libsodium secretstream XChaCha20-Poly1305 through `libsodium-rs` 0.2.4 and `libsodium-sys-stable` 1.24.0. Version 1 uses the magic bytes `HEMOSRC` plus one zero byte, one version byte, a 24-byte secretstream header, encrypted data frames, and one final encrypted metadata frame. Use at most 1 MiB of plaintext per data frame.

Derive a separate file key from the source-file purpose key for each random 256-bit opaque object identifier. Use the UTF-8 local account identifier as the HKDF salt and `hemo-tracker/source-file/v1/<lowercase-object-id>` as the exact `info` value. Authenticate each frame with the UTF-8 additional-data value `hemo-tracker-source-file` + NUL + `1` + NUL + account identifier + NUL + object identifier + NUL + frame-type decimal + NUL + frame-index decimal. Data frame type is `0`; metadata frame type is `1`; the first frame index is `0`. The final JSON frame has the fields `original_filename`, `media_type`, `sha256`, and `plaintext_bytes`. The SHA-256 value is lowercase hexadecimal.

Write source ciphertext to an opaque partial path. Synchronize it and rename it only after the final frame succeeds. Never create a plaintext temporary file.

The desktop application accepts source files up to 512 MiB. The streaming
container keeps 1 MiB data frames, and libsodium secretstream performs its
authenticated internal rekeying. V1 does not add an application-level rekey
interval. After the opaque rename, the application synchronizes the containing
directory on Unix systems. Windows relies on the filesystem's durable rename
semantics because directory handles cannot be synchronized portably by this
crate.

## Consequences

A passphrase change creates a new passphrase envelope. It does not re-encrypt clinical data. Recovery-key replacement creates a new recovery envelope. A purpose-key generation change applies only to new writes until a verified migration completes.

V1 accepts only the Argon2id parameters recorded above. It rejects an envelope
with another memory cost, iteration count, lane count, or format version. V1
does not perform an automatic parameter migration during unlock.

The webview receives typed domain results. It does not receive raw keys, SQL, vault paths, source-file paths, or cipher configuration.

Functionality work must preserve these formats and boundaries. An independent security review must approve the key hierarchy, envelope encoding, nonce rules, SQLCipher build, source-file container, recovery flow, and failure handling before V1 is released for real medical data.
