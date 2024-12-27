import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
});

test.describe("Wol landing page", () => {
  test("Machine list is loading", async ({ page }) => {
    await expect(page.getByText("Testing-docker-container")).toBeVisible();
  });
});
