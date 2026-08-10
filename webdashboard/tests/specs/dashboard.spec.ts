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
  test("renders a real responsive Recharts SVG with data marks", async ({ page }) => {
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
    await expect(surface.locator("path:visible, circle:visible, rect.recharts-rectangle:visible").first()).toBeVisible();
  });

  test("shows the loading skeleton while dashboard preferences are pending", async ({ page }) => {
    let releasePreferences: (() => void) | undefined;
    let releaseStats: (() => void) | undefined;
    await page.route("**/api/v1/dashboard/preferences", async (route) => {
      await new Promise<void>((resolve) => { releasePreferences = resolve; });
      await route.fallback();
    });
    await page.route("**/api/v1/dashboard/workspace-stats", async (route) => {
      await new Promise<void>((resolve) => { releaseStats = resolve; });
      await route.fallback();
    });

    const navigation = page.goto("/dashboard");
    await expect.poll(() => Boolean(releasePreferences)).toBe(true);
    await expect.poll(() => Boolean(releaseStats)).toBe(true);
    await expect(page.getByText("...").first()).toBeVisible();
    releasePreferences?.();
    releaseStats?.();
    await navigation;
    await expect(page.locator('[aria-controls="section-charts"]')).toBeVisible();
  });

  test("renders an explicit empty state for a valid empty chart response", async ({ page }) => {
    await page.route("**/api/v1/dashboard/attendance-trend", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([]),
      });
    });

    await openCharts(page);
    await expect(page.getByText(/Không có dữ liệu|No data found/i).first()).toBeVisible({ timeout: 15000 });
  });

  test("shows an alert for an HTTP chart failure and preserves chart error copy", async ({ page }) => {
    await page.route("**/api/v1/dashboard/attendance-trend", async (route) => {
      await route.fulfill({
        status: 503,
        contentType: "application/json",
        body: JSON.stringify({ error: "unavailable" }),
      });
    });

    await openCharts(page);
    const alert = page.getByRole("alert").first();
    await expect(alert).toBeVisible({ timeout: 15000 });
    await expect(alert).toContainText(/lỗi|error|connection/i);
    await expect(alert).toHaveAttribute("aria-live", "assertive");
    expect(await page.getByRole("alert").count()).toBeGreaterThan(1);
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
    await expect(page.getByText(/Không có dữ liệu|No data found/i)).toHaveCount(0);
  });

  test("normalizes persisted old, current, and invalid widget IDs", async ({ page }) => {
    await page.route("**/api/v1/dashboard/preferences", async (route) => {
      if (route.request().method() !== "GET") {
        await route.fallback();
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          id: "dashboard-pref-legacy",
          user_id: "user-mock-1",
          workspace_id: "workspace-mock-1",
          active_view: "custom",
          visible_widgets: {
            attendance_trend: true,
            department_chart: true,
            legacy_attendance_chart: true,
            invalid_widget_id: false,
          },
          widget_order: ["legacy_attendance_chart", "department_chart", "department_chart"],
          collapsed_sections: { summary: false, charts: false, actions: true },
        }),
      });
    });

    await openCharts(page);
    await expect(page.locator("svg.recharts-surface").first()).toBeVisible({ timeout: 15000 });
    await expect(page.getByText(/legacy_attendance_chart|invalid_widget_id/i)).toHaveCount(0);
  });

  test("validates dashboard preference PUT bodies in the API mock", async ({ page }) => {
    await page.goto("/dashboard");
    const invalidResponse = await page.evaluate(async () => {
      const response = await fetch("/api/v1/dashboard/preferences", {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ unknown_field: true }),
      });
      return response.status;
    });
    expect(invalidResponse).toBe(400);

    const validResponse = await page.evaluate(async () => {
      const response = await fetch("/api/v1/dashboard/preferences", {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ active_view: "custom", visible_widgets: { attendance_trend: true } }),
      });
      return response.status;
    });
    expect(validResponse).toBe(200);
  });

  test("keeps dashboard controls within the viewport at every configured responsive size", async ({ page }) => {
    await openCharts(page);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    expect(overflow).toBeLessThanOrEqual(1);

    const headerActions = page.locator("header").last();
    await expect(headerActions).toBeVisible();
    const headerBox = await headerActions.boundingBox();
    expect(headerBox?.width ?? 0).toBeLessThanOrEqual(await page.evaluate(() => window.innerWidth));
  });
});
