import { expect, test } from "@playwright/test";
import path from "node:path";
import { readFileSync } from "node:fs";

const screenshotDirectory = path.join(
  process.cwd(),
  "docs/assets/screenshots/v1",
);
const fixtureAnalytes = JSON.parse(
  readFileSync(path.join(process.cwd(), "fixtures/v1/analytes.json"), "utf8"),
);
const fixtureReports = JSON.parse(
  readFileSync(path.join(process.cwd(), "fixtures/v1/reports.json"), "utf8"),
);

test.use({ viewport: { width: 1280, height: 900 }, colorScheme: "light" });

test("capture locked vault", async ({ page }) => {
  await mockTauri(page, "locked");
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Unlock your vault" }),
  ).toBeVisible();
  await page.screenshot({
    path: path.join(screenshotDirectory, "desktop-locked-vault.png"),
    fullPage: true,
  });
});

test("capture unlocked overview", async ({ page }) => {
  await mockTauri(page, "unlocked");
  await page.goto("/");
  await page.getByLabel("Trend analyte").selectOption("hemoglobin");
  await page.getByLabel("Target range analyte").selectOption("hemoglobin");
  await expect(page.getByText("Fictional Central Laboratory")).toBeVisible();
  await page.screenshot({
    path: path.join(screenshotDirectory, "desktop-unlocked-overview.png"),
    fullPage: true,
  });
});

async function mockTauri(
  page: import("@playwright/test").Page,
  status: string,
) {
  await page.addInitScript(
    ({ vaultStatus }) => {
      const analyte = fixtureAnalytes.analytes.find(
        (item) => item.id === "hemoglobin",
      );
      const report = fixtureReports.reports[1];
      if (!analyte || !report) throw new Error("V1 fixture is incomplete");
      Object.assign(window, {
        __TAURI_INTERNALS__: {
          invoke: async (command: string) => {
            if (command === "get_vault_state")
              return { accountExists: true, status: vaultStatus };
            if (command === "list_analyte_definitions")
              return fixtureAnalytes.analytes;
            if (command === "list_lab_reports") return [report.id];
            if (command === "get_lab_report") return report;
            throw new Error(`Unexpected screenshot command: ${command}`);
          },
        },
      });
    },
    { vaultStatus: status },
  );
}
