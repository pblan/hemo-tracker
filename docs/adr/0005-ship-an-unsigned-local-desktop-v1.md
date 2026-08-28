---
status: accepted
---

# Ship an unsigned local desktop V1

## Context

Hemo Tracker can implement and verify its encrypted local workflows without external identity, server, or signing services. Application signing, notarization, Google OAuth, and trusted update publication require external accounts and credentials that are not available for V1.

The product must make useful progress without weakening the local data-protection design or pretending that an unsigned build has a verified publisher identity.

## Decision

V1 is an unsigned local-first desktop application for macOS and Windows. It has no runtime server dependency. The user creates an account vault with a passphrase and recovery key. The user unlocks the account vault with the passphrase in V1.

V1 includes lab reports, source files, flexible measurements, analyte definitions, unit normalization, personal target ranges, trends, encrypted backup, and local export.

Defer these capabilities to the “Post-V1: Signed sync” milestone:

- Saved trusted-device unlock through a signed application identity.
- Google identity and server whitelist enforcement.
- Encrypted synchronization and multi-device conflicts.
- Self-hosted server operation.
- Signed and notarized packages.
- Signed application updates.

V1 can create unsigned local packages and a version tag. Release notes and user guides must state that the operating system cannot verify the publisher. The release process must not call an unsigned package trusted, signed, or production-ready for medical data.

## Consequences

The V1 critical path no longer needs signing credentials, Google configuration, or server deployment. The local vault and clinical workflows remain compatible with later synchronization because domain entities keep opaque identifiers and version metadata.

Users must enter the passphrase after the application locks or restarts. V1 does not provide device convenience unlock.

The independent security review covers the local threat model, encrypted formats, Tauri boundary, exports, and backups. A later review must cover identity, synchronization, server operation, signing, and updates before those capabilities ship.
