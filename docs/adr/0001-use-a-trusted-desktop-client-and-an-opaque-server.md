---
status: accepted
---

# Use a trusted desktop client and an opaque server

ADR 0005 narrows this decision for V1. The local V1 has no server and uses unsigned packages. This ADR governs the Post-V1 signed synchronization milestone.

Hemo Tracker must protect clinical content from other users and from an administrator who controls the self-hosted server. A server-hosted web client cannot meet this requirement because the administrator can replace the client code. Use a separately installed and signed desktop client. The client owns decryption and clinical processing. The server stores identity data, permitted operations metadata, wrapped keys, and opaque ciphertext only.

## Considered options

- Server-side encryption was simpler, but the server could obtain plaintext and keys.
- A browser client with client-side encryption protected stored data, but a server administrator could replace the served code.
- A trusted desktop client adds release and platform work, but it keeps keys outside the server trust boundary.

## Consequences

- The project must sign macOS and Windows releases and updates.
- The update signing key must stay outside the data server.
- Google authentication controls ciphertext access. It does not decrypt clinical data.
- The client must support a separate passphrase, recovery key, and trusted devices.
- Server-side clinical search, plotting, extraction, and normalization are not possible.
- A compromised trusted device can expose plaintext while the vault is unlocked.
