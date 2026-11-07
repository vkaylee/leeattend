import { expect, test } from "../worker-orchestrator";

test.describe("Standalone Desktop Dashboard Contracts", () => {
  test("renders desktop page with default active widgets", async ({ page }) => {
    await page.goto("/dashboard");

    const desktopPage = page.getByTestId("desktop-page");
    await expect(desktopPage).toBeVisible({ timeout: 15000 });

    const widgetGrid = page.getByTestId("desktop-widget-grid");
    await expect(widgetGrid).toBeVisible();

    // Default widgets should be rendered
    await expect(page.getByTestId("desktop-widget-quick-punch")).toBeVisible();
    await expect(page.getByTestId("desktop-widget-quick-shortcuts")).toBeVisible();
    await expect(page.getByTestId("desktop-widget-system-health")).toBeVisible();
  });

  test("shows the loading skeleton while permissions are pending", async ({ page }) => {
    const { promise: permissionsPending, resolve: releasePermissions } = Promise.withResolvers<void>();
    await page.route("**/api/v1/me/permissions", async (route) => {
      await permissionsPending;
      await route.fallback();
    });

    const navigation = page.goto("/dashboard");
    const skeleton = page.getByTestId("desktop-skeleton");
    await expect(skeleton).toBeVisible({ timeout: 10000 });

    releasePermissions();
    await navigation;
    await expect(page.getByTestId("desktop-page")).toBeVisible({ timeout: 15000 });
  });

  test("renders customizer modal and allows toggling widgets", async ({ page }) => {
    await page.goto("/dashboard");
    await expect(page.getByTestId("desktop-page")).toBeVisible({ timeout: 15000 });

    const customizeBtn = page.getByTestId("desktop-customize-btn");
    await expect(customizeBtn).toBeVisible();
    await customizeBtn.click();

    const customizerModal = page.getByTestId("desktop-customizer-modal");
    await expect(customizerModal).toBeVisible();

    // Toggle quick-punch off
    const toggleBtn = page.getByTestId("toggle-widget-quick-punch");
    await expect(toggleBtn).toBeVisible();
    await toggleBtn.click();

    // Close modal
    await page.getByRole("button", { name: /Xong|Done/i }).click();
    await expect(customizerModal).toBeHidden();

    // Verify quick-punch is hidden
    await expect(page.getByTestId("desktop-widget-quick-punch")).toHaveCount(0);

    // Re-open and reset to default
    await customizeBtn.click();
    await expect(customizerModal).toBeVisible();
    await page.getByTestId("reset-widgets-btn").click();
    await page.getByRole("button", { name: /Xong|Done/i }).click();

    // Verify quick-punch is restored
    await expect(page.getByTestId("desktop-widget-quick-punch")).toBeVisible();
  });

  test("renders empty state when all widgets are disabled", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("leeattend-desktop-widgets-v1", JSON.stringify({
        enabledIds: [],
        order: [],
      }));
    });

    await page.goto("/dashboard");
    await expect(page.getByTestId("desktop-page")).toBeVisible({ timeout: 15000 });

    const emptyState = page.getByTestId("desktop-empty-state");
    await expect(emptyState).toBeVisible();
    await expect(page.getByTestId("desktop-empty-add-widget-btn")).toBeVisible();
  });

  test("opens and closes DSM 7 feature search from header trigger", async ({ page }) => {
    await page.goto("/dashboard");
    await expect(page.getByTestId("desktop-page")).toBeVisible({ timeout: 15000 });

    const searchBtn = page.getByTestId("desktop-search-trigger");
    await expect(searchBtn).toBeVisible();
    await searchBtn.click();

    const launcherModal = page.getByTestId("dsm-launcher-modal");
    await expect(launcherModal).toBeVisible();

    // Close using close button
    const closeBtn = page.getByTestId("dsm-launcher-close-btn");
    await closeBtn.click();
    await expect(launcherModal).toBeHidden();
  });

  test("keeps dashboard controls within viewport without horizontal overflow", async ({ page }) => {
    await page.goto("/dashboard");
    await expect(page.getByTestId("desktop-page")).toBeVisible({ timeout: 15000 });

    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    expect(overflow).toBeLessThanOrEqual(1);

    const header = page.getByTestId("desktop-customize-btn");
    await expect(header).toBeVisible();
    const box = await header.boundingBox();
    expect(box?.x ?? 0).toBeGreaterThanOrEqual(0);
  });
});
