# ADR-0007: Keep source facts immutable and corrections explicit

## Status

Accepted

## Context

Laboratory documents contain source facts. Users can enter a value incorrectly. A correction must not change the original document or hide the correction history.

## Decision

Store the original source file as an immutable encrypted object. Store the exact source label, value, unit, interval, and flag with each measurement. A correction updates the current measurement fields and records `updatedAt` and `updatedBy`. Derived values and normalized values are separate from source facts.

## Consequences

- The application can show the current value and its provenance.
- A source file can be checked again when a value is corrected.
- Normalization and plots can change without rewriting source facts.
- Permanent deletion must be a separate, explicit action.

## References

- [Software engineering and UX research](../research/software-engineering-ux.md)
- [Documentation policy](../documentation-policy.md)
