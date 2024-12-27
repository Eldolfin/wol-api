import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page, context }) => {
  await page.goto("/");
});

test.describe("Wol landing page", () => {
  test("Machine list is loading", async ({ page }) => {
    await expect(page.getByText("Testing-docker-container")).toBeVisible();
  });

  test("Terminal is working", async ({ page }) => {
    await page.getByRole("button", { name: "Connect via ssh" }).click();
    await page.locator(".xterm-rows > div").first().click();
    await page.getByLabel("Terminal input").fill("echo 'hello' 'world'");
    await page.getByLabel("Terminal input").press("Enter");
    await expect(page.locator('[id="__nuxt"]')).toContainText("hello world");
  });
});
