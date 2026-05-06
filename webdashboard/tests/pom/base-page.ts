import { Page, expect } from '@playwright/test';

export class BasePage {
  constructor(protected readonly page: Page) {}

  async goto(path = '/') {
    await this.page.goto(path);
  }

  async verifyTitle(testId: string) {
    await expect(this.page.getByTestId(testId)).toBeVisible({ timeout: 10000 });
  }
}
