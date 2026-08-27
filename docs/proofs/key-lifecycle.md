# Account key lifecycle proof

Date: 2026-08-27

Status: Proposed input for the security ADR

## Purpose

This proof tests the account key lifecycle before the application stores health data.

The proof is not the production key store. A security review must approve the design before the application uses it for real data.

## Key hierarchy

The proof creates one random 256-bit account data key for each account. The application does not derive this key from a passphrase.

The proof creates a separate random 256-bit recovery key. It makes two envelopes for the same account data key:

- A passphrase envelope supports normal unlock and passphrase changes.
- A recovery envelope supports recovery without the old passphrase.

The recovery key uses the versioned form `HTRK1-<base64url>`. The payload contains the 32-byte recovery key and the first four bytes of its SHA-256 digest. The checksum detects typing and storage errors. It is not an authentication mechanism. The authenticated recovery envelope detects a valid but wrong recovery key.

The server stores the envelopes. The server must not store the passphrase, recovery key, wrapping keys, account data key, or derived purpose keys.

The proof uses HKDF-SHA-256 to derive purpose keys from the account data key. The derivation binds the account identifier, purpose, format version, and key generation. The current purposes are the database, source files, and sync manifest.

## Passphrase parameters

The proposed version 1 defaults are:

| Field | Value |
| --- | --- |
| Algorithm | Argon2id version 19 |
| Memory | 65,536 KiB |
| Iterations | 3 |
| Lanes | 1 |
| Salt | 16 random bytes |
| Output | 32 bytes |

These parameters exceed the current OWASP minimum. The project must measure unlock time on the slowest supported computer before it accepts the ADR. A later envelope version can change these parameters.

## Envelope format

The envelope is a versioned JSON object. Binary fields use unpadded Base64URL. Version 1 has these fields:

| Field | Meaning |
| --- | --- |
| `format` | Fixed value `hemo-tracker-key-envelope` |
| `version` | Envelope version |
| `account_id` | Account that owns the key |
| `protector` | Passphrase KDF data or the recovery-key type |
| `cipher` | Fixed value `xchacha20-poly1305` |
| `nonce` | Random 24-byte nonce |
| `ciphertext` | Encrypted account data key and authentication tag |

Authenticated data binds the format, version, account identifier, cipher, protector type, and all KDF parameters. A change to this metadata causes unlock to fail.

The proof uses a new nonce for every envelope. A passphrase change decrypts only the account data key and creates a new passphrase envelope. It does not re-encrypt clinical data.

## Error and secret handling

Wrong credentials and changed encrypted content return one error: `credentials or key envelope are invalid`. The error does not contain a passphrase, key, nonce, salt, or ciphertext.

Owned passphrases and key buffers use `zeroize` on drop. This action reduces the lifetime of secret data. It cannot remove every compiler, operating-system, or hardware copy.

The example program prints only the envelope version, account identifier, and success status. It does not print secret material.

## Proof commands

Run these commands from the repository root:

```sh
bun run test:key-proof
cargo clippy --manifest-path proofs/key-lifecycle/Cargo.toml --all-targets -- -D warnings
cargo run --manifest-path proofs/key-lifecycle/Cargo.toml
```

The automated tests prove these behaviors:

- Separate account creation produces separate account data keys.
- The passphrase envelope restores the account data key.
- The recovery envelope restores the same account data key.
- A passphrase change keeps all purpose keys stable.
- A wrong passphrase and changed ciphertext return the same safe error.
- A purpose or generation change produces a separate purpose key.
- A serialized envelope retains its version and account binding.

## Limits and next decisions

The proof keeps secrets in process memory. It does not integrate an operating-system credential store, SQLCipher, source files, synchronization, or account recovery screens.

## Rotation rules

A passphrase change increments no key generation. It creates a new passphrase envelope for the same account data key. Recovery-key replacement creates a new recovery key and recovery envelope for the same account data key. The application must delete the replaced envelope after it confirms that the new envelope works.

A purpose-key rotation increments the generation for one purpose. New writes use only the new generation. Existing records retain their generation and remain readable with the old-generation purpose key. A background migration can re-encrypt old records. The application can retire an old generation only after it confirms that no stored object refers to that generation.

An account data key rotates only after suspected key compromise or a cryptographic format migration. This rotation requires re-encryption of every purpose-protected object and creation of new passphrase and recovery envelopes. The application must keep the old account data key available until it verifies every migrated object. It must then remove the old envelopes and key material.

The security ADR must define recovery-key presentation and backup, device enrollment, key revocation, parameter migration, and the maximum accepted KDF cost. An independent security reviewer must review the hierarchy, envelope encoding, authenticated data, nonce rules, recovery design, and error behavior.
