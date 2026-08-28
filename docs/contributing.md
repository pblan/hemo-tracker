# Contributor guide

## Documentation checks

Install Bun 1.3.5 or a later 1.x release. Install the isolated documentation tools after you clone the repository or change `tools/docs/bun.lock`:

```sh
bun install --cwd tools/docs --frozen-lockfile
```

Run this command after you change a Markdown file, a diagram, or a documentation image:

```sh
bun run --cwd tools/docs check
```

The command prints the number of checked Markdown files when it succeeds. Correct each reported file and run the command again when it fails. Run the install command again if Bun cannot find a documentation tool.

The command checks Markdown style, relative link targets, image alternative text, image file names, and Mermaid syntax. The check does not test external links. Review external sources when you update time-specific research.

Use descriptive kebab-case names for images. Put V1 product screenshots in `docs/assets/screenshots/v1`. Follow the [documentation policy](documentation-policy.md) for ASD-STE100, screenshot, and diagram rules.

The documentation workflow runs only when documentation files or its checker change. It must finish in less than one minute after the dependency cache restores. The V1 repository does not publish GitHub Pages.

## Source checks

See the [root README](../README.md#commands) for the maintained development commands.

The UI validation workflow runs the Playwright webview and accessibility tests
when desktop UI, fixture, or end-to-end test files change. It is separate from
the fast routine CI workflow.
