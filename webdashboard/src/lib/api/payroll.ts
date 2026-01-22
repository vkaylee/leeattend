import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { Payslip, PayrollSettings, SalaryComponentDefinition, EmployeeSalaryComponent, PayrollIncentivePolicy } from "./types";

export async function fetchIncentivePolicies(wsId: string): Promise<PayrollIncentivePolicy[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/incentive-policies`);
  if (!response.ok) throw new Error("Failed to fetch incentive policies");
  return response.json();
}

export async function createIncentivePolicy(wsId: string, data: any): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/incentive-policies`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create incentive policy");
}

export async function deleteIncentivePolicy(wsId: string, policyId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/incentive-policies/${policyId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete incentive policy");
}

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

export async function fetchPayrollSettingsHistory(wsId: string): Promise<PayrollSettings[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-settings/history`);
  if (!response.ok) throw new Error("Failed to fetch payroll settings history");
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

export async function calculate_payroll(
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

export async function calculateBatchPayslips(
  wsId: string,
  data: { employee_ids: string[]; year: number; month: number }
): Promise<{ results: any[] }> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/payroll/calculate-batch`,
    { 
      method: "POST",
      body: JSON.stringify(data)
    }
  );
  if (!response.ok) throw new Error("Failed to calculate batch payroll");
  return response.json();
}

export async function fetchWorkspacePayslips(
  wsId: string,
  year: number,
  month: number
): Promise<Payslip[]> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/payroll/payslips?year=${year}&month=${month}`
  );
  if (!response.ok) throw new Error("Failed to fetch workspace payslips");
  return response.json();
}

export async function fetchEmployeePayslips(wsId: string, employeeId: string): Promise<Payslip[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payslips`);
  if (!response.ok) throw new Error("Failed to fetch payslips");
  return response.json();
}

export async function setDepartmentPayrollConfig(
  wsId: string, 
  deptId: string, 
  data: { std_working_days: number; hours_per_day: number; effective_date: string }
): Promise<any> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments/${deptId}/payroll-config`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to set department payroll config");
  return response.json();
}

export async function fetchDepartmentPayrollConfigs(
  wsId: string, 
  deptId: string
): Promise<any[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments/${deptId}/payroll-config`);
  if (!response.ok) throw new Error("Failed to fetch department payroll configs");
  return response.json();
}

export async function setEmployeePayrollConfig(
  wsId: string, 
  employeeId: string, 
  data: { std_working_days: number; hours_per_day: number; effective_date: string }
): Promise<any> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payroll-config`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to set employee payroll config");
  return response.json();
}

export async function fetchEmployeePayrollConfigs(
  wsId: string, 
  employeeId: string
): Promise<any[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payroll-config`);
  if (!response.ok) throw new Error("Failed to fetch employee payroll configs");
  return response.json();
}
