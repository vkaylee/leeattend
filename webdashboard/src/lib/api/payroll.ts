import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { 
  Payslip, 
  PayrollSettings, 
  SalaryComponentDefinition, 
  EmployeeSalaryComponent, 
  PayrollIncentivePolicy,
  PayrollRuleAssignment,
  SalaryBaseGroup,
  SalaryBaseGroupMember
} from "./types";

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
  if (!response.ok) throw new Error("Failed to delete overtime policy");
}

export async function fetchSalaryBaseGroups(wsId: string): Promise<{ groups: SalaryBaseGroup[], members: SalaryBaseGroupMember[] }> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-bases`);
  if (!response.ok) throw new Error("Failed to fetch salary bases");
  return response.json();
}



export async function assignComponentToSalaryBase(wsId: string, baseId: string, componentId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-bases/${baseId}/members`, {
    method: "POST",
    body: JSON.stringify({ component_definition_id: componentId }),
  });
  if (!response.ok) throw new Error("Failed to assign component");
}

export async function unassignComponentFromSalaryBase(wsId: string, baseId: string, componentId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-bases/${baseId}/members/${componentId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to unassign component");
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

export async function fetchEmployeePayslips(
  wsId: string,
  employeeId: string,
): Promise<Payslip[]> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payslips`
  );
  if (!response.ok) throw new Error("Failed to fetch payslips");
  return response.json();
}

export async function fetchEmployeePayrollConfigs(wsId: string, employeeId: string) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payroll-config`);
  if (!response.ok) throw new Error("Failed to fetch employee payroll configs");
  return response.json();
}

// Overtime Policies
export async function fetchOvertimePolicies(wsId: string) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies`);
  if (!response.ok) throw new Error("Failed to fetch overtime policies");
  return response.json();
}

export async function fetchOvertimePolicy(wsId: string, policyId: string) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies/${policyId}`);
  if (!response.ok) throw new Error("Failed to fetch overtime policy");
  return response.json();
}

export async function createOvertimePolicy(wsId: string, data: any) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create overtime policy");
  return response.json();
}

export async function addOvertimeRule(wsId: string, policyId: string, data: any) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies/${policyId}/rules`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to add overtime rule");
  return response.json();
}

export async function setEmployeePayrollConfig(wsId: string, employeeId: string, data: any) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payroll-config`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to set employee payroll config");
  return response.json();
}

export async function fetchDepartmentPayrollConfigs(wsId: string, deptId: string) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments/${deptId}/payroll-config`);
  if (!response.ok) throw new Error("Failed to fetch department payroll configs");
  return response.json();
}

export async function setDepartmentPayrollConfig(wsId: string, deptId: string, data: any) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments/${deptId}/payroll-config`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to set department payroll config");
  return response.json();
}

export async function calculateBatchPayslips(wsId: string, data: any) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/calculate-batch`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to calculate batch payslips");
  return response.json();
}

export async function fetchWorkspacePayslips(wsId: string, year: number, month: number) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/payslips?year=${year}&month=${month}`);
  if (!response.ok) throw new Error("Failed to fetch workspace payslips");
  return response.json();
}

export async function assignOvertimePolicy(wsId: string, data: any) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies/assign`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to assign overtime policy');
  return response.ok;
}

// --- Payroll Rules API ---

export async function fetchPayrollRules(wsId: string): Promise<any[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules`);
  if (!response.ok) throw new Error("Failed to fetch payroll rules");
  return response.json();
}

export async function createPayrollRule(wsId: string, data: any): Promise<any> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create payroll rule");
  return response.json();
}

export async function fetchPayrollRuleTiers(wsId: string, ruleId: string): Promise<any[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/tiers`);
  if (!response.ok) throw new Error("Failed to fetch payroll rule tiers");
  return response.json();
}

export async function createPayrollRuleTier(wsId: string, ruleId: string, data: any): Promise<any> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/tiers`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create payroll rule tier");
  return response.json();
}

// --- Payroll Rule Assignments ---

export async function fetchPayrollRuleAssignments(wsId: string, ruleId: string): Promise<PayrollRuleAssignment[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/assignments`);
  if (!response.ok) throw new Error("Failed to fetch rule assignments");
  return response.json();
}

export async function createPayrollRuleAssignment(wsId: string, ruleId: string, data: Partial<PayrollRuleAssignment>): Promise<PayrollRuleAssignment> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/assignments`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const errorBody = await response.text().catch(() => "");
    throw new Error(errorBody || "Failed to create rule assignment");
  }
  return response.json();
}

export async function deletePayrollRuleAssignment(wsId: string, ruleId: string, assignmentId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/assignments/${assignmentId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete rule assignment");
}
