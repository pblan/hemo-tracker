# ADR-0011: Separate the analyte catalog from measurement entry

## Status

Accepted

## Date

2026-08-28

## Context

A measurement records one source fact from one lab report. An analyte definition
is reusable catalog configuration for identity, canonical units, aliases, and
personal target ranges. Combining their creation in the report form made one
workflow responsible for two different concerns. It also made it possible to
link several measurements in one report to the wrong definition.

## Decision

Manage analyte definitions only on the Analytes page. The report form can select
an existing definition for each measurement or leave that measurement unlinked.
An unlinked measurement can be safely relinked later. A measurement never
creates or edits an analyte definition.

## Consequences

- The analyte catalog is a deep module with a focused interface: create and
  review reusable definitions.
- Measurement entry remains focused on immutable source facts.
- Every measurement row owns its own optional analyte link.
- Users must create a definition before they can link a new measurement to it.
- Relinking remains the path for older or initially unlinked measurements.

## Alternatives considered

### Create definitions from the report form

Rejected. It couples catalog configuration to source-fact entry and makes the
per-measurement identity ambiguous when a report has more than one result.

### Require an analyte link for every measurement

Rejected. Manual entry must preserve unfamiliar or not-yet-configured source
results without forcing users to invent a definition during report capture.
