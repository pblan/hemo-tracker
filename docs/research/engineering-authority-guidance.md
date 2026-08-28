# Engineering authority guidance

Date: 2026-08-28

Status: guidance for V1 development and agent work

## Purpose

This note defines a source hierarchy and engineering rules for Hemo Tracker.
It applies to the data model, local architecture, user interface, documentation, and agent instructions.
It does not replace an accepted ADR.

## Source hierarchy

Use the source that owns a fact.
Use sources in this order:

1. Use a standard or protocol specification for normative behavior.
2. Use the official framework or library documentation for product behavior.
3. Use the operating-system guidance for platform behavior.
4. Use an original pattern author for design guidance.
5. Use a local benchmark or test for a Hemo Tracker claim.

Do not present a design pattern as a requirement.
Do not copy an example architecture without a current product need.
Record a difficult-to-reverse decision in an ADR.
Keep each ADR short and immutable.
Add a new ADR when a decision changes.
This structure follows Martin Fowler's guidance for architecture decision records.
See [Martin Fowler, Architecture Decision Record](https://martinfowler.com/bliki/ArchitectureDecisionRecord.html).

## Stable engineering principles

### Keep one clear model in the local context

A bounded context contains one internally consistent model and one shared language.
Hemo Tracker V1 has one local laboratory-result context.
Use the terms in `CONTEXT.md` in code, issues, and documentation.
Keep future identity and synchronization terms outside this model until those features start.
See [Martin Fowler, Bounded Context](https://martinfowler.com/bliki/BoundedContext.html).

The data model must preserve the difference between a lab report, a source file, a measurement, an analyte definition, a source reference interval, and a personal target range.
Do not combine these concepts to reduce table count.
The model must support a new analyte definition without a schema change to each old lab report.

### Preserve source facts and calculate views

Store the exact laboratory text as source facts.
Store corrections as explicit provenance.
Calculate normalized values, comparison decisions, and plot series from source facts and current rules.
React also recommends that applications avoid redundant and duplicate state when they can calculate the value from existing state.
See [React, Choosing the State Structure](https://react.dev/learn/choosing-the-state-structure).

Do not use the React component tree as the owner of clinical state.
The account vault is the owner.
The interface can keep temporary form state before a save.
It must refresh its view from the confirmed domain result after a save.

### Make each user action atomic

Use one database transaction for one complete domain action when the action changes several related records.
Examples include a lab report with measurements, a correction with provenance, and an analyte-definition change with its range data.
SQLite states that a transaction applies all changes or no changes.
SQLite also permits one writer at a time.
Keep write transactions short and handle a busy result as an expected failure.
See [SQLite, Transactions](https://www.sqlite.org/lang_transaction.html) and [SQLite, Atomic Commit](https://www.sqlite.org/atomiccommit.html).

Database atomicity does not make a database and a source file one atomic unit.
Use a staged file, a database transaction, explicit cleanup, and recovery tests for actions that change both.
Do not claim crash safety without a failure test for each transition.

### Keep encryption behind the native boundary

SQLCipher encrypts database pages and journal pages.
It can accept raw key data.
This supports the current account-key design.
It does not protect plaintext that the application sends to logs, exports, screenshots, the clipboard, or the WebView.
See [Zetetic, SQLCipher design](https://www.zetetic.net/sqlcipher/design/).

Tauri capabilities assign permissions to windows and WebViews.
Permissions merge when a window belongs to more than one capability.
Use one small capability set for the main window.
Grant only the commands and paths that a current workflow needs.
See [Tauri, Capabilities](https://v2.tauri.app/security/capabilities/).

The WebView must receive typed domain data.
It must not receive encryption keys, SQL, vault paths, source-file paths, or cipher settings.
Treat this rule as an architecture fitness check.

### Make accessibility a release property

WCAG 2.2 requires a programmatic name, role, and value for controls.
It also requires programmatic status messages and keyboard operation.
It defines a minimum pointer target size of 24 by 24 CSS pixels, with stated exceptions.
See [W3C, WCAG 2.2](https://www.w3.org/TR/WCAG22/) and [W3C, WCAG 2.2 changes](https://www.w3.org/WAI/standards-guidelines/wcag/new-in-22/).

For each form, use visible labels, field instructions, inline errors, and an error summary when several fields fail.
Announce save, restore, export, and failure status without an unnecessary focus change.
See [W3C, User Notification](https://www.w3.org/WAI/tutorials/forms/notifications/).

Use one column for the main data-entry sequence.
Use more columns only when the window size and field relationship make the order clear.
Keep the keyboard order equal to the visual order.
Microsoft gives the same guidance for desktop forms and requires clear labels for controls and control groups.
See [Microsoft, Forms for Windows apps](https://learn.microsoft.com/en-us/windows/apps/design/controls/forms).

Use text or a text-and-icon label when an icon can be unclear.
Do not make a destructive action the primary action.
See [Apple, Buttons](https://developer.apple.com/design/human-interface-guidelines/buttons).

Each plot must have an equivalent data table.
Do not use color, hover, or pointer input as the only way to read a value or flag.
Test the packaged application with keyboard input and a platform screen reader.
An automated DOM check is useful, but it is not proof of desktop accessibility.

### Keep future work out of the V1 architecture

Fowler states that a future abstraction has a cost when the current product does not use it.
He also states that this rule does not prohibit refactoring or tests that keep the code easy to change.
See [Martin Fowler, Yagni](https://martinfowler.com/bliki/Yagni.html).

Do not add server repositories, synchronization messages, event buses, or conflict frameworks to V1.
Keep opaque identifiers and version fields because accepted ADRs require them.
Add the remaining synchronization design when the Post-V1 work starts.

## Optional patterns

### Rich domain model

A rich domain model can help when rules become complex and interact.
A transaction script can stay clear for a simple workflow.
Fowler states that the extra domain-model cost pays off when the application has much domain logic.
See [Martin Fowler, Domain Logic and SQL](https://martinfowler.com/articles/dblogic.html).

For V1, place normalization, comparability, correction, and report-state rules behind named domain operations.
Do not require a class for each noun.
Deepen a module only when it removes repeated rules or prevents an invalid state.

### Event sourcing and CQRS

Do not use event sourcing in V1.
The product needs source-fact preservation and correction provenance.
It does not need an event log as the system of record.
Fowler describes event sourcing as a design in which the event log can rebuild application state.
That design adds event evolution, replay, snapshot, and audit responsibilities.
See [Martin Fowler, Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html).

Do not add CQRS for plot reads.
Use SQL queries and pure view builders first.
Add a stored read model only after a measured query or render limit requires it.

### Local-first synchronization

Local-first is a product property in V1 because all core work is local.
It is not proof that a future multi-device merge design is correct.
Keep the synchronization design deferred under ADR 0005.
Require a separate ADR and conflict prototype before implementation.

## Required engineering evidence

Use these checks for each relevant change:

- Add a domain test for each rule that can change a clinical value or comparison.
- Add a transaction failure test for each multi-record write.
- Add a recovery test for each database and file transition.
- Add a boundary test that rejects data which must not enter the WebView.
- Add keyboard, accessible-name, error, and status checks for each workflow.
- Run a packaged macOS and Windows check for platform behavior.
- Use a benchmark before a performance claim becomes an ADR constraint.
- Update an ADR only through a new superseding ADR.

## Recommended agent rules

Add these short rules to the agent instructions when the project next revises them:

- Cite the source that owns a technical claim.
- Treat patterns as options, not requirements.
- Do not add a future abstraction without a current acceptance criterion.
- Keep source facts separate from derived views.
- Put one complete domain action in one transaction when possible.
- Do not expose keys, SQL, or local paths to the WebView.
- Add failure evidence for a crash-safety claim.
- Test user workflows with keyboard input and platform assistive technology.
- Create a superseding ADR. Do not rewrite an accepted decision.

## Limits

These sources provide engineering guidance.
They do not prove that Hemo Tracker is medically safe, accessible, secure, or crash-safe.
Tests and specialist review must provide product evidence.
The Apple and Microsoft guidance describes native platform behavior.
Hemo Tracker uses a WebView, so apply the interaction principle and verify the result in the packaged application.
