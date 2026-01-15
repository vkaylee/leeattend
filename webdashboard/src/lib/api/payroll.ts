import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { SalaryHistory, Payslip } from "./types";

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
