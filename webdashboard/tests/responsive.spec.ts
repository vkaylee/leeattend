import { expect, test } from "./worker-orchestrator";

test.describe("Responsive layout contracts", () => {
  test.beforeEach(async ({ authPage }) => {
    expect(authPage).toBeDefined();
  });

  test("uses the correct navigation for the viewport", async ({ page }, testInfo) => {
    const bottomNav = page.getByTestId("bottom-nav-more");
    const navRail = page.getByTestId("nav-rail-logo");

    if (testInfo.project.name === "responsive-mobile" || testInfo.project.name === "responsive-tablet") {
      await expect(bottomNav).toBeVisible();
      await expect(navRail).toBeHidden();
      return;
    }

    await expect(bottomNav).toBeHidden();
    await expect(navRail).toBeVisible();
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

  test("keeps the end of the app content above the mobile bottom nav", async ({ page }, testInfo) => {
    test.skip(testInfo.project.name !== "responsive-mobile" && testInfo.project.name !== "responsive-tablet", "The bottom nav only exists below lg.");

    const main = page.locator("[data-app-scroll-container]");
    const bottomNav = page.getByTestId("bottom-nav-more").locator("xpath=ancestor::nav");
    const navBox = await bottomNav.boundingBox();
    const paddingBottom = await main.evaluate((element) => Number.parseFloat(getComputedStyle(element).paddingBottom));

    expect(navBox).not.toBeNull();
    expect(paddingBottom, JSON.stringify({ paddingBottom, navHeight: navBox?.height })).toBeGreaterThanOrEqual(navBox!.height);
  });

  test("keeps bottom navigation controls at least 44px tall", async ({ page }, testInfo) => {
    test.skip(testInfo.project.name !== "responsive-mobile" && testInfo.project.name !== "responsive-tablet", "Touch targets are asserted below lg.");

    const bottomNav = page.getByRole("navigation").last();
    const controls = [
      page.getByTestId("bottom-nav-more"),
      bottomNav.getByRole("link").first(),
    ];

    for (const control of controls) {
      const box = await control.boundingBox();
      expect(box).not.toBeNull();
      expect(box!.width).toBeGreaterThanOrEqual(44);
      expect(box!.height).toBeGreaterThanOrEqual(44);
    }
  });

  test("keeps the floating dock visible on desktop", async ({ page }, testInfo) => {
    test.skip(testInfo.project.name !== "responsive-desktop", "Floating dock layout is asserted at the desktop boundary.");

    await page.goto("/employees");

    const dock = page.getByTestId("floating-dock");
    await expect(dock).toBeVisible();

    const dockBox = await dock.boundingBox();
    expect(dockBox).not.toBeNull();
  });
});
