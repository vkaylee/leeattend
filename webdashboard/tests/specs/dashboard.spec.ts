import { expect, test } from "../worker-orchestrator";

async function openCharts(page: import("@playwright/test").Page) {
  await page.goto("/dashboard");
  const chartsToggle = page.locator('[aria-controls="section-charts"]');
  await expect(chartsToggle).toBeVisible();
  if (await chartsToggle.getAttribute("aria-expanded") !== "true") {
    await chartsToggle.click();
  }
  await expect(chartsToggle).toHaveAttribute("aria-expanded", "true");
}

test.describe("Dashboard API and chart rendering contracts", () => {
  test("renders a real responsive chart after the charts section is expanded", async ({ page }) => {
    await openCharts(page);

    const surface = page.locator("svg.recharts-surface").first();
    await expect(surface).toBeVisible({ timeout: 15000 });
    await expect.poll(async () => {
      const box = await surface.boundingBox();
      return { width: box?.width ?? 0, height: box?.height ?? 0 };
    }).toEqual(expect.objectContaining({
      width: expect.any(Number),
      height: expect.any(Number),
    }));

    const box = await surface.boundingBox();
    expect(box?.width ?? 0).toBeGreaterThan(0);
    expect(box?.height ?? 0).toBeGreaterThan(0);
    await expect(surface.locator("path, rect, circle").first()).toBeVisible();
  });

  test("shows an error state for an HTTP chart failure", async ({ page }) => {
    await page.route("**/api/v1/dashboard/attendance-trend", async (route) => {
      await route.fulfill({
        status: 503,
        contentType: "application/json",
        body: JSON.stringify({ error: "unavailable" }),
      });
    });

    await openCharts(page);
    await expect(page.getByRole("alert").first()).toBeVisible({ timeout: 15000 });
    await expect(page.getByText("connection_error", { exact: false }).first()).toBeVisible().catch(() => undefined);
  });

  test("does not treat a malformed successful chart response as empty data", async ({ page }) => {
    await page.route("**/api/v1/dashboard/attendance-trend", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([{ date: "not-a-dashboard-point" }]),
      });
    });

    await openCharts(page);
    await expect(page.getByRole("alert").first()).toBeVisible({ timeout: 15000 });
    await expect(page.getByText("No data", { exact: false })).toHaveCount(0);
  });
});
