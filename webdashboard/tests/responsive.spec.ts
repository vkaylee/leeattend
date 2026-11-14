import { expect, test } from "./worker-orchestrator";

test.describe("Responsive layout contracts", () => {
  test.beforeEach(async ({ authPage }) => {
    expect(authPage).toBeDefined();
  });

  const categoryOf = (testInfo: { project: { metadata?: { category?: string } } }) =>
    testInfo.project.metadata?.category;

  const isMobile = (testInfo: { project: { metadata?: { category?: string } } }) =>
    categoryOf(testInfo) === "mobile";

  const isDesktop = (testInfo: { project: { metadata?: { category?: string } } }) =>
    categoryOf(testInfo) === "desktop";

  test("uses sidebar and topbar navigation across viewports", async ({ page }, testInfo) => {
    const topbar = page.getByTestId("topbar-container");
    const sidebar = page.getByTestId("sidebar-container");

    await expect(topbar).toBeVisible();
    if (isMobile(testInfo)) {
      await expect(sidebar).toBeHidden();
      await expect(page.getByTestId("topbar-mobile-menu-btn")).toBeVisible();
    } else {
      await expect(sidebar).toBeVisible();
    }
  });

  test("does not create horizontal page overflow", async ({ page }) => {
    await expect.poll(() => page.evaluate(() => ({
      clientWidth: document.documentElement.clientWidth,
      scrollWidth: document.documentElement.scrollWidth,
    }))).toEqual(expect.objectContaining({
      scrollWidth: expect.any(Number),
    }));

    const dimensions = await page.evaluate(() => ({
      clientWidth: document.documentElement.clientWidth,
      scrollWidth: document.documentElement.scrollWidth,
    }));
    expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.clientWidth);
  });

  test("keeps mobile navigation drawer within viewport", async ({ page }, testInfo) => {
    test.skip(!isMobile(testInfo), "Mobile drawer contract applies only to mobile viewport.");

    await page.getByTestId("topbar-mobile-menu-btn").click();
    const drawer = page.getByTestId("sidebar-mobile-drawer");
    await expect(drawer).toBeVisible();

    const drawerBox = await drawer.boundingBox();
    expect(drawerBox).not.toBeNull();
    expect(drawerBox!.width).toBeLessThanOrEqual(testInfo.project.use.viewport?.width ?? 430);
  });

  test("keeps primary touch controls at least 44px tall on mobile", async ({ page }, testInfo) => {
    test.skip(!isMobile(testInfo), "Touch targets are asserted at mobile boundary.");

    const controls = [
      page.getByTestId("topbar-mobile-menu-btn"),
      page.getByTestId("topbar-search-trigger-mobile"),
    ];

    for (const control of controls) {
      const box = await control.boundingBox();
      expect(box).not.toBeNull();
      expect(box!.width).toBeGreaterThanOrEqual(44);
      expect(box!.height).toBeGreaterThanOrEqual(44);
    }
  });

  test("keeps sidebar visible on desktop", async ({ page }, testInfo) => {
    test.skip(!isDesktop(testInfo), "Desktop sidebar contract applies at desktop boundary.");

    await page.goto("/employees");
    await expect(page.getByTestId("sidebar-container")).toBeVisible();
  });
});
