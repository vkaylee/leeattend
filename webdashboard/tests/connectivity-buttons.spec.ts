/* eslint-disable @typescript-eslint/prefer-nullish-coalescing, no-console */
import { test, expect } from "./worker-orchestrator";

test.describe("Connectivity Buttons Visibility Tests", () => {
    test.beforeEach(async ({ connectivityPage }) => {
        // authPage fixture automatically logs in and sets up workspace
        await connectivityPage.goto();
        await connectivityPage.waitForLoad();
    });

    test('should show global add device button and agent-specific add device button', async ({ page, connectivityPage }) => {
        // 1. Verify global "Add Device" button
        const globalAddButton = page.getByTestId('global-add-device-btn');
        await expect(globalAddButton).toBeVisible();
        // Use regular expression to handle both languages
        await expect(globalAddButton).toHaveText(/add device|thêm thiết bị/i);

        // 2. Check agent-specific button if agents exist
        const agentCards = page.locator('[data-testid^="agent-card-"]');
        const agentCount = await agentCards.count();

        if (agentCount > 0) {
            const firstCard = agentCards.first();
            const testId = await firstCard.getAttribute('data-testid') ?? '';
            const agentId = testId.replace('agent-card-', '');

            // Expand agent to see its inner "Add Device" button
            await connectivityPage.expandAgent(agentId);

            // Verify button inside card
            const agentAddButton = page.getByTestId(`add-device-btn-${agentId}`);
            await expect(agentAddButton).toBeVisible();
            await expect(agentAddButton).toHaveText(/add device|thêm thiết bị/i);
        } else {
            
        }
    });

    test('global add device button should open form if agent exists', async ({ page, connectivityPage }) => {
        const agentsList = page.locator('[data-testid^="agent-card-"]');
        if (await agentsList.count() > 0) {
            await connectivityPage.clickGlobalAddDevice();
            await expect(page.getByTestId('add-device-form')).toBeVisible();
        } else {
            test.skip(true, 'No agents available to test global add device form');
        }
    });
});

