# Sidebar navigation guidance

Date: 2026-08-28

Status: design guidance for the local desktop application

## Scope

This note defines the sidebar and page-shell guidance for Hemo Tracker.
It covers desktop navigation, page grouping, responsive behavior, and settings.
It does not define medical interpretation.

## Source findings

### Use a persistent sidebar for peer destinations

Apple describes a sidebar as a leading navigation surface for sections of an app.
It recommends a broad and mostly flat hierarchy.
It recommends no more than two levels.
It also recommends that users can hide and show the sidebar.
See [Apple Human Interface Guidelines: Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars).

Microsoft describes NavigationView as a top-level navigation control.
It supports a left pane and adapts between expanded, compact, and minimal modes.
It supports grouped items, a footer, and a settings item.
Microsoft recommends a shallow hierarchy.
See [Microsoft NavigationView](https://learn.microsoft.com/en-us/windows/apps/design/controls/navigationview) and [Microsoft navigation basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/navigation-basics).

Material recommends a navigation drawer when an application has five or more top-level destinations.
It orders destinations by importance and groups related destinations.
It uses a permanent drawer on large screens when users switch destinations often.
It uses a dismissible or modal drawer when content needs more space.
See [Material navigation drawer](https://m2.material.io/components/navigation-drawer).

### Keep labels clear and the hierarchy shallow

Microsoft identifies consistency, simplicity, and clarity as the main navigation principles.
It recommends familiar controls and clear destination labels.
See [Microsoft navigation basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/navigation-basics).

Carbon uses a fixed left panel for secondary navigation.
It allows links and submenus.
It does not support a third navigation tier.
It recommends tabs inside the page when more content is needed below a submenu.
See [Carbon UI shell left panel](https://carbondesignsystem.com/components/UI-shell-left-panel/usage/).

GitHub Primer describes a NavList as a vertical list of links for the current context.
It uses a selected link with `aria-current="page"`.
It recommends a heading outline for settings-style navigation.
See [GitHub Primer NavList](https://www.primer.style/product/components/nav-list/).

### Separate navigation from actions

A navigation item changes the current page.
A command performs an action on the current page or application.
Do not put reset, export, or delete beside page links without a clear group boundary.
Use a page action, overflow menu, or confirmation dialog for such commands.

Microsoft places settings at the end of a NavigationView list.
It pins settings to the bottom of the pane.
See [Microsoft app settings guidelines](https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings).

WAI-ARIA defines `nav` as a navigation landmark for a group of navigation links.
It recommends a unique label when a page has more than one navigation landmark.
It defines a menu as a command widget with different keyboard behavior from ordinary links.
Do not give a persistent page-link sidebar `role="menu"` unless it implements the complete menu pattern.
See [WAI-ARIA navigation landmark](https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/examples/navigation.html) and [WAI-ARIA menu and menubar](https://www.w3.org/WAI/ARIA/apg/patterns/menubar/).

### Adapt without losing orientation

Microsoft describes compact and minimal left navigation modes for narrow windows.
The selected destination and page header must remain visible in each mode.
See [Microsoft NavigationView display modes](https://learn.microsoft.com/en-us/windows/apps/design/controls/navigationview).

Material uses a permanent drawer for large desktop layouts and a compact rail or modal drawer at smaller widths.
The drawer can scroll independently when its destinations exceed the window height.
See [Material navigation drawer behavior](https://m2.material.io/components/navigation-drawer).

Apple recommends that a sidebar is not hidden by default because discoverability matters.
See [Apple Human Interface Guidelines: Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars).

WAI-ARIA landmarks help assistive technology users move between major regions.
The page should contain a labelled `nav` landmark and one `main` landmark.
Do not create many landmarks for small visual sections.
See [WAI-ARIA landmarks](https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/).

## Hemo Tracker recommendations

### Shell

Use a persistent left sidebar on macOS and Windows desktop.
Use a width near 240 pixels when expanded.
Use a compact icon rail only when the user chooses it or the window is narrow.
Keep the sidebar visible on first launch.
Persist the user's expanded or compact choice locally.

Use this top-level order:

1. Overview
2. Trends
3. Reports
4. Analytes
5. Backups and exports

Put Settings at the bottom of the sidebar.
Put the vault state near the product name.
Show the current page with text, icon, background, and a visible focus state.
Use one `nav` landmark with the label `Primary`.
Use one `main` landmark for the page content.

Do not add nested sidebar pages for each analyte.
Use the Analytes page for analyte definitions, units, laboratory ranges, and personal target ranges.
Use the Reports page for report facts and measurements.
Keep these workflows separate.
Do not allow a report-entry form to create or edit an analyte definition.

### Page layout

Use a stable application shell.
Keep the sidebar and page header in fixed positions.
Let only the page content scroll.
Use a constrained content width for forms and settings.
Allow trend tables and plots to use the available width.
Use a page title, a short purpose statement, and one primary action.
Keep secondary actions in a nearby action group or overflow menu.

Use section headings inside pages.
Do not use a third sidebar level.
Use tabs inside a page only when the tabs share the same object and task.
Use breadcrumbs only for a real detail path, such as Reports > Report date.

### Sidebar interaction

Use normal links or buttons with visible text.
Use `aria-current="page"` for the selected page.
Do not use `role="menu"` for the persistent sidebar.
Make the complete sidebar reachable with the keyboard.
Keep the focus indicator visible.
Support the platform menu button to collapse or expand the sidebar.
When the sidebar collapses, keep an accessible name and tooltip for every icon.
Do not make an icon the only label.

At narrow widths, open the sidebar as a modal drawer.
Trap focus inside the open drawer.
Close it after navigation, Escape, or activation of the scrim.
Return focus to the menu button after close.
Do not use this modal behavior on the normal desktop layout.

### Visual system

Use a quiet surface for the sidebar.
Use one accent color for the selected destination.
Use a thin divider between primary pages and Settings.
Do not use a different color for every page.
Use one consistent icon set.
Keep icon meaning stable across the application.
Use sufficient contrast for inactive text, selected text, and focus rings.

Use cards for grouped settings and summaries.
Do not put every control in a card.
Use whitespace and section headings to create grouping.
Use tables for dense measurement data.
Use plots for trends and relationships.
Keep page actions close to the content that they change.

### Navigation test cases

- The first launch shows the sidebar and identifies the current page.
- A user can reach every page with keyboard input.
- The selected page has `aria-current="page"`.
- The sidebar has one labelled navigation landmark.
- Settings is last and stays at the bottom of the sidebar.
- The sidebar does not contain more than two hierarchy levels.
- A collapsed icon has an accessible name and a tooltip.
- A narrow-window drawer closes on Escape and restores focus.
- Navigating to a page does not duplicate history entries.
- A page action does not look like a navigation destination.
- An analyte definition can be changed without creating a report.
- Report entry cannot create or edit an analyte definition.
- Plot pages keep their legend, table, and data-quality notes after navigation.

These checks provide product evidence.
They do not claim formal WCAG conformance.

