import { Page, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class LoginPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  async goto() {
    await super.goto('/login');
  }

  async login(email: string, password: string) {
    await this.page.getByTestId('login-email-input').fill(email);
    await this.page.getByTestId('login-password-input').fill(password);
    await this.page.getByTestId('login-submit-btn').click();
  }

  async fastLogin() {
    await this.page.getByTestId('fast-login-btn').click();
  }

  async verifyLoginError() {
    await expect(this.page.getByRole('alert')).toBeVisible();
  }
}
