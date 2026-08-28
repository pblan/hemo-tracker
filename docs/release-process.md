# Release process

## V1 release type

V1 is an unsigned local desktop release for macOS and Windows. It has no server, Google identity, synchronization, signed update, notarization, or verified publisher identity.

Do not describe a V1 package as signed, notarized, trusted, or production-ready for medical data. Each release note and installation guide must explain the operating-system warning and the risk of an unverified publisher.

## Pipeline roles

The routine CI workflow runs formatting, lint, type checks, unit tests, and the webview smoke test. It runs on each pull request and each push to `main`. It cancels an older run for the same branch. The target duration is less than two minutes.

The native validation workflow builds, packages, and starts the macOS and Windows clients. It runs when native or packaging inputs change. A maintainer can also start it manually. Routine documentation and TypeScript-only changes do not start native packaging.

Each security proof has a separate path-filtered workflow. A proof change runs only its required targets. Exceptional large-file and platform-store checks run only after a manual request.

The local release workflow runs only for a semantic-version tag. It builds unsigned packages from the tag commit. It does not receive signing credentials and does not generate updater metadata.

## Version tags

Use tags in the form `vMAJOR.MINOR.PATCH`. Use a pre-release suffix only for a test release, for example `v1.0.0-rc.1`.

Create a release tag only from a reviewed commit on `main`. The application version, release tag, and release title must contain the same version. The release workflow must reject a mismatch. It must not create or move major or minor alias tags.

## Local release workflow requirements

- Permit only one release workflow at a time.
- Use minimum GitHub token permissions.
- Pin third-party release actions to an immutable release or commit.
- Build unsigned macOS and Windows artifacts from the tag commit.
- Generate SHA-256 checksums and a software bill of materials.
- Create a draft GitHub release with a clear unsigned-build warning.
- Require a maintainer to inspect and publish the draft.
- Never include account vaults, source files, credentials, or clinical fixtures in artifacts.

## V1 release gate

Do not create the V1 tag until the release issue is the only open issue in the “V1: Local desktop app” milestone. The independent local security review must have no unresolved release-blocking finding.

Feature tickets can close before security-hardening tickets. They must preserve the accepted encrypted formats and Rust/webview boundary. Use only deterministic fictional data until the security gate passes.

The exact tag commit must pass routine CI, native validation, documentation checks, and the local security-proof workflows. A maintainer must verify installation and the complete local workflow on macOS and Windows before publication.

## Post-V1 release gate

Signed distribution and updates require a separate accepted ADR and proof. That later work must use a protected release environment, external signing-key custody, signed and notarized macOS artifacts, signed Windows artifacts, and signed updater metadata.
