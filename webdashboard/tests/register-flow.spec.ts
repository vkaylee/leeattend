import { test, expect } from '@playwright/test';

test.describe('Registration Flow', () => {
  test('should register a new user and login', async ({ page }) => {
    // 1. Go to Login
    await page.goto('http://localhost:5173/login');
    
    // 2. Click Sign up
    await page.click('a[href="/register"]');
    await expect(page).toHaveURL('http://localhost:5173/register');
    await expect(page.locator('h2')).toContainText('Create Account');

    // 3. Fill Form
    const uniqueEmail = `testuser_${Date.now()}@example.com`;
    await page.fill('input[name="full_name"]', 'Test User');
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', 'password123');
    await page.fill('input[name="confirm_password"]', 'password123');

    // 4. Submit
    await page.click('button[type="submit"]');

    // 5. Expect Redirect to Onboarding (or Home if onboarding skipped logic exists)
    // The current RegisterPage redirects to /onboarding
    await page.waitForURL('**/onboarding');
    
    // Check local storage for token
    const token = await page.evaluate(() => localStorage.getItem('access_token'));
    expect(token).toBeTruthy();
  });
});
