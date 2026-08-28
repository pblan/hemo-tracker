---
status: accepted
---

# Seed fictional demo reports in new vaults

## Context

The first local vault has no personal laboratory data. An empty overview makes
the plot, report history, and source-file workflows difficult to inspect. The
repository already has a deterministic fictional fixture for tests, but the
fixture must not be copied into a user account automatically.

## Decision

Create three complete reports when a new local vault is created. Each report
uses the `demo` tag, fictional notes, one encrypted text source file, and
hemoglobin, glucose, and creatinine measurements across three collection dates.
Use representative source units and reference intervals so the trend view can
show normalization and changing dates immediately.

Do not seed demo reports when an existing vault is unlocked. Store the demo
reports through the same encrypted report and source-file paths as user data.
Show a visible notice in report history. The user can archive or permanently
delete the reports with the normal report controls.

## Consequences

- A new user can inspect the main workflow before entering personal data.
- Demo reports consume a small amount of encrypted local storage.
- The UI and user guides must state that demo reports are fictional.
- Tests must prove that new vault creation seeds the reports and lock/reopen
  does not seed them again.
- The repository fixture and runtime demo set remain separate data sets.
