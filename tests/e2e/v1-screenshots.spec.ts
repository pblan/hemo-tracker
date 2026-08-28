import { expect, test } from "@playwright/test";
import path from "node:path";

const screenshotDirectory = path.join(
  process.cwd(),
  "docs/assets/screenshots/v1",
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
  await page.getByLabel("Trend analyte").selectOption("hb");
  await page.getByLabel("Target range analyte").selectOption("hb");
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
      const analyte = {
        id: "hb",
        name: "Hemoglobin",
        component: "Hemoglobin",
        property: "MCnc",
        specimen: "Blood",
        scale: "Quantitative",
        aliases: ["Hb"],
        loincCode: "718-7",
        personalTargetRanges: [
          {
            id: "range-1",
            lowerBound: "12.0",
            upperBound: "16.0",
            unit: "g/dL",
            validFrom: "2026-01-01",
            context: "Fictional personal example",
          },
        ],
      };
      const report = {
        id: "report-1",
        collectionTime: "2026-08-20T08:30:00+02:00",
        laboratory: "Fictional Central Laboratory",
        status: "complete",
        sourceFileCount: 1,
        measurementCount: 1,
        sourceFiles: [
          {
            filename: "fictional-report.pdf",
            mediaType: "application/pdf",
            role: "primary",
          },
        ],
        measurements: [
          {
            id: "m1",
            sourceLabel: "Hemoglobin",
            sourceValue: "13.8",
            sourceUnit: "g/dL",
            sourceReferenceInterval: "12.0–16.0",
            sourceFlag: "within range",
            analyteId: "hb",
            updatedAt: "",
            updatedBy: "local-user",
          },
        ],
      };
      Object.assign(window, {
        __TAURI_INTERNALS__: {
          invoke: async (command: string) => {
            if (command === "get_vault_state")
              return { accountExists: true, status: vaultStatus };
            if (command === "list_analyte_definitions") return [analyte];
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
