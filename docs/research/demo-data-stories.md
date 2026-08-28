# Fictional demo-data stories

Status: accepted research input for V1 fixture planning

Date: 2026-08-28

## Scope

This note defines safe stories for fictional laboratory data. It supports UI
testing and plot interpretation. It is not medical advice and does not define
diagnostic ranges.

## Recommended stories

1. Stable baseline across 8 to 12 irregularly dated reports.
2. A short-lived fictional deviation followed by recovery, without naming a
   diagnosis or cause.
3. Glucose and creatinine values in alternate units with reviewed normalized
   values. Source values remain unchanged.
4. A comparability boundary when specimen, method, property, or laboratory
   identity changes. Do not join points unless the analyte identity is proven.
5. A changed laboratory interval and a separate dated personal target range.
6. Missing, textual, comparator, and arbitrary-unit results that remain out of
   numeric trends unless an explicit rule supports them.
7. A corrected result with correction metadata and visible provenance.
8. Complete, draft, and archived reports, including PDF, image, and correction
   source files on one report.
9. Cross-panel coherence. CBC values should move together. Calculated eGFR
   must not look like an independently measured value.
10. Sparse and retroactively linked series, including an analyte added after
    earlier reports.

## Coverage target

The comprehensive repository fixture should contain at least 25 analytes and
12 reports across 18 to 24 months. It should cover CBC, metabolic, liver,
thyroid, iron, inflammation, lipids, electrolytes, and one qualitative or
custom analyte. Each analyte should have at least three points unless the sparse
series is intentional. Expected metadata must assert story coverage, not only
file counts.

The runtime seed can remain smaller for fast first-run startup only if the UI
offers a clear path to inspect the comprehensive fictional fixture. The seed
must still contain a readable trend, a range example, a missing or excluded
value, and a source report link.

## Correctness risks found

- Repeating a source flag on every report while the value stays constant does
  not tell an interpretable story. Flags must follow the fictional deviation.
- Repeating constant values weakens trend interpretation.
- A generic interval such as `Fictional interval` does not exercise interval
  rendering.
- A missing value must not have a parsed numeric value.
- Unit conversion requires the exact reviewed analyte identity and compatible
  UCUM dimensions. Conversion alone is not enough.

## Authority references

- [LOINC CBC panel 58410-2](https://loinc.org/58410-2)
- [LOINC comprehensive metabolic panel 24323-8](https://loinc.org/24323-8)
- [FHIR R5 Observation](https://hl7.org/fhir/R5/observation.html)
- [UCUM](https://ucum.org/)

These sources describe interoperability concepts and observation structures.
They do not make the fictional values clinically valid for a person.
