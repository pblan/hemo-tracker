# V1 fixture data

This directory contains deterministic fictional data for local development and
tests.

The fixture has at least 25 analytes and 12 reports. It includes CBC, metabolic,
liver, thyroid, iron, inflammation, lipid, and electrolyte analytes. It includes
different units, missing and comparator values, source flags, date ranges,
source-file roles, corrections, sparse series, and report states.

The JSON files keep source facts separate from derived expectations:

- `analytes.json` and `reports.json` are source facts.
- `expected-normalization.json` and `expected-trends.json` are test
  expectations.

Run `bun run mock-data:generate` to regenerate the fixture. Run
`bun run mock-data:verify` to check IDs, references, roles, and expected counts.
Generation uses fixed values and is idempotent.

All names, reports, values, and files are fictional. Do not add real health data
to this directory. Do not include this fixture as a user account or as a release
vault.
