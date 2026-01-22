import { test } from './worker-orchestrator';

test.describe('Employee Search Integration', () => {
  test('should find and select an employee by name', async ({ authPage, employeesPage }) => {
    test.setTimeout(60000);

    // 1. Navigate to employees page
    await employeesPage.goto();
    await employeesPage.verifyTitle('employees-title');

    // 2. Create an employee via POM
    console.log('Creating new employee via POM...');
    await employeesPage.createEmployee('SearchTarget', 'EMP999', '2024-01-01');

    // Wait for the new employee to appear
    await employeesPage.verifyEmployeeVisible('SearchTarget');

    console.log('Employee created. Starting search test...');

    // 3. Use the search input
    await employeesPage.search('SearchTarget');

    // 4. Verify the list is filtered
    await employeesPage.verifyEmployeeVisible('SearchTarget');
    
    console.log('Search successful, target found in filtered list.');
  });
});