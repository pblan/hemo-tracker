# hemo-tracker

Hemo Tracker stores and plots laboratory results. It keeps source files and
manual result entries in an encrypted local vault. A self-hosted server
synchronizes ciphertext between trusted desktop clients.

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

Run the self-hosted server in another terminal:

```sh
bun run dev:server
```

The server health route is `http://localhost:3000/health`.

## Commands

| Command               | Result                                       |
| --------------------- | -------------------------------------------- |
| `bun run check`       | Run formatting, lint, types, and unit tests. |
| `bun run test:e2e`    | Run the desktop webview smoke test.          |
| `bun run build`       | Build all TypeScript workspace packages.     |
| `bun run dev:desktop` | Start the Tauri desktop client.              |
| `bun run dev:server`  | Start the self-hosted server.                |

## Workspace

- `apps/desktop` contains the Tauri and React desktop client.
- `apps/server` contains the self-hosted TypeScript server.
- `packages/contracts` contains shared validation contracts.

## Project documents

- [Product plan](docs/product-plan.md)
- [Architecture](docs/architecture.md)
- [Domain glossary](CONTEXT.md)
- [Architecture decisions](docs/adr/)
- [Research notes](docs/research/)
