# Issue tracker: GitHub

Issues and specifications for this repository live in GitHub Issues.

Use the `gh` CLI for all operations. Run commands in this repository. The CLI will infer the repository from the Git remote.

## Operations

- Create an issue with `gh issue create`.
- Read an issue and its comments with `gh issue view <number> --comments`.
- List issues with `gh issue list`.
- Add a comment with `gh issue comment <number> --body "..."`.
- Add or remove labels with `gh issue edit`.
- Close an issue with `gh issue close`.

Use JSON output and `jq` when a skill must process issue data.

## Pull requests as a request surface

PRs as a request surface: no.

GitHub uses one number sequence for issues and pull requests. If a reference is ambiguous, run `gh pr view <number>`. If that command fails, run `gh issue view <number>`.

## Publish to the issue tracker

Create a GitHub issue when a skill instructs you to publish a specification or ticket.

## Fetch a ticket

Run `gh issue view <number> --comments`.

## Wayfinding

Use one issue with the label `wayfinder:map` as the map.

Use child issues as tickets. Prefer native GitHub sub-issues. Use a task list if sub-issues are not available.

Use these ticket labels:

- `wayfinder:research`
- `wayfinder:prototype`
- `wayfinder:grilling`
- `wayfinder:task`

Prefer native GitHub issue dependencies for blocking relations. Use a `Blocked by: #<number>` line if native dependencies are not available.

Assign a ticket before work starts. Add the result as a comment. Close the ticket after completion.
