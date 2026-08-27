# GitHub documentation workflow research

Date: 2026-08-27

## Question

What is a clean documentation and screenshot workflow for Hemo Tracker V1 in this GitHub repository?

## Findings

GitHub renders Mermaid diagrams in Markdown files, issues, pull requests, discussions, and wikis. A fenced code block with the `mermaid` language identifier is sufficient. See [Creating diagrams](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-diagrams).

GitHub recommends relative links and image paths for repository content. Relative paths work on the selected branch and in a local clone. The root README is the normal repository entry page. See [About the repository README file](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes).

GitHub documentation guidance recommends task-focused content, short paragraphs, active voice, meaningful headings, and a scannable structure. It separates quickstarts, procedures, tutorials, concepts, references, and troubleshooting content. See [Best practices for GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/best-practices-for-github-docs).

GitHub requires meaningful alternative text for accessible images. Its style guidance recommends descriptive kebab-case image file names. It also states that page text must explain the information in a diagram or graph. See [Basic writing and formatting syntax](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) and the [GitHub Docs style guide](https://docs.github.com/en/contributing/style-guide-and-content-model/style-guide).

GitHub Pages can publish a `docs` directory, but it creates another deployment and public site boundary. Repository Markdown already meets the V1 need. Pages is not necessary for V1. See [Creating a GitHub Pages site](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site).

`markdownlint-cli2` supports repository globs, ignore globs, and shared configuration. It is a maintained command-line interface for the `markdownlint` rules. See the [`markdownlint-cli2` documentation](https://github.com/DavidAnson/markdownlint-cli2).

Mermaid provides a `parse` function that checks syntax without rendering a diagram. This function avoids a browser and keeps the documentation gate small. See [Mermaid usage](https://mermaid.js.org/config/usage.html#syntax-validation-without-rendering).

## Recommendation

Keep V1 documentation as GitHub-flavored Markdown in the repository. Use the root README as the entry page. Add separate user and operator sections under `docs`.

Use Mermaid UML sequence, state, and class diagrams when a visual improves understanding. Do not add decorative diagrams. Keep the equivalent required information in text.

Use deterministic fictional data for screenshots. Store reviewed V1 images in the repository. Capture screenshots through a repeatable Playwright fixture where possible. Capture native operating-system dialogs manually on each signed platform because browser automation cannot reproduce those dialogs reliably.

Use `markdownlint-cli2` for Markdown style. Use one repository script for relative links, image rules, and Mermaid parsing. Run these checks in a path-filtered workflow. Do not add screenshot rendering to routine source-code CI. Add screenshot freshness and manual visual review to the V1 release gate.
