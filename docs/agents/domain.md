# Domain documentation

This repository uses a single-context domain layout.

## Read before work starts

Read `CONTEXT.md` before you explore or change the domain model.

Read the relevant ADRs in `docs/adr/` before you change the architecture.

If a file or directory does not exist, continue without a warning. Create domain files only when a term or decision needs them.

## Layout

Use this layout:

```text
/
├── CONTEXT.md
├── docs/
│   └── adr/
└── src/
```

`CONTEXT.md` is a glossary. It must not contain implementation details.

ADRs record important decisions that are difficult to reverse. Each ADR must explain the context, decision, alternatives, and consequences.

## Vocabulary

Use the terms from `CONTEXT.md`.

Do not use a synonym when the glossary defines a preferred term.

If a required term is missing, use the domain-modeling workflow.

## Conflicts

Identify any proposed change that conflicts with an accepted ADR.

Do not override an ADR without a new decision. A replacement ADR must identify the ADR that it supersedes.
