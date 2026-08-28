---
status: accepted
---

# Reset the local vault to a fresh demo set

## Context

Reviewers need a reliable way to return to the populated first-run state. A
reset can also remove real laboratory data, so the workflow must be explicit and
must not weaken the local encryption boundary.

## Decision

Add **Reset vault to demo data** to the unlocked local data controls. Require the
current passphrase, the exact confirmation text `RESET DEMO VAULT`, and a final
warning. V1 does not require a backup before reset because the user explicitly
chooses the irreversible action. The native client creates a fresh vault in a
staging directory with a new account key set and recovery key. It validates the
fresh vault, then atomically replaces the active directory. If replacement or
reopen fails, the prior vault remains usable.

Use the exact small fictional data set from new-vault creation. Preserve
non-vault application preferences. Show the new recovery key once and require
the user to acknowledge that it was stored.

## Alternatives considered

### Require a backup before every reset

Rejected for V1. It makes returning to demo data unnecessarily difficult and the
user has explicitly accepted the irreversible warning. The UI still offers the
encrypted backup action immediately beside reset.

### Clear records in the existing vault

Rejected. A fresh directory and fresh keys provide a true new-vault state and
avoid retaining old key material or stale identifiers.

## Consequences

- Reset is useful for local demos and UI inspection.
- The action can permanently remove personal data without a backup.
- Fresh recovery keys must be stored after every reset.
- The reset path shares the atomic replacement and validation rules with restore.
