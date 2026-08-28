# Mock data

Hemo Tracker uses small deterministic mock data for tests, screenshots, and local exploration.

The V1 fixture uses at least 25 analytes and 12 irregularly dated reports. It includes CBC, metabolic, liver, thyroid, iron, inflammation, lipid, and electrolyte analytes. It includes draft, complete, and archived reports. It includes multiple source files, alternate units, missing values, comparator values, flags, corrections, sparse series, and personal target ranges.

Run these commands from the repository root:

```text
bun run mock-data:generate
bun run mock-data:verify
```

Keep source facts in `fixtures/v1/analytes.json` and `fixtures/v1/reports.json`. Keep calculated values in the expectation files. The application must calculate normalized values again. It must not trust calculated values from a fixture.

Use only fictional names and values. Never add real laboratory reports, identifiers, addresses, or health data. Release packages must not contain account vaults or generated benchmark data.

The small fixture is suitable for routine checks. Large plot benchmarks are separate work and must not slow routine CI.

New local vaults use a focused runtime demo set. The application creates six
complete fictional reports with encrypted text source files and eight
representative analytes. The runtime set demonstrates irregular dates, a
temporary flagged deviation, alternate glucose units, changed intervals, and
source-linked trends. It is smaller than the comprehensive repository fixture
so first-run startup stays quick.
