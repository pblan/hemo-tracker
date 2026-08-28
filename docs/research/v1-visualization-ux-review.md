# V1 visualization and demo-data UX review

Status: review baseline for issue #36

Date: 2026-08-28

## Purpose

This review checks if a new user can understand the V1 plots and the default
fictional data before importing a personal laboratory report. The review does
not provide medical advice. The values in the fixture are examples for product
behavior only.

## Data review

The repository fixture contains 14 analytes and 10 reports. The reports use
irregular collection dates. They include draft, complete, and archived states,
multiple source files, source flags, missing values, changing laboratory
intervals, corrections, personal target ranges, compatible unit variants, and
blocked normalization cases. This gives the interface enough variation to show
normal data, incomplete data, and data that must not be compared.

The runtime seed is intentionally smaller. A new vault contains three complete
fictional reports with encrypted source text and three representative
measurements per report. The runtime seed is a quickstart sample, not the full
benchmark fixture. The UI marks the records as demo data.

## Interpretation checks

| Question a new user should answer | Current evidence | Result |
| --- | --- | --- |
| What changed over time? | Irregular collection dates and a line plot with a date table | Pass |
| Which value came from the source report? | Source value, source unit, interval, and flag columns | Pass |
| Which value is derived? | Normalized value and normalized unit columns | Pass |
| Why is a point excluded? | Missing or not-evaluated table text and data-quality warnings | Pass |
| Which range is which? | Separate laboratory and personal-target labels and patterned text | Pass |
| How can the source be checked? | Open report action on each linked point | Pass |
| Does the screen give a diagnosis or health score? | No score, gauge, ranking, or advice in the overview | Pass |

## Visualization review

The plot title names the selected analyte. The accessible table remains the
semantic view because the uPlot canvas does not expose every point to a screen
reader. A nearby live status line reports the selected date, source value,
normalized value, and source flag. Dragging changes the collection-time range.
Laboratory intervals use an orange band and personal targets use a teal band;
the table and explanatory text do not depend on color alone.

The review found two follow-up improvements that are not release blockers:

1. Add a short first-run explanation that the orange band is supplied by the
   laboratory and the teal band is a personal target. The current explanation
   appears below a plot and may be missed before the first interaction.
2. Add one screenshot with a missing or excluded point visible. The current
   overview screenshot demonstrates the main populated path but not every data
   quality state.

## Safety decision

The fixture is reasonable as fictional product data because it exercises the
domain rules without claiming to represent a real person. It must remain
clearly fictional. Release notes and user guides must state that the demo data
does not support diagnosis or treatment decisions.

## Evidence

- [`fixtures/v1/analytes.json`](../../fixtures/v1/analytes.json)
- [`fixtures/v1/reports.json`](../../fixtures/v1/reports.json)
- [`docs/development/mock-data.md`](../development/mock-data.md)
- [`docs/assets/screenshots/v1/`](../assets/screenshots/v1/)
- [`apps/desktop/src/components/TrendPlot.tsx`](../../apps/desktop/src/components/TrendPlot.tsx)
- [`docs/user/README.md`](../user/README.md)
