# Product plan

## Purpose

Hemo Tracker helps one person keep and inspect longitudinal laboratory results. The first release supports multiple private accounts. Each account tracks results for one person.

The application stores source files and manual entries. It does not extract values automatically. It does not give medical advice.

## Product principles

- Preserve every source fact.
- Keep source facts separate from derived data.
- Let the user add analyte definitions at any time.
- Let the user link missing measurements to old lab reports.
- Recompute derived views from the latest analyte definitions.
- Block unsafe unit conversion and unsafe series joins.
- Keep clinical content encrypted from other users and server administrators.
- Use simple interfaces and maintained standard tools.
- Follow ASD-STE100 in maintained documentation.

## Version 1 scope

### Accounts and devices

- Sign in with Google OAuth.
- Permit only Google emails on a server whitelist.
- Keep Google authentication separate from data encryption.
- Create an encryption passphrase and a user-held recovery key.
- Approve trusted macOS and Windows devices.
- Access an existing local vault while the server is offline.
- Synchronize ciphertext when the server is available.

### Lab reports

- Create a draft lab report.
- Record collection date and time.
- Record report date, laboratory, ordering clinician, fasting state, notes, and tags.
- Add multiple source files.
- Assign the role `primary`, `supplement`, or `correction` to each source file.
- Accept PDF, PNG, JPEG, and HEIC previews.
- Store other file types as opaque source files.
- Mark a report as complete or archived.
- Add measurements to a complete report later.

### Manual measurement entry

- Enter measurements in a table.
- Search the analyte catalog.
- Create an analyte definition during entry.
- Enter a numeric, ordinal, or text source value.
- Preserve source value text and source unit text.
- Enter the source reference interval and source flag.
- Validate each row when the user enters it.
- Accept German and English number and date formats.
- Require confirmation for ambiguous input.
- Allow more than one measurement for one analyte identity in one lab report.

### Analyte catalog

- Seed a representative catalog from official LOINC panel definitions.
- Include common blood count, metabolic, lipid, iron, thyroid, and inflammation analytes.
- Let the user add any analyte definition.
- Store aliases and an optional LOINC code.
- Define identity with component, property, specimen, scale, and relevant method.
- Add dated personal target ranges with an optional note.
- Preview and confirm bulk relinking of measurements.
- Block relinking when identity or unit rules are unsafe.

### Unit normalization

- Use UCUM codes for machine-readable units.
- Convert commensurable units automatically.
- Use reviewed analyte-specific rules for mass and substance concentration conversions.
- Do not convert arbitrary units, titers, ambiguous fractions, or incompatible identities.
- Keep normalized values as derived data.
- Never replace the source value or source unit.

### Views

The application has five main views.

1. **Overview** shows recent reports, pinned analytes, and data-quality warnings.
2. **Reports** shows report metadata, source files, and the manual entry table.
3. **Trends** shows one detailed analyte plot or up to six aligned plots.
4. **Analytes** shows definitions, aliases, conversion rules, and personal target ranges.
5. **Settings** shows account, devices, recovery, synchronization, export, archive, and deletion controls.

Each overview card shows the latest source value, source unit, collection date, source flag, and a small trend. The user selects the pinned analytes.

### Plots

- Use the true collection time on the horizontal axis.
- Show one focused plot at full detail.
- Show no more than six aligned small plots for comparison.
- Show source reference intervals and personal target ranges with different labels and patterns.
- Keep the source flag visible.
- Mark comparability boundaries.
- Link each point to its lab report.
- Provide shared zoom and time-range selection.
- Support pointer and keyboard inspection.
- Provide the same values in an accessible table.
- Do not use color as the only status signal.
- Keep correlation and distribution plots outside version 1.

### Corrections and conflicts

- Replace a corrected manual value without revision history.
- Store `updatedAt` and `updatedBy` metadata.
- Keep original source files immutable.
- Keep a correction source file beside the prior files.
- Detect a synchronization conflict at the entity level.
- Keep both encrypted conflict versions.
- Ask the user to select one complete version.
- Do not merge conflict fields automatically.

### Export and deletion

- Create an encrypted full backup.
- Create a decrypted ZIP with source files, JSON, and CSV after an explicit warning.
- Archive lab reports in the normal interface.
- Require confirmation before permanent deletion.
- Mark an account for deletion before the server deletes its ciphertext.
- Prompt the user to export data before account deletion.

## Excluded from version 1

- OCR and automatic result extraction.
- Diagnosis, health scores, and treatment advice.
- Automatic urgency rules.
- Clinician and patient workflows.
- Sharing between accounts.
- Wearable data.
- Reminders and notifications that contain clinical data.
- Free-form chart annotations.
- Server-side clinical search, plots, or unit conversion.
- Administrator recovery of account data.

## Security limits

The system protects clinical content against other users, storage operators, backup disclosure, a server administrator, and a compromised server.

The system does not protect against these threats:

- Malware or an attacker on a trusted device.
- An attacker who can replace the signed client or trusted update source.
- Screen capture while the application shows data.
- Loss of all trusted devices, the passphrase, and the recovery key.
- Metadata disclosure. The server can see account identity, ciphertext size, device count, storage use, and connection time.

Google can see that the user signs in to Hemo Tracker. Google does not receive the encryption passphrase or account data key.

## Quality requirements

- Support macOS and Windows in version 1.
- Meet WCAG 2.2 AA where it applies.
- Use no third-party analytics in the clinical client.
- Keep clinical content out of logs, crash reports, notifications, and previews.
- Use signed application releases and signed updates.
- Keep release signing keys outside the self-hosted data server.
- Obtain a specialist security review before the application stores real medical data.

## Plot benchmark

Build the same realistic fixture for uPlot and Apache ECharts. Keep Recharts as an implementation and accessibility baseline.

The stress fixture contains:

- 1,000 lab reports.
- 100,000 measurements.
- 250 analyte definitions.
- 20 visible plots.
- Ten years of irregular collection dates.
- Changing source reference intervals.
- Unit, method, specimen, and laboratory boundaries.

The target is an initial plot render below 500 ms. The target for pointer interaction is below 50 ms on a representative device. Record bundle size, memory use, keyboard access, export quality, and theme behavior.

Select the plot engine in an ADR after the benchmark.

## Delivery plan

### Phase 0: Prove security and plot choices

- Prove the account key lifecycle.
- Prove recovery and passphrase change.
- Prove native credential storage on macOS and Windows.
- Prove SQLCipher builds, migrations, temporary-file behavior, backup, and restore.
- Prove streaming source-file encryption.
- Prove Google OAuth with PKCE and a loopback callback.
- Prove signed macOS and Windows builds.
- Prove signed Tauri updates.
- Benchmark uPlot and Apache ECharts.
- Get an independent review of the threat model and key design.

Exit condition: Accepted ADRs name the security formats, storage tooling, OAuth flow, update process, and plot engine.

### Phase 1: Build the local vault

- Create the Tauri application shell.
- Create the narrow Rust vault module.
- Add the encrypted local database.
- Add analyte definitions and the seed catalog.
- Add lab reports, source files, and manual measurements.
- Add archive, correction, export, and local backup.

Exit condition: One trusted device can manage a complete account vault while offline.

### Phase 2: Build trends

- Add safe normalization.
- Add comparability checks.
- Add the overview, trend plots, range bands, target bands, and accessible tables.
- Add bulk analyte relinking with a preview.

Exit condition: New analyte definitions and normalization rules update all derived views without source-data changes.

### Phase 3: Build encrypted synchronization

- Add Google sign-in and the server whitelist.
- Add device approval and wrapped-key transfer.
- Add opaque object synchronization.
- Add simple conflict selection.
- Add encrypted server backups and restore tests.
- Add account administration and delayed deletion.

Exit condition: Two trusted devices can synchronize through a server that cannot read clinical content.

### Phase 4: Harden and release

- Complete accessibility checks.
- Complete privacy and security tests.
- Complete macOS and Windows signing.
- Complete the signed update flow.
- Complete deployment, backup, recovery, and threat-model guides.
- Complete an external security review.

Exit condition: The release gates pass on macOS and Windows.
