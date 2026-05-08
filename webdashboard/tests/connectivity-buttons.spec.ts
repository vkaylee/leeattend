 
import { test, expect } from "./worker-orchestrator";

test.describe("Connectivity Buttons Visibility Tests", () => {
    // eslint-disable-next-line unused-imports/no-unused-vars
    test.beforeEach(async ({ authPage, connectivityPage }) => {
        // authPage fixture automatically logs in and sets up workspace
        await connectivityPage.goto();
        await connectivityPage.waitForLoad();
    });

    test('should show agent-specific add device button', async ({ page, connectivityPage }) => {
        // Check agent-specific button if agents exist
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
        }
    });

    test('agent-specific add device button should open form', async ({ page, connectivityPage }) => {
        const agentCards = page.locator('[data-testid^="agent-card-"]');
        if (await agentCards.count() > 0) {
            const firstCard = agentCards.first();
            const testId = await firstCard.getAttribute('data-testid') ?? '';
            const agentId = testId.replace('agent-card-', '');

            await connectivityPage.expandAgent(agentId);
            await connectivityPage.clickAddDeviceForAgent(agentId);
            await expect(page.getByTestId('add-device-form')).toBeVisible();
        } else {
            test.skip(true, 'No agents available to test add device form');
        }
    });
});

