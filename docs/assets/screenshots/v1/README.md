# V1 screenshot manifest

This directory stores screenshots for the unsigned local V1 release.

## Capture procedure

1. Create a clean local vault with fictional data only.
2. Use the same 1280 x 900 viewport, light theme, and fixture data on macOS and Windows.
3. Capture the webview with the Playwright flow after the interface is stable.
4. Capture native file dialogs manually. Do not include real file names, paths, account names, or secrets.
5. Record the application version, platform, viewport, theme, fixture, and capture method in the image entry below.
6. Review each image for personal data and verify the adjacent procedure contains all required instructions.

## Current entries

- `desktop-locked-vault.png`: Hemo Tracker 0.1.0, browser-rendered desktop webview, 1280 x 900 viewport, light theme, fictional locked-vault fixture, Playwright controlled Tauri API mock. Alternative text: "Hemo Tracker locked-vault screen with passphrase and recovery-key unlock forms."

## Naming and alternative text

Use descriptive kebab-case PNG names. Each Markdown image must have alternative text that states the important result or control. Do not use a screenshot as the only source of an instruction.
