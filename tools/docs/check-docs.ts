import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, extname, relative, resolve } from "node:path";

import mermaid from "mermaid";

const root = resolve(import.meta.dir, "../..");
const ignoredDirectories = new Set([".agents", ".git", "node_modules"]);
const markdownlint = resolve(
  import.meta.dir,
  "node_modules/.bin",
  process.platform === "win32" ? "markdownlint-cli2.cmd" : "markdownlint-cli2",
);
const lint = Bun.spawnSync(
  [markdownlint, "**/*.md", "#.agents", "#node_modules"],
  { cwd: root, stderr: "inherit", stdout: "inherit" },
);
if (lint.exitCode !== 0) process.exit(lint.exitCode);

const collectMarkdown = (directory: string): string[] =>
  readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isDirectory() && ignoredDirectories.has(entry.name)) return [];

    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) return collectMarkdown(path);
    return entry.isFile() && extname(entry.name) === ".md" ? [path] : [];
  });

const errors: string[] = [];
const markdownFiles = collectMarkdown(root);

for (const file of markdownFiles) {
  const source = readFileSync(file, "utf8");
  const displayFile = relative(root, file);
  const referenceDefinition = /^\s*\[[^\]]+\]:\s*\S+/m;
  if (referenceDefinition.test(source)) {
    errors.push(`${displayFile}: use inline links instead of reference links`);
  }
  if (/!?\[[^\]]*\]\(<[^>]+>\)/.test(source)) {
    errors.push(`${displayFile}: do not use angle-bracket link destinations`);
  }

  const linkPattern = /(!?)\[([^\]]*)\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
  for (const match of source.matchAll(linkPattern)) {
    const [, imageMarker, label, rawTarget] = match;
    const target = rawTarget.replace(/^<|>$/g, "");

    if (imageMarker === "!") {
      const normalizedLabel = label.trim().toLowerCase();
      if (
        label.trim().length < 8 ||
        normalizedLabel === "image" ||
        normalizedLabel === "screenshot"
      ) {
        errors.push(`${displayFile}: image alternative text must describe the image`);
      }

      const imageName = target.split("/").at(-1)?.split("#")[0] ?? "";
      if (!/^[a-z0-9]+(?:-[a-z0-9]+)+\.[a-z0-9]+$/.test(imageName)) {
        errors.push(`${displayFile}: image file name must be descriptive kebab-case: ${target}`);
      }
    }

    if (/^(?:[a-z]+:|#)/i.test(target)) continue;

    const pathWithoutFragment = decodeURIComponent(target.split("#")[0]);
    const resolvedTarget = resolve(dirname(file), pathWithoutFragment);
    const exists =
      existsSync(resolvedTarget) &&
      (!statSync(resolvedTarget).isDirectory() ||
        existsSync(resolve(resolvedTarget, "README.md")));
    if (!exists) errors.push(`${displayFile}: relative link does not exist: ${target}`);
  }

  const mermaidPattern = /```mermaid\s*\n([\s\S]*?)```/g;
  for (const [index, match] of [...source.matchAll(mermaidPattern)].entries()) {
    try {
      const valid = await mermaid.parse(match[1], { suppressErrors: true });
      if (!valid) throw new Error("Mermaid rejected the diagram syntax");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      errors.push(`${displayFile}: Mermaid block ${index + 1} is invalid: ${message}`);
    }
  }
}

if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exit(1);
}

console.log(`Checked ${markdownFiles.length} Markdown files.`);
