# hemo-tracker

Hemo Tracker stores and plots laboratory results. V1 keeps source files and
manual measurements in an encrypted local account vault. V1 runs without a
server.

The project does not give medical advice.

## Requirements

- Bun 1.3.5 or later in the 1.x series.
- The stable Rust toolchain.
- The Tauri 2 platform prerequisites for macOS or Windows.

## Quick start

```sh
bun install
bun run dev:desktop
```

## Commands

| Command                 | Result                                       |
| ----------------------- | -------------------------------------------- |
| `bun run check`         | Run formatting, lint, types, and unit tests. |
| `bun run test:e2e`      | Run the desktop webview smoke test.          |
| `bun run test:rust`     | Run the native desktop tests.                |
| `bun run build`         | Build all TypeScript workspace packages.     |
| `bun run build:desktop` | Build and package the desktop client.        |
| `bun run dev:desktop`   | Start the Tauri desktop client.              |

## Workspace

- `apps/desktop` contains the Tauri and React desktop client.
- `apps/server` contains deferred post-V1 server scaffolding.
- `crates` contains the production key-lifecycle and encrypted-vault modules.
- `packages/contracts` contains shared validation contracts.
- `proofs` contains focused verification programs for accepted technical
  choices.

## Project documents

- [User documentation](docs/user/)
- [Operator documentation](docs/operations/)
- [Product plan](docs/product-plan.md)
- [Architecture](docs/architecture.md)
- [Local V1 threat model](docs/security/local-threat-model.md)
- [Domain glossary](CONTEXT.md)
- [Architecture decisions](docs/adr/)
- [Security and technology proofs](docs/proofs/)
- [Research notes](docs/research/)
- [Contributor guide](docs/contributing.md)
