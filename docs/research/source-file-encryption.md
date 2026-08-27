# Source-file encryption research

Date: 2026-08-27

## Question

Which maintained format can encrypt large source files with bounded memory and strong ordering checks?

## Findings

Libsodium `crypto_secretstream_xchacha20poly1305` encrypts an ordered sequence of messages. It authenticates each message and optional additional data. It detects a changed, removed, reordered, or duplicated message. A final tag marks the required end of a stream. The construction can rekey automatically before its counter limit. See the [libsodium secretstream documentation](https://libsodium.gitbook.io/doc/secret-key_cryptography/secretstream).

The `libsodium-rs` crate provides typed Rust bindings for secretstream. Version 0.2.4 uses the maintained `libsodium-sys-stable` binding. The selected binding bundles libsodium 1.0.22 for the supported native targets. See the [`libsodium-rs` repository](https://github.com/jedisct1/libsodium-rs) and [`libsodium-rs` documentation](https://docs.rs/libsodium-rs/latest/libsodium_rs/crypto_secretstream/).

The `dryoc` project implements compatible cryptography in pure Rust. Its project documentation states that it has no third-party security audit. The proof does not select it for V1. See the [`dryoc` repository](https://github.com/brndnmtthws/dryoc).

The age format is a maintained and interoperable file format. Its normal recipient model does not directly match the account purpose-key hierarchy and required account-object associated data. The proof does not select it for V1. See the [`rage` repository](https://github.com/str4d/rage).

## Recommendation

Use libsodium secretstream through pinned `libsodium-rs` and `libsodium-sys-stable` versions. Add a small versioned container only for deterministic frame boundaries and encrypted application metadata. Do not change the secretstream algorithm.

Derive one file key from the account source-file purpose key, account identifier, format version, and opaque object identifier. Bind the same context and the frame index as additional authenticated data for every frame.

Use a 1 MiB plaintext chunk. Store the original filename, media type, plaintext byte count, and SHA-256 checksum only in the encrypted final metadata frame. Require the secretstream final tag and reject any bytes after it.

Write encryption output to a ciphertext partial file that uses the opaque object identifier. Flush and synchronize it before one atomic rename. Remove the partial ciphertext after an error. Do not create a plaintext temporary file.

The Rust wrapper and the small container need an independent security review. The review must check framing limits, additional-data encoding, key derivation, final-tag enforcement, partial plaintext behavior during decryption, and native library provenance.
