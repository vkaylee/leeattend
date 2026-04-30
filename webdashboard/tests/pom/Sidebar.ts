import { Page, _expect } from '@playwright/test';

export class Sidebar {
  constructor(private readonly page: Page) {}

  async navigateTo(testId: string) {
    const link = this.page.getByTestId(testId);
    if (!(await link.isVisible())) {
      // Logic to expand section if needed could go here
      // For now, assume standard visibility or use direct navigation if needed
    }
    await link.click();
  }

  async expandSection(sectionTestId: string) {
    await this.page.getByTestId(sectionTestId).click();
  }

  async logout() {
    await this.page.getByTestId('sidebar-btn-logout').click();
  }
}
