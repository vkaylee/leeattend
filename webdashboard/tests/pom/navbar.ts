import { BasePage } from './base-page';

export class Navbar extends BasePage {
  async logout() {
    // Direct logout button in DSM icon rail
    await this.page.getByTestId('nav-rail-logout').click();
  }

  async openProfile() {
    // Direct profile link in DSM icon rail
    await this.page.getByTestId('nav-rail-user-avatar').click();
  }

  async openSettings() {
    // Settings Hub is accessible via the nav rail system button or direct navigation
    // The DSM shell uses nav-panel groups for navigation
    await this.page.goto('/settings');
  }
}
