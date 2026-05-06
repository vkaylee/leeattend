import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { 
  Payslip, 
  PayrollSettings,
  SalaryComponentDefinition,
  EmployeeSalaryComponent, 
  CreateSalaryComponentRequest,
  PayrollIncentivePolicy,
  PayrollRuleAssignment,
  SalaryBaseGroup,
  SalaryBaseGroupMember,
  OvertimePolicy,
  OvertimeRule,
  PayrollRuleDefinition,
  PayrollRuleTier,
  BatchCalculateResult,
  ConfigRecord
} from "./types";

export async function fetchIncentivePolicies(wsId: string): Promise<PayrollIncentivePolicy[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/incentive-policies`);
 
  if (!response.ok) throw new Error("Failed to fetch incentive policies");
  return response.json() as Promise<PayrollIncentivePolicy[]>;
}

export async function createIncentivePolicy(wsId: string, data: Partial<PayrollIncentivePolicy>): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/incentive-policies`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create incentive policy");
}

export async function cloneIncentivePolicy(wsId: string, policyId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/incentive-policies/${policyId}/clone`, {
    method: "POST",
  });
  if (!response.ok) throw new Error("Failed to clone incentive policy");
}

export async function deleteIncentivePolicy(wsId: string, policyId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/incentive-policies/${policyId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete overtime policy");
}

export async function createSalaryBaseGroup(wsId: string, data: { name: string; slug: string; description?: string }): Promise<SalaryBaseGroup> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-bases`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create salary base group");
  return response.json() as Promise<SalaryBaseGroup>;
}

export async function fetchSalaryBaseGroups(wsId: string): Promise<{ groups: SalaryBaseGroup[], members: SalaryBaseGroupMember[] }> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-bases`);
  if (!response.ok) throw new Error("Failed to fetch salary bases");
  return response.json() as Promise<{ groups: SalaryBaseGroup[], members: SalaryBaseGroupMember[] }>;
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
  return response.json() as Promise<PayrollSettings>;
}

export async function updatePayrollSettings(wsId: string, data: Partial<PayrollSettings>): Promise<PayrollSettings> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-settings`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update payroll settings");
  return response.json() as Promise<PayrollSettings>;
}

export async function fetchPayrollSettingsHistory(wsId: string): Promise<PayrollSettings[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-settings/history`);
  if (!response.ok) throw new Error("Failed to fetch payroll settings history");
  return response.json() as Promise<PayrollSettings[]>;
}

export async function fetchSalaryComponents(wsId: string): Promise<SalaryComponentDefinition[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-components`);
  if (!response.ok) throw new Error("Failed to fetch salary components");
  return response.json() as Promise<SalaryComponentDefinition[]>;
}

export async function createSalaryComponent(
  wsId: string, 
  data: CreateSalaryComponentRequest
): Promise<SalaryComponentDefinition> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/salary-components`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create salary component");
  return response.json() as Promise<SalaryComponentDefinition>;
}

export async function fetchEmployeeSalaryComponents(wsId: string, employeeId: string): Promise<EmployeeSalaryComponent[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/salary-components`);
  if (!response.ok) throw new Error("Failed to fetch employee salary components");
  return response.json() as Promise<EmployeeSalaryComponent[]>;
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
  return response.json() as Promise<EmployeeSalaryComponent>;
}

export async function calculate_payroll(
  wsId: string,
  employeeId: string,
  year: number,
  month: number,
): Promise<Payslip> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/calculate-payroll?year=${year.toString()}&month=${month.toString()}`,
    { method: "POST" }
  );
  if (!response.ok) throw new Error("Failed to calculate payroll");
  return response.json() as Promise<Payslip>;
}

export async function fetchEmployeePayslips(
  wsId: string,
  employeeId: string,
): Promise<Payslip[]> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payslips`
  );
  if (!response.ok) throw new Error("Failed to fetch payslips");
  return response.json() as Promise<Payslip[]>;
}

export async function fetchEmployeePayrollConfigs(wsId: string, employeeId: string): Promise<ConfigRecord[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payroll-config`);
  if (!response.ok) throw new Error("Failed to fetch employee payroll configs");
  return response.json() as Promise<ConfigRecord[]>;
}

// Overtime Policies
export async function fetchOvertimePolicies(wsId: string) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies`);
  if (!response.ok) throw new Error("Failed to fetch overtime policies");
  return response.json() as Promise<OvertimePolicy[]>;
}

export async function fetchOvertimePolicy(wsId: string, policyId: string) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies/${policyId}`);
 
  if (!response.ok) throw new Error("Failed to fetch overtime policy");
  return response.json() as Promise<[OvertimePolicy, OvertimeRule[]]>;
}

export async function createOvertimePolicy(wsId: string, data: Partial<OvertimePolicy>) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies`, {
    method: "POST",
    body: JSON.stringify(data),
  });
 
  if (!response.ok) throw new Error("Failed to create overtime policy");
  return response.json() as Promise<OvertimePolicy>;
}

export async function addOvertimeRule(wsId: string, policyId: string, data: Partial<OvertimeRule>) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies/${policyId}/rules`, {
    method: "POST",
    body: JSON.stringify(data),
  });
 
  if (!response.ok) throw new Error("Failed to add overtime rule");
  return response.json() as Promise<OvertimeRule>;
}

export async function setEmployeePayrollConfig(wsId: string, employeeId: string, data: Record<string, unknown>) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payroll-config`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to set employee payroll config");
  return response.json() as Promise<Record<string, unknown>>;
}

export async function fetchDepartmentPayrollConfigs(wsId: string, deptId: string) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments/${deptId}/payroll-config`);
 
  if (!response.ok) throw new Error("Failed to fetch department payroll configs");
  return response.json() as Promise<Record<string, unknown>>;
}

export async function setDepartmentPayrollConfig(wsId: string, deptId: string, data: Record<string, unknown>) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments/${deptId}/payroll-config`, {
    method: "POST",
    body: JSON.stringify(data),
  });
 
  if (!response.ok) throw new Error("Failed to set department payroll config");
  return response.json() as Promise<Record<string, unknown>>;
}

export async function calculateBatchPayslips(wsId: string, data: { employee_ids: string[]; year: number; month: number }) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/calculate-batch`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to calculate batch payslips");
  return response.json() as Promise<{ results: BatchCalculateResult[] }>;
}

export async function fetchWorkspacePayslips(wsId: string, year: number, month: number) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll/payslips?year=${year.toString()}&month=${month.toString()}`);
 
  if (!response.ok) throw new Error("Failed to fetch workspace payslips");
  return response.json() as Promise<Payslip[]>;
}

export async function assignOvertimePolicy(wsId: string, data: { employee_id?: string; department_id?: string; policy_id: string; effective_from: string }) {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/overtime/policies/assign`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to assign overtime policy');
  return response.ok;
 
}

// --- Payroll Rules API ---

export async function fetchPayrollRules(wsId: string): Promise<PayrollRuleDefinition[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules`);
 
 
  if (!response.ok) throw new Error("Failed to fetch payroll rules");
  return response.json() as Promise<PayrollRuleDefinition[]>;
}

export async function createPayrollRule(wsId: string, data: Partial<PayrollRuleDefinition>): Promise<PayrollRuleDefinition> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules`, {
    method: "POST",
    body: JSON.stringify(data),
  });
 
  if (!response.ok) throw new Error("Failed to create payroll rule");
  return response.json() as Promise<PayrollRuleDefinition>;
}

export async function fetchPayrollRuleTiers(wsId: string, ruleId: string): Promise<PayrollRuleTier[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/tiers`);
 
 
  if (!response.ok) throw new Error("Failed to fetch payroll rule tiers");
  return response.json() as Promise<PayrollRuleTier[]>;
}

export async function createPayrollRuleTier(wsId: string, ruleId: string, data: Partial<PayrollRuleTier>): Promise<PayrollRuleTier> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/tiers`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create payroll rule tier");
  return response.json() as Promise<PayrollRuleTier>;
}

// --- Payroll Rule Assignments ---

export async function fetchPayrollRuleAssignments(wsId: string, ruleId: string): Promise<PayrollRuleAssignment[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/assignments`);
  if (!response.ok) throw new Error("Failed to fetch rule assignments");
  return response.json() as Promise<PayrollRuleAssignment[]>;
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
  return response.json() as Promise<PayrollRuleAssignment>;
}

export async function deletePayrollRuleAssignment(wsId: string, ruleId: string, assignmentId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/assignments/${assignmentId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete rule assignment");
}
