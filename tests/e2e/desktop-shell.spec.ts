import { expect, test } from "@playwright/test";

test("desktop webview identifies Hemo Tracker", async ({ page }) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", { level: 1, name: "Hemo Tracker" }),
  ).toBeVisible();
  await expect(page.getByText(/local-first laboratory results/i)).toBeVisible();
});
