# Local application UI and UX practices

Date: 2026-08-28

## Scope

This note records primary-source guidance for the Hemo Tracker desktop user interface.
It focuses on manual laboratory data entry, local file selection, and result review.
It does not give medical advice.

## Source evidence

### Forms need visible labels and useful errors

WCAG 2.2 requires labels or instructions when a control needs user input.
It also requires text that identifies and describes an input error.
Color alone is not enough.
See [WCAG 2.2, Success Criterion 3.3.2](https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html) and [Success Criterion 3.3.1](https://www.w3.org/WAI/WCAG22/Understanding/error-identification.html).

WCAG 2.2 also recommends a programmatically identifiable input purpose when the input has a recognized purpose.
This can help users and assistive technology complete forms.
See [WCAG 2.2, Success Criterion 1.3.5](https://www.w3.org/WAI/WCAG22/Understanding/identify-input-purpose.html).

Chakra UI's `Field` component provides a label, helper text, required indicator, and error text.
Its invalid state is intended to pair with visible error text.
See [Chakra UI Field](https://chakra-ui.com/docs/components/field).

### Desktop privileges must stay narrow

Tauri capabilities grant permissions to a window or WebView.
Plugin permissions must be referenced by a capability.
Tauri recommends small, explicit permissions and scopes.
See [Tauri capabilities](https://v2.tauri.app/security/capabilities/) and [Tauri permissions](https://v2.tauri.app/security/permissions/).

The Tauri dialog plugin defines `dialog:allow-open` for the open-file command.
The application can use the native picker while keeping the selected path in Rust.
See [Tauri dialog plugin](https://v2.tauri.app/plugin/dialog/).

## Product recommendations

### Entry flow

- Use one clear page for a new lab report.
- Group fields by purpose: report details, source file, and measurement.
- Mark required fields in text.
- Keep entered values after a validation error.
- Show one inline error beside each invalid field.
- Put focus on the first invalid field after submit.
- Use a visible status message after save, attach, lock, and complete actions.
- Explain source value, normalized value, source reference interval, and personal target range in nearby helper text.
- Do not silently change source text during parsing or normalization.

### Safe and calm visual design

- Use a small set of semantic colors for success, warning, and error.
- Pair each color with text or an icon with an accessible name.
- Keep strong contrast for text, controls, focus indicators, and chart marks.
- Use a visible keyboard focus style.
- Keep primary actions stable in the same location.
- Use short action labels such as “Save draft”, “Add measurement”, and “Complete report”.
- Confirm before an action that can hide or remove user data.
- Do not use urgency language for a source flag unless the source provides that language.

### Review and plots

- Show an accessible table for every chart.
- Show the collection date, exact source value, source unit, source flag, and source reference interval.
- Keep chart axes and units explicit.
- Use actual collection time. Do not space points equally when dates differ.
- Mark a comparability boundary when the analyte identity, unit, specimen, method, or laboratory changes.
- Make chart tooltips supplementary. They must not be the only way to read a value.
- Use small multiples for several analytes. Keep each analyte on its own scale.

### Native file selection

- Keep file paths inside the Rust command boundary.
- Return only the source-file identifier and original filename to the WebView.
- Use the minimum Tauri capability required for opening a file.
- Show the selected filename before the user saves the report.
- Show a clear error when the user cancels or when encryption fails.

## Verification practice

Every UI change should include these checks when relevant:

1. Use keyboard navigation from the first control to the final action.
2. Verify labels, required state, error text, and focus after invalid submit.
3. Verify the workflow at narrow and wide window sizes.
4. Verify light and dark themes if both are supported.
5. Verify that charts have an equivalent table.
6. Verify that a cancelled file picker does not create partial report data.
7. Run the repository lint, type, unit, and native tests.

These checks are product quality rules. They do not claim formal WCAG conformance.

## Open evidence questions

- The cited sources do not define the best visual theme for personal laboratory data.
- The cited sources do not define safe clinical wording for source flags.
- The application still needs a packaged macOS and Windows keyboard and screen-reader review.
