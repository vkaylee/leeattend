import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { SalaryHistory, Payslip, PayrollSettings, SalaryComponentDefinition, EmployeeSalaryComponent } from "./types";

export async function fetchPayrollSettings(wsId: string): Promise<PayrollSettings> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-settings`);
  if (!response.ok) throw new Error("Failed to fetch payroll settings");
  return response.json();
}

export async function updatePayrollSettings(wsId: string, data: Partial<PayrollSettings>): Promise<PayrollSettings> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-settings`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update payroll settings");
  return response.json();
}

export async function fetchSalaryComponents(wsId: string): Promise<SalaryComponentDefinition[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-components`);
  if (!response.ok) throw new Error("Failed to fetch salary components");
  return response.json();
}

export async function createSalaryComponent(
  wsId: string, 
  data: Partial<SalaryComponentDefinition>
): Promise<SalaryComponentDefinition> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-components`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create salary component");
  return response.json();
}

export async function fetchEmployeeSalaryComponents(wsId: string, employeeId: string): Promise<EmployeeSalaryComponent[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/salary-components`);
  if (!response.ok) throw new Error("Failed to fetch employee salary components");
  return response.json();
}

export async function assignEmployeeSalaryComponent(
  wsId: string, 
  employeeId: string, 
  data: Partial<EmployeeSalaryComponent>
): Promise<EmployeeSalaryComponent> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/salary-components`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to assign salary component");
  return response.json();
}

export async function fetchSalaryHistory(wsId: string, employeeId: string): Promise<SalaryHistory[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/salary`);
  if (!response.ok) throw new Error("Failed to fetch salary history");
  return response.json();
}

export async function addSalaryHistory(
  wsId: string, 
  employeeId: string, 
  data: Partial<SalaryHistory>
): Promise<SalaryHistory> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/salary`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to add salary history");
  return response.json();
}

export async function calculatePayroll(
  wsId: string,
  employeeId: string,
  year: number,
  month: number,
): Promise<Payslip> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/calculate-payroll?year=${year}&month=${month}`,
    { method: "POST" }
  );
  if (!response.ok) throw new Error("Failed to calculate payroll");
  return response.json();
}

export async function fetchEmployeePayslips(wsId: string, employeeId: string): Promise<Payslip[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payslips`);
  if (!response.ok) throw new Error("Failed to fetch payslips");
  return response.json();
}
