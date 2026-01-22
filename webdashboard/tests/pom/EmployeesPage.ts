import { Page, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class EmployeesPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  async goto() {
    await super.goto('/employees');
  }

  async createEmployee(name: string, id: string, hireDate: string) {
    await this.page.getByTestId('add-employee-btn').click();
    await this.page.getByTestId('employees-form-name-input').fill(name);
    await this.page.getByTestId('employees-form-id-input').fill(id);
    await this.page.getByTestId('employees-form-hire-date-input').fill(hireDate);
    await this.page.getByTestId('save-employee-btn').click();
  }

  async search(term: string) {
    const input = this.page.getByTestId('employee-search');
    await input.fill(term);
  }

  async verifyEmployeeVisible(name: string) {
    await expect(this.page.getByText(name)).toBeVisible({ timeout: 15000 });
  }

  async selectEmployeeRow(id: string) {
    await this.page.getByTestId(`employees-row-${id}`).click();
  }

  async editEmployee() {
    await this.page.getByTestId('employees-btn-edit').click();
  }
}
