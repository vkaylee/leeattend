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
import { apiClient } from "./client";

export async function fetchIncentivePolicies(wsId: string): Promise<PayrollIncentivePolicy[]> {
  const { data: resData } = await apiClient.get<PayrollIncentivePolicy[]>(`/workspaces/${wsId}/payroll/incentive-policies`);
    return resData;
}

export async function createIncentivePolicy(wsId: string, data: Partial<PayrollIncentivePolicy>): Promise<void> {
  await apiClient.post(`/workspaces/${wsId}/payroll/incentive-policies`, data);
}

export async function cloneIncentivePolicy(wsId: string, policyId: string): Promise<void> {
  await apiClient.post(`/workspaces/${wsId}/payroll/incentive-policies/${policyId}/clone`, {});
}

export async function deleteIncentivePolicy(wsId: string, policyId: string): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/payroll/incentive-policies/${policyId}`);
}

export async function createSalaryBaseGroup(wsId: string, data: { name: string; slug: string; description?: string }): Promise<SalaryBaseGroup> {
  const { data: resData } = await apiClient.post<SalaryBaseGroup>(`/workspaces/${wsId}/salary-bases`, data);
    return resData;
}

export async function fetchSalaryBaseGroups(wsId: string): Promise<{ groups: SalaryBaseGroup[], members: SalaryBaseGroupMember[] }> {
  const { data: resData } = await apiClient.get<{ groups: SalaryBaseGroup[], members: SalaryBaseGroupMember[] }>(`/workspaces/${wsId}/salary-bases`);
    return resData;
}



export async function assignComponentToSalaryBase(wsId: string, baseId: string, componentId: string): Promise<void> {
  await apiClient.post(`/workspaces/${wsId}/salary-bases/${baseId}/members`, { component_definition_id: componentId });
}

export async function unassignComponentFromSalaryBase(wsId: string, baseId: string, componentId: string): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/salary-bases/${baseId}/members/${componentId}`);
}

export async function fetchPayrollSettings(wsId: string): Promise<PayrollSettings> {
  const { data: resData } = await apiClient.get<PayrollSettings>(`/workspaces/${wsId}/payroll-settings`);
    return resData;
}

export async function updatePayrollSettings(wsId: string, data: Partial<PayrollSettings>): Promise<PayrollSettings> {
  const { data: resData } = await apiClient.patch<PayrollSettings>(`/workspaces/${wsId}/payroll-settings`, data);
    return resData;
}

export async function fetchPayrollSettingsHistory(wsId: string): Promise<PayrollSettings[]> {
  const { data: resData } = await apiClient.get<PayrollSettings[]>(`/workspaces/${wsId}/payroll-settings/history`);
    return resData;
}

export async function fetchSalaryComponents(wsId: string): Promise<SalaryComponentDefinition[]> {
  const { data: resData } = await apiClient.get<SalaryComponentDefinition[]>(`/workspaces/${wsId}/salary-components`);
    return resData;
}

export async function createSalaryComponent(
  wsId: string, 
  data: CreateSalaryComponentRequest
): Promise<SalaryComponentDefinition> {
  const { data: resData } = await apiClient.post<SalaryComponentDefinition>(`/workspaces/${wsId}/salary-components`, data);
    return resData;
}

export async function fetchEmployeeSalaryComponents(wsId: string, employeeId: string): Promise<EmployeeSalaryComponent[]> {
  const { data: resData } = await apiClient.get<EmployeeSalaryComponent[]>(`/workspaces/${wsId}/employees/${employeeId}/salary-components`);
    return resData;
}

export async function assignEmployeeSalaryComponent(
  wsId: string, 
  employeeId: string, 
  data: Partial<EmployeeSalaryComponent>
): Promise<EmployeeSalaryComponent> {
  const { data: resData } = await apiClient.post<EmployeeSalaryComponent>(`/workspaces/${wsId}/employees/${employeeId}/salary-components`, data);
    return resData;
}

export async function calculate_payroll(
  wsId: string,
  employeeId: string,
  year: number,
  month: number,
): Promise<Payslip> {
  const { data: resData } = await apiClient.post<Payslip>(`/workspaces/${wsId}/employees/${employeeId}/calculate-payroll?year=${year.toString()}&month=${month.toString()}`, {});
    return resData;
}

export async function fetchEmployeePayslips(
  wsId: string,
  employeeId: string,
): Promise<Payslip[]> {
  const { data: resData } = await apiClient.get<Payslip[]>(`/workspaces/${wsId}/employees/${employeeId}/payslips`);
    return resData;
}

export async function fetchEmployeePayrollConfigs(wsId: string, employeeId: string): Promise<ConfigRecord[]> {
  const { data: resData } = await apiClient.get<ConfigRecord[]>(`/workspaces/${wsId}/employees/${employeeId}/payroll-config`);
    return resData;
}

// Overtime Policies
export async function fetchOvertimePolicies(wsId: string) {
  const { data: resData } = await apiClient.get<OvertimePolicy[]>(`/workspaces/${wsId}/overtime/policies`);
    return resData;
}

export async function fetchOvertimePolicy(wsId: string, policyId: string) {
  const { data: resData } = await apiClient.get<[OvertimePolicy, OvertimeRule[]]>(`/workspaces/${wsId}/overtime/policies/${policyId}`);
    return resData;
}

export async function createOvertimePolicy(wsId: string, data: Partial<OvertimePolicy>) {
  const { data: resData } = await apiClient.post<OvertimePolicy>(`/workspaces/${wsId}/overtime/policies`, data);
    return resData;
}

export async function addOvertimeRule(wsId: string, policyId: string, data: Partial<OvertimeRule>) {
  const { data: resData } = await apiClient.post<OvertimeRule>(`/workspaces/${wsId}/overtime/policies/${policyId}/rules`, data);
    return resData;
}

export async function setEmployeePayrollConfig(wsId: string, employeeId: string, data: Record<string, unknown>) {
  const { data: resData } = await apiClient.post<Record<string, unknown>>(`/workspaces/${wsId}/employees/${employeeId}/payroll-config`, data);
    return resData;
}

export async function fetchDepartmentPayrollConfigs(wsId: string, deptId: string) {
  const { data: resData } = await apiClient.get<Record<string, unknown>>(`/workspaces/${wsId}/departments/${deptId}/payroll-config`);
    return resData;
}

export async function setDepartmentPayrollConfig(wsId: string, deptId: string, data: Record<string, unknown>) {
  const { data: resData } = await apiClient.post<Record<string, unknown>>(`/workspaces/${wsId}/departments/${deptId}/payroll-config`, data);
    return resData;
}

export async function calculateBatchPayslips(wsId: string, data: { employee_ids: string[]; year: number; month: number }) {
  const { data: resData } = await apiClient.post<{ results: BatchCalculateResult[] }>(`/workspaces/${wsId}/payroll/calculate-batch`, data);
    return resData;
}

export async function fetchWorkspacePayslips(wsId: string, year: number, month: number) {
  const { data: resData } = await apiClient.get<Payslip[]>(`/workspaces/${wsId}/payroll/payslips?year=${year.toString()}&month=${month.toString()}`);
    return resData;
}

export async function assignOvertimePolicy(wsId: string, data: { employee_id?: string; department_id?: string; policy_id: string; effective_from: string }) {
  await apiClient.post(`/workspaces/${wsId}/overtime/policies/assign`, data);
}

// --- Payroll Rules API ---

export async function fetchPayrollRules(wsId: string): Promise<PayrollRuleDefinition[]> {
  const { data: resData } = await apiClient.get<PayrollRuleDefinition[]>(`/workspaces/${wsId}/payroll-rules`);
    return resData;
}

export async function createPayrollRule(wsId: string, data: Partial<PayrollRuleDefinition>): Promise<PayrollRuleDefinition> {
  const { data: resData } = await apiClient.post<PayrollRuleDefinition>(`/workspaces/${wsId}/payroll-rules`, data);
    return resData;
}

export async function fetchPayrollRuleTiers(wsId: string, ruleId: string): Promise<PayrollRuleTier[]> {
  const { data: resData } = await apiClient.get<PayrollRuleTier[]>(`/workspaces/${wsId}/payroll-rules/${ruleId}/tiers`);
    return resData;
}

export async function createPayrollRuleTier(wsId: string, ruleId: string, data: Partial<PayrollRuleTier>): Promise<PayrollRuleTier> {
  const { data: resData } = await apiClient.post<PayrollRuleTier>(`/workspaces/${wsId}/payroll-rules/${ruleId}/tiers`, data);
    return resData;
}

// --- Payroll Rule Assignments ---

export async function fetchPayrollRuleAssignments(wsId: string, ruleId: string): Promise<PayrollRuleAssignment[]> {
  const { data: resData } = await apiClient.get<PayrollRuleAssignment[]>(`/workspaces/${wsId}/payroll-rules/${ruleId}/assignments`);
    return resData;
}

export async function createPayrollRuleAssignment(wsId: string, ruleId: string, data: Partial<PayrollRuleAssignment>): Promise<PayrollRuleAssignment> {
  const { data: resData } = await apiClient.post<PayrollRuleAssignment>(`/workspaces/${wsId}/payroll-rules/${ruleId}/assignments`, data);
    return resData;
}

export async function deletePayrollRuleAssignment(wsId: string, ruleId: string, assignmentId: string): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/payroll-rules/${ruleId}/assignments/${assignmentId}`);
}
