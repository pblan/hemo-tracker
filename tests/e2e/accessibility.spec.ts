import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";
import path from "node:path";

const fixtureAnalytes = JSON.parse(
  readFileSync(path.join(process.cwd(), "fixtures/v1/analytes.json"), "utf8"),
);
const fixtureReports = JSON.parse(
  readFileSync(path.join(process.cwd(), "fixtures/v1/reports.json"), "utf8"),
);

test.use({ viewport: { width: 1280, height: 900 }, colorScheme: "light" });

test("unlocked webview has no automated WCAG violations", async ({ page }) => {
  await page.addInitScript(
    ({ analytes, reports }) => {
      Object.assign(window, {
        __TAURI_INTERNALS__: {
          invoke: async (command: string, args?: { reportId?: string }) => {
            if (command === "get_vault_state")
              return { accountExists: true, status: "unlocked" };
            if (command === "list_analyte_definitions") return analytes;
            if (command === "list_lab_reports")
              return reports.map((report) => report.id);
            if (command === "get_lab_report")
              return (
                reports.find((report) => report.id === args?.reportId) ??
                reports[0]
              );
            throw new Error(`Unexpected accessibility command: ${command}`);
          },
        },
      });
    },
    {
      analytes: fixtureAnalytes.analytes,
      reports: fixtureReports.reports,
    },
  );
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Record a lab report" }),
  ).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});
