import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";
import { ConnectivityPage } from "./pom/ConnectivityPage";

test.setTimeout(60000);

test.describe("Connectivity Buttons Visibility Tests", () => {
    test.beforeEach(async ({ page }) => {
        // Login
        await page.goto('/login');
        await page.getByTestId('fast-login-btn').click();

        // Wait for dashboard title to be visible (LoginPage reloads after login)
        await expect(page.getByTestId('dashboard-title')).toBeVisible({ timeout: 30000 });
    });

    test('should show global add device button and agent-specific add device button', async ({ page }) => {
        const connectivity = new ConnectivityPage(page);
        await connectivity.goto();
        await connectivity.waitForLoad();

        // 1. Verify global "Add Device" button
        const globalAddBtn = page.getByTestId('global-add-device-btn');
        await expect(globalAddBtn).toBeVisible();
        await expect(globalAddBtn).toContainText(/Thêm thiết bị|Add Device/);

        // 2. Clear agents if any to test empty state or just use existing ones
        // For this test, we just check if any agent exists, and if so, check the button
        const agentCards = page.locator('[data-testid^="agent-card-"]');
        const agentCount = await agentCards.count();

        if (agentCount > 0) {
            const firstCard = agentCards.first();
            const testId = await firstCard.getAttribute('data-testid') || '';
            const agentId = testId.replace('agent-card-', '');

            // Expand agent
            await connectivity.expandAgent(agentId);

            // Verify button inside card
            const agentAddBtn = page.getByTestId(`add-device-btn-${agentId}`);
            await expect(agentAddBtn).toBeVisible();
            await expect(agentAddBtn).toContainText(/Thêm thiết bị|Add Device/);
        } else {
            console.log("No agents found, skipping agent-specific button check");
        }
    });

    test('global add device button should open form if agent exists', async ({ page }) => {
        const connectivity = new ConnectivityPage(page);
        await connectivity.goto();
        await connectivity.waitForLoad();

        const agentsList = page.locator('[data-testid^="agent-card-"]');
        if (await agentsList.count() > 0) {
            await connectivity.clickGlobalAddDevice();
            await expect(page.getByTestId('add-device-form')).toBeVisible();
        } else {
            test.skip(true, 'No agents available to test global add device form');
        }
    });
});
