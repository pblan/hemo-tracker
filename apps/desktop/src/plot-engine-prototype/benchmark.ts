import { readdirSync, statSync, writeFileSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "@playwright/test";

type EngineResult = {
  firstRenderMs: number;
  pointerP95Ms: number;
  heapBytes: number;
  themeChangeMs: number;
  exportBytes: number;
  keyboardTableReachable: boolean;
  cleanupCanvasCount: number;
  candidateChunkBytes: number;
};

const scriptDirectory = fileURLToPath(new URL(".", import.meta.url));
const repository = resolve(scriptDirectory, "../../../..");
const server = spawn(
  "bun",
  ["run", "--cwd", "apps/desktop", "dev", "--host", "127.0.0.1"],
  { cwd: repository, stdio: ["ignore", "ignore", "inherit"] },
);

async function waitForServer() {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      const response = await fetch("http://127.0.0.1:1420");
      if (response.ok) return;
    } catch {
      // The development server can still be starting.
    }
    await new Promise((done) => setTimeout(done, 100));
  }
  throw new Error("Prototype server did not start");
}

async function measure(engine: "echarts" | "uplot"): Promise<EngineResult> {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    viewport: { width: 1440, height: 900 },
  });
  await page.goto(
    `http://127.0.0.1:1420/?plot-engine-prototype&engine=${engine}`,
  );
  await page.locator('body[data-ready="true"]').waitFor();
  const firstRenderMs = Number(
    await page.locator("body").getAttribute("data-first-render-ms"),
  );

  const pointerLatencies = await page.evaluate(async () => {
    const canvas = document.querySelector("canvas");
    if (!canvas) throw new Error("Candidate did not create a canvas");
    const samples: number[] = [];
    for (let index = 0; index < 40; index += 1) {
      const started = performance.now();
      canvas.dispatchEvent(
        new PointerEvent("pointermove", {
          bubbles: true,
          clientX: 20 + index,
          clientY: 50,
        }),
      );
      await new Promise<void>((done) => requestAnimationFrame(() => done()));
      samples.push(performance.now() - started);
    }
    return samples.sort((left, right) => left - right);
  });
  const pointerP95Ms =
    pointerLatencies[Math.floor(pointerLatencies.length * 0.95)]!;

  const session = await page.context().newCDPSession(page);
  await session.send("Performance.enable");
  const performanceMetrics = await session.send("Performance.getMetrics");
  const heapBytes =
    performanceMetrics.metrics.find(
      (metric) => metric.name === "JSHeapUsedSize",
    )?.value ?? 0;

  const themeStarted = performance.now();
  await page.getByRole("button", { name: "Change theme" }).click();
  await page.evaluate(
    () =>
      new Promise<void>((done) =>
        requestAnimationFrame(() => requestAnimationFrame(() => done())),
      ),
  );
  const themeChangeMs = performance.now() - themeStarted;

  const summary = page.locator("summary").first();
  await summary.focus();
  await page.keyboard.press("Enter");
  await page.keyboard.press("Tab");
  const keyboardTableReachable = await page
    .getByRole("region", { name: /result table/ })
    .first()
    .evaluate((element) => element === document.activeElement);

  const exportBytes = await page.evaluate((selectedEngine) => {
    const canvas = document.querySelector("canvas");
    if (!canvas) return 0;
    if (selectedEngine === "uplot") return canvas.toDataURL("image/png").length;
    return Number(
      document.querySelector<HTMLElement>("[data-export-bytes]")?.dataset
        .exportBytes ?? 0,
    );
  }, engine);

  await page.goto("about:blank");
  const cleanupCanvasCount = await page.locator("canvas").count();
  await browser.close();

  const assets = resolve(repository, "apps/desktop/dist/assets");
  const candidateChunkBytes = readdirSync(assets)
    .filter((name) =>
      name.toLowerCase().includes(engine === "uplot" ? "uplot" : "echarts"),
    )
    .reduce((total, name) => total + statSync(resolve(assets, name)).size, 0);

  return {
    firstRenderMs,
    pointerP95Ms,
    heapBytes,
    themeChangeMs,
    exportBytes,
    keyboardTableReachable,
    cleanupCanvasCount,
    candidateChunkBytes,
  };
}

try {
  await waitForServer();
  const build = spawnSync("bun", ["run", "--cwd", "apps/desktop", "build"], {
    cwd: repository,
    stdio: "inherit",
  });
  if (build.status !== 0) throw new Error("Prototype build failed");
  const results = {
    fixture: {
      reports: 1_000,
      measurements: 100_000,
      analytes: 250,
      visiblePlots: 20,
    },
    measuredAt: new Date().toISOString(),
    uplot: await measure("uplot"),
    echarts: await measure("echarts"),
  };
  const output = resolve(scriptDirectory, "plot-benchmark-results.json");
  writeFileSync(output, `${JSON.stringify(results, null, 2)}\n`);
  console.log(JSON.stringify(results, null, 2));
} finally {
  server.kill();
}
