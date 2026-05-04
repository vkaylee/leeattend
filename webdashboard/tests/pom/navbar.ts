import { BasePage } from './base-page';

export class Navbar extends BasePage {
  async logout() {
    // Click User Profile Menu Trigger
    await this.page.getByTestId('navbar-user-menu-trigger').click();
    // Click Logout button in the dropdown
    await this.page.getByTestId('user-menu-logout-btn').click();
  }

  async openProfile() {
    await this.page.getByTestId('navbar-user-menu-trigger').click();
    await this.page.getByTestId('user-menu-profile-btn').click();
  }

  async openSettings() {
    await this.page.getByTestId('navbar-user-menu-trigger').click();
    await this.page.getByTestId('user-menu-settings-btn').click();
  }
}
