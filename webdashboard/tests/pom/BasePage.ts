import { Page, expect } from '@playwright/test';

export class BasePage {
  constructor(protected readonly page: Page) {}

  async goto(path: string = '/') {
    await this.page.goto(path);
  }

  async verifyTitle(testId: string) {
    await expect(this.page.getByTestId(testId)).toBeVisible({ timeout: 30000 });
  }
}
