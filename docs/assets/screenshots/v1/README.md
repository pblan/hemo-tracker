# V1 screenshot manifest

This directory stores screenshots for the unsigned local V1 release.

## Capture procedure

1. Run `bunx playwright test tests/e2e/v1-screenshots.spec.ts` from the repository root.
2. Review the generated images for fictional data only.
3. Use the same 1280 x 900 viewport, light theme, and fixture data on macOS and Windows.
4. Capture the webview with the Playwright flow after the interface is stable.
5. Capture native file dialogs manually. Do not include real file names, paths, account names, or secrets.
6. Record the application version, platform, viewport, theme, fixture, and capture method in the image entry below.
7. Review each image for personal data and verify the adjacent procedure contains all required instructions.

## Current entries

- `desktop-locked-vault.png`: Hemo Tracker 0.1.0, browser-rendered desktop webview, 1280 x 900 viewport, light theme, fictional locked-vault fixture, Playwright controlled Tauri API mock. Alternative text: "Hemo Tracker locked-vault screen with passphrase and recovery-key unlock forms."
- `desktop-unlocked-overview.png`: Hemo Tracker 0.1.0, browser-rendered desktop webview, 1280 x 900 viewport, light theme, fictional hemoglobin report and personal target range fixture with the first-run demo-data notice, Playwright controlled Tauri API mock. Alternative text: "Hemo Tracker unlocked overview with a hemoglobin trend, personal target ranges, fictional demo-data notice, report history, report entry, and local data controls."

## Naming and alternative text

Use descriptive kebab-case PNG names. Each Markdown image must have alternative text that states the important result or control. Do not use a screenshot as the only source of an instruction.
