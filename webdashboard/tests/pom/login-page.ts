/* eslint-disable @typescript-eslint/no-useless-constructor */
import { Page, expect } from '@playwright/test';

import { MOCK_USER, MOCK_WORKSPACE } from '../mocks/mock-data';
import { BasePage } from './base-page';

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
    // We inject keys directly into localStorage to bypass API calls.
    // Use correct snake_case keys used by the application.
    await this.page.evaluate(({ user, workspace }) => {
      
      localStorage.setItem('access_token', 'mock-access-token');
      localStorage.setItem('refresh_token', 'mock-refresh-token');
      localStorage.setItem('active_user_id', user.id);
      localStorage.setItem('active_workspace_id', workspace.id);
      localStorage.setItem('currentUser', JSON.stringify(user));
      localStorage.setItem('e2e_test_mode', 'true');
    }, { user: MOCK_USER, workspace: MOCK_WORKSPACE });
  }

  async verifyLoginError() {
    await expect(this.page.getByRole('alert')).toBeVisible();
  }
}
