# Release process

## Pipeline roles

The routine CI workflow runs formatting, lint, type checks, unit tests, and the
webview smoke test. It runs on each pull request and each push to `main`. It
cancels an older run for the same branch. The target duration is less than two
minutes.

The native validation workflow builds, packages, and starts the macOS and
Windows clients. It runs when native or packaging inputs change. A maintainer
can also start it manually. Routine documentation and TypeScript-only changes
do not start native packaging.

Each security proof has a separate path-filtered workflow. A proof change runs
only its required targets. The 2 GiB source-file test runs only after a manual
request. It does not run in routine CI.

The release workflow will run only for a semantic-version tag. Issue
“Prove signed applications and updates” owns this workflow. Do not add an
unsigned release workflow before that proof is complete.

## Version tags

Use tags in the form `vMAJOR.MINOR.PATCH`. Use a pre-release suffix only for a
test release, for example `v1.0.0-rc.1`.

Create a release tag only from a reviewed commit on `main`. The application
version, release tag, updater metadata, and release title must contain the same
version.

The release workflow must reject a version mismatch. It must not create or move
major or minor alias tags.

## Release workflow requirements

- Use a protected GitHub `release` environment.
- Permit only one release workflow at a time.
- Use minimum GitHub token permissions.
- Keep macOS, Windows, and Tauri update signing keys outside the data server.
- Pin third-party release actions to an immutable release or commit.
- Build signed macOS and Windows artifacts from the tag commit.
- Generate signed updater artifacts and updater metadata.
- Create a draft GitHub release.
- Attach checksums and a software bill of materials.
- Require a maintainer to inspect and publish the draft.
- Never expose signing values in logs or workflow artifacts.

## Release gate

Do not create the V1 tag until all V1 issues are closed and the independent
security review has no unresolved release-blocking finding.
