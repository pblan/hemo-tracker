---
status: accepted
---

# Preserve source facts and derive normalized views

Laboratory labels, units, methods, specimens, and reference intervals can change. Hemo Tracker must not rewrite a source result when an analyte definition or conversion rule changes. Preserve source files and source measurement fields. Store normalized values and trend series as derived data that the trusted client can recalculate from the latest analyte definitions.

## Considered options

- A fixed analyte schema would simplify entry, but it would reject new and uncommon results.
- Replacing source values with canonical values would simplify queries, but it would remove evidence and make correction unsafe.
- Preserving source facts needs more fields, but it supports retroactive analyte definitions and verifiable conversions.

## Consequences

- Analyte identity uses component, property, specimen, scale, and a relevant method.
- The application can add or change analyte definitions without making old lab reports stale.
- UCUM conversion is automatic only for safe commensurable units.
- Reviewed analyte-specific rules control mass and substance concentration conversion.
- The application blocks ambiguous or incompatible conversions.
- Source reference intervals and personal target ranges remain separate concepts.
