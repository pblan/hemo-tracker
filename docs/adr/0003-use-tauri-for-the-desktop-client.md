---
status: accepted
---

# Use Tauri for the desktop client

Hemo Tracker needs a signed macOS and Windows client with a React interface and a narrow native security module. Use Tauri 2 with React 19, TypeScript, Vite, Chakra UI, TanStack Query, and Zod. Keep keys, decrypted database access, and source-file encryption in Rust commands. This choice keeps the useful parts of the Mood Tracker stack without adding Next.js server rendering inside a desktop application.

## Considered options

- A Next.js web application could reuse more Mood Tracker structure, but it cannot satisfy the active server-administrator threat model.
- Electron has a larger JavaScript runtime and a broader default security surface.
- Tauri adds Rust and native build work, but it supports a small signed client and explicit capabilities.

## Consequences

- Vite replaces Next.js in the desktop client.
- Bun remains the JavaScript package and task tool.
- Chakra UI supplies predefined interface elements.
- Rust code stays limited to native and security-sensitive modules.
- The project must prove SQLCipher, native credential storage, OAuth callbacks, and signed updates on both target systems.
- The plot engine remains behind an engine-neutral interface until a benchmark selects it.
