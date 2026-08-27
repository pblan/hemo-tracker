# Documentation policy

## Purpose

This policy keeps the V1 usage documentation and technical documentation clear, current, and easy to use on GitHub.

All maintained text must follow ASD-STE100. Each page must use one term for one concept. Each procedure must state its result, prerequisites, steps, verification, and common recovery actions.

## Documentation structure

The root `README.md` is the entry page. It must explain the product, state its safety limits, provide the shortest supported start path, and link to the detailed documentation.

Use these directories:

```text
docs/
  user/          End-user setup and task procedures
  operations/    Self-hosting and recovery procedures
  adr/           Accepted architecture decisions
  proofs/        Security and technology proof results
  research/      Time-specific source research
  assets/
    screenshots/ Versioned product screenshots
```

Keep documentation in the repository. Use relative links and image paths so that links work on a branch and in a local clone. Do not require GitHub Pages for V1.

## Content types

Use a short quickstart for the first successful workflow. Use task procedures for report entry, source-file management, analyte management, plots, ranges, synchronization, backup, restore, recovery, device revocation, and account deletion.

Keep operator procedures separate from end-user procedures. Keep architecture, domain rules, ADRs, proof results, and contributor commands in technical documentation.

Do not repeat one rule on many pages. Link to the page that owns the rule.

## Diagrams

Use a diagram only when it makes a relationship, sequence, state change, or dependency easier to understand. Use fenced Mermaid syntax that GitHub can render in Markdown files and issues.

Use a flowchart for system boundaries. Use a sequence diagram for authentication, unlock, synchronization, backup, and recovery. Use a state diagram for report, device, and conflict states.

Explain the important diagram information in the adjacent text. Do not make a diagram the only source of required information.

## Screenshots

Store V1 screenshots under `docs/assets/screenshots/v1/`. Use descriptive kebab-case file names. Use PNG unless another lossless format has a clear size benefit.

Create screenshots from deterministic fictional data. Never use real health data, account data, email addresses, secrets, file paths, or device names. Use the same window size, theme, sample account, and sample reports for each capture.

Capture macOS and Windows when the interface or operating-system step differs. Use one screenshot when both platforms show the same webview content.

Each image must have meaningful alternative text. The text must state the important result or control. It must not repeat all visible details. Adjacent text must contain every instruction that the screenshot supports.

Record the application version and capture procedure in `docs/assets/screenshots/v1/README.md`. Replace a screenshot when the related interface changes. Keep the same file name when the screenshot still represents the same step.

## Verification

The documentation gate must check these items before V1:

- All relative links and image paths work.
- All Mermaid diagrams render on GitHub.
- Each image has meaningful alternative text.
- Each screenshot uses fictional data and matches the V1 interface.
- The quickstart works from a clean supported computer.
- macOS and Windows differences are correct.
- The application safety limits match the product and security documents.
- A reviewer checks all maintained text against ASD-STE100.

Routine source-code checks must not render or compare screenshots. Run documentation checks only when Markdown files, diagram sources, or documentation assets change. Run screenshot capture and visual review as a release gate or when a related interface changes.
