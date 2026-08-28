# Desktop UX and chart guidance

Date: 2026-08-28

Status: V1 design guidance

## Scope

This note records source-based guidance for menus, status feedback, and blood-result charts.
It does not provide medical advice.

## Source findings

### Menus and navigation

Apple describes the macOS menu bar as a place for app commands and keyboard shortcuts.
It recommends standard shortcuts and custom shortcuts only for frequent app actions.
The WAI-ARIA Authoring Practices Guide defines menu keyboard behavior.
When a menu opens, focus moves to its first item. Arrow keys move within the menu. `Escape` closes the menu. `Tab` leaves the menu instead of moving between menu items.
Microsoft places frequent commands in a command bar and uses a menu for a list of commands or options.

Sources: [Apple, Customizing menus](https://developer.apple.com/tutorials/app-dev-training/customizing-menus-with-commands-and-shortcuts), [Apple, Keyboards](https://developer.apple.com/design/human-interface-guidelines/keyboards/), [WAI-ARIA APG, Menu and Menubar](https://www.w3.org/WAI/ARIA/apg/patterns/menubar/), and [Microsoft, Menu flyout and menu bar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus).

### Status feedback and toasts

Apple says that alerts are for critical information and actions that need immediate attention.
It says to avoid alerts for information that is not actionable.
Microsoft describes an InfoBar as a visible, non-modal message for a changed app state.
It recommends a dialog when the app needs confirmation or blocks the workflow.
Material describes a snackbar as short feedback for an operation.
The snackbar must not be the only way to access a core workflow.
It should have a useful action when the user can undo or inspect the result.

Sources: [Apple, Alerts](https://developer.apple.com/design/human-interface-guidelines/alerts), [Microsoft, InfoBar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/infobar), [Microsoft, Dialogs and flyouts](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/), and [Material, Snackbars](https://m2.material.io/components/snackbars-toasts.html).

### Accessible charts

W3C classifies charts as complex images.
It requires a short identification and a longer text alternative for the information that the chart conveys.
W3C also recommends a data table or another structured description for complex information.
The WAI-ARIA APG defines an alert as a brief important message that does not interrupt the task.
Use text and semantics with color. Do not use color as the only indicator.

Sources: [W3C, Complex images](https://www.w3.org/WAI/tutorials/images/complex/), [W3C, Alternative content](https://www.w3.org/WAI/WCAG2/supplemental/patterns/o7p02-alternative-content/), [W3C, Easy checks](https://www.w3.org/WAI/test-evaluate/preliminary/), and [WAI-ARIA APG, Patterns](https://www.w3.org/WAI/ARIA/apg/patterns/).

### uPlot behavior

uPlot provides a legend, axes, scales, bands, cursor interaction, and synchronized cursors.
Its public type definitions expose these as separate chart options.
The project documentation reports a small bundle and fast time-series rendering.
The chart library does not provide the complete accessible explanation for a clinical result.
Hemo Tracker must provide the title, unit, range meaning, source status, and table outside the canvas.

Sources: [uPlot API types](https://github.com/leeoniya/uPlot/blob/master/dist/uPlot.d.ts) and [uPlot README](https://github.com/leeoniya/uPlot/blob/master/README.md).

## Hemo Tracker design

### Application shell

Use a persistent left navigation rail or sidebar for the main destinations:

- Overview
- Reports
- Trends
- Analytes
- Backups and exports
- Settings

Keep the primary action visible in the current page. Use a menu for secondary actions such as reset demo data, export, and help.
On macOS, expose the same commands in the native app menu. On Windows, expose the same commands in the app menu or command bar.
Do not hide the only path to a core workflow in a menu.

Use these keyboard commands only when implemented in the visible menu:

- New report: `Command-N` on macOS and `Control-N` on Windows
- Save: `Command-S` on macOS and `Control-S` on Windows
- Search: `Command-F` on macOS and `Control-F` on Windows
- Settings: `Command-,` on macOS and `Control-,` on Windows

### Feedback policy

Use one feedback surface for each class of event:

| Event | Surface | Required content |
| --- | --- | --- |
| Saved, attached, exported, or restored | Toast/snackbar | Completed action and destination or count |
| Vault is locked or data is unavailable | Persistent inline banner | State and next action |
| Validation failure | Inline field message and summary | Field, problem, and correction |
| Destructive reset or permanent deletion | Modal confirmation | Scope, consequence, and cancel action |
| Unexpected failure | Inline error or dialog | What failed and safe recovery action |

Every toast must also have a durable result in the page, such as the saved report in the report list.
Announce status text through an `aria-live` region without moving focus.
Use `role="alert"` only for urgent errors that need immediate attention.
Do not stack many toasts. Keep an undo action where the operation supports undo.

### Trend plot anatomy

Every trend view should contain these elements in this order:

1. A heading with the analyte name.
2. A subtitle with the displayed unit and collection date span.
3. A visible legend with series names and visual marks.
4. A short explanation of the laboratory interval and personal target range.
5. The plot with explicit axis labels and tick values.
6. Data-quality notes for missing, excluded, or non-comparable points.
7. A linked table with date, source value, source unit, normalized value, flag, and ranges.

Use distinct line styles or patterns as well as color:

- Measured value: solid line and point marker.
- Laboratory interval: light filled band with a text label.
- Personal target: separate band or dashed boundary with a text label.
- Missing value: gap in the line and a table row with the reason.
- Non-comparable value: point marker with a text label and a boundary note.

Do not describe a result as healthy, safe, or diagnostic.
Use factual labels such as “below source interval”, “above source interval”, or “not comparable”.
Show the source interval that applied to each measurement. Do not imply that a personal target is a laboratory interval.

### Interactions

Support keyboard access to analyte selection, date range, unit display, table, and export.
Hover is supplementary. A focused point must expose its date, value, unit, flag, and source range in text.
Provide a reset-zoom action and preserve a stable default view.
Do not make a dense chart depend on a hover tooltip for interpretation.

## Acceptance checks

- A user can reach all destinations and actions with keyboard input.
- Each menu has visible labels, correct focus movement, and an Escape path.
- Save, restore, export, and reset give status feedback without an unnecessary focus change.
- A validation failure has inline text and a summary when several fields fail.
- A chart has a visible legend and a linked accessible table.
- The chart remains understandable when color is unavailable.
- Missing and non-comparable points remain visible in the table.
- A screenshot review checks the default mock data for a stable trend, a deviation, a recovery, a gap, and a range change.

These checks provide product evidence. They do not claim formal WCAG conformance or medical safety.
