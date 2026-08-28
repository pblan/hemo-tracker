# Software engineering and user experience research

Status: accepted guidance for V1 planning

## Purpose

This note records guidance for the local-first user interface and data model. It supports issues that must finish before server synchronization.

## Findings

### Keep one clear domain model per context

Martin Fowler describes a bounded context as a boundary for one consistent model. The model must use one shared language inside that boundary. Hemo Tracker therefore keeps the local account vault model separate from future identity and synchronization models. The terms in `CONTEXT.md` remain the source for names.

Source: [Martin Fowler, Bounded Context](https://martinfowler.com/bliki/BoundedContext.html).

### Preserve facts and derive views

Fowler describes event sourcing as a way to record immutable events and rebuild current state. Hemo Tracker does not need full event sourcing for V1. It should still preserve source laboratory text and record user changes as explicit domain actions. A correction must not overwrite the source value. A derived normalized value must remain recalculable.

Source: [Martin Fowler, Event Sourcing](https://www.martinfowler.com/eaaDev/EventSourcing.html).

### Model laboratory results with context

HL7 FHIR models an Observation with an effective date, a value, an interpretation, a reference range, and a reason when a value is absent. It also separates the time when an observation occurred from the time when it became available. Hemo Tracker should keep collection time, source value, source flag, source reference interval, and missing-value reason as separate fields. It should not treat a personal target range as a laboratory reference interval.

Source: [HL7 FHIR R5 Observation](https://hl7.org/fhir/R5/observation-definitions.html).

### Make forms predictable

W3C requires labels or instructions for inputs. W3C also requires text that identifies an input error and explains the error. Labels should stay close to their controls. A long form should use a clear linear order. Hemo Tracker forms should therefore use visible labels, short instructions, explicit required markers, inline errors, a summary for multiple errors, and keyboard-safe focus movement.

Sources: [W3C, Labels or Instructions](https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html), [W3C, Error Identification](https://www.w3.org/WAI/WCAG22/Understanding/error-identification.html), and [W3C, Labeling Controls](https://www.w3.org/WAI/tutorials/forms/labels/).

### Keep plot and table views linked

The plot is a fast visual summary. It is not the only access path to a measurement. Each plot must have a linked table with the same data, units, flags, intervals, and comparability boundaries. The table must support keyboard access and screen readers. Plot interactions must not hide a source fact.

This rule is consistent with ADR 0004. The plot adapter owns visual behavior. Feature code supplies domain data.

## V1 data and UX rules

1. Keep source facts immutable after entry.
2. Add a correction as a new explicit action. Keep the original source value.
3. Show the collection time and the entry time as different concepts when both exist.
4. Show source reference intervals beside source values.
5. Show personal target ranges as guidance. Do not call them normal ranges.
6. Preserve missing and exceptional values with a reason.
7. Mark comparability boundaries when identity, unit, specimen, method, or laboratory changes.
8. Use one vertical form order for keyboard and pointer users.
9. Validate rows before submission. Identify the row and field in text.
10. Show save progress and success feedback. Never imply success before the vault confirms it.
11. Keep destructive actions recoverable where possible. Require confirmation for permanent deletion.
12. Keep all clinical text inside the unlocked account vault boundary.

## Issues before server synchronization

Create these issues before work on server synchronization. Each issue should be assigned before implementation and should include a testable acceptance checklist.

### Local report history and search

Provide a history view for draft, complete, and archived lab reports. Add date, laboratory, status, and analyte filters. Keep the list usable with keyboard navigation. Show an empty state and a clear no-results state. Do not expose source paths or keys.

### Measurement form usability and validation

Replace ad hoc row errors with field-level validation and a form error summary. Add visible labels, required-field text, input instructions, `aria-invalid`, and `aria-describedby` links. Preserve entered rows after a failed save. Test keyboard order and screen-reader names.

### Correction and provenance workflow

Add an explicit correction action for a measurement. Preserve the original source fields. Record who made the correction and when. Show the correction relationship in the detail view. Recalculate normalized views from the current analyte definition without changing source facts.

### Trend plot and accessible data table

Build the uPlot adapter for one analyte series and its linked table. Display irregular dates, source intervals, personal target ranges, flags, missing values, and comparability boundaries. Add a text alternative and export-safe table. Test with keyboard input and representative data.

### Backup, export, and destructive-action UX

Add clear flows for encrypted backup, local export, archive, and permanent deletion. Explain the destination, data scope, and recovery limits before the action. Show progress, success, and actionable failure text. Require confirmation for permanent deletion. Test cancellation and partial failure without data loss.

## Research limits

These sources guide design. They do not provide medical advice or define the complete Hemo Tracker data model. HL7 FHIR is a useful interoperability reference. Hemo Tracker does not claim FHIR conformance in V1.
