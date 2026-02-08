import { test, expect } from './worker-orchestrator';

test.describe.skip('Debug Metadata Verification', () => {
  test('should find debug metadata on the login page in test environment', async ({ page }) => {
    await page.goto('/login');
    
    // Find the Fast Login button which we decorated with debugProps
    const fastLoginBtn = page.getByRole('button', { name: /Fast Login/i });
    await expect(fastLoginBtn).toBeVisible();
    
    // Extract the data-debug-context attribute
    const debugContext = await fastLoginBtn.getAttribute('data-debug-context');
    
    expect(debugContext).toBeDefined();
    expect(debugContext).not.toBeNull();
    
    // Parse the JSON data
    const data = JSON.parse(debugContext!);
    expect(data.devEmail).toBe('admin@leeattend.com');
    expect(data.devPass).toBe('admin123');
    
    console.log('✅ Debug metadata verified successfully on Fast Login button.');
  });
});
