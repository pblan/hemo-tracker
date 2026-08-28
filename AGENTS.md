# Agent instructions

All maintained documentation must follow ASD-STE100.

Use short and direct sentences. Use one term for one concept. Use active voice where possible. This rule applies to Markdown files, ADRs, user instructions, and error messages.

Do not change source laboratory text, citations, code identifiers, or generated data to meet ASD-STE100.

Read relevant domain documentation and ADRs before you change the system.

Use the source that owns a technical claim. Treat design patterns as options. Do not add a future abstraction without a current acceptance criterion. See `docs/research/engineering-authority-guidance.md` when you make a data-model, architecture, UI, accessibility, or engineering-evidence decision.

Keep source facts separate from derived views. Put one complete domain action in one transaction when possible. Add failure evidence before you claim crash safety.

Keep encryption keys, SQL, cipher settings, and local paths outside the WebView. Test changed user workflows with keyboard input and platform assistive technology.

Create a superseding ADR when an accepted decision changes. Do not rewrite the accepted ADR.

## Agent skills

### Issue tracker

Track issues and specifications in GitHub Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the five default triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

Use the single-context domain layout. See `docs/agents/domain.md`.
