/* eslint-disable max-lines -- API wrapper module: 300 lines of exported functions; splitting would scatter imports */
import { client, fetch } from "./client";
import {
  Payslip,
  PayrollSettings,
  UpdatePayrollSettingsRequest,
  SalaryComponentDefinition,
  EmployeeSalaryComponent,
  EmployeeSalaryComponentDetail,
  CreateSalaryComponentRequest,
  CreatePayrollRuleDefinitionRequest,
  PayrollIncentivePolicyResponse,
  PayrollRuleAssignment,
  SalaryBaseGroup,
  SalaryBaseGroupMember,
  OvertimePolicy,
  OvertimeRule,
  PayrollRuleDefinition,
  PayrollRuleTier,
  BatchCalculateResult,
  ConfigRecord,
  CreateOvertimePolicyRequest,
  EmployeePayrollConfig,
  DepartmentPayrollConfig,
  SetEmployeePayrollConfigRequest,
  SetDepartmentPayrollConfigRequest,
  IncentiveTier,
} from "./types";

export async function fetchIncentivePolicies(): Promise<PayrollIncentivePolicyResponse[]> {
  const { data: resData } = await client.GET<PayrollIncentivePolicyResponse[]>("/v1/payroll/incentive-policies");
  return resData;
}

export async function createIncentivePolicy(data: { base_amount: number; effective_from: string; name: string; payout_type: string; tiers: Omit<IncentiveTier, 'id' | 'policy_id'>[] }): Promise<void> {
  await client.POST("/v1/payroll/incentive-policies", {
    body: data,
  });
}

export async function cloneIncentivePolicy(policyId: string): Promise<void> {
  await client.POST("/v1/payroll/incentive-policies/{policy_id}/clone", {
    params: { path: { policy_id: policyId } }
  });
}

export async function deleteIncentivePolicy(policyId: string): Promise<void> {
  await client.DELETE("/v1/payroll/incentive-policies/{policy_id}", {
    params: { path: { policy_id: policyId } }
  });
}

export async function createSalaryBaseGroup(data: { name: string; slug: string; description?: string }): Promise<SalaryBaseGroup> {
  const { data: resData } = await client.POST<SalaryBaseGroup>("/v1/salary-bases", {
    body: data
  });
  return resData;
}

export async function fetchSalaryBaseGroups(): Promise<{ groups: SalaryBaseGroup[], members: SalaryBaseGroupMember[] }> {
  const { data: resData } = await client.GET<{ groups: SalaryBaseGroup[], members: SalaryBaseGroupMember[] }>("/v1/salary-bases");
  return resData;
}

export async function assignComponentToSalaryBase(baseId: string, componentId: string): Promise<void> {
  await client.POST("/v1/salary-bases/{base_id}/members", {
    params: { path: { base_id: baseId } },
    body: { component_definition_id: componentId }
  });
}

export async function unassignComponentFromSalaryBase(baseId: string, componentId: string): Promise<void> {
  await client.DELETE("/v1/salary-bases/{base_id}/members/{component_id}", {
    params: { path: { base_id: baseId, component_id: componentId } }
  });
}

export async function fetchPayrollSettings(): Promise<PayrollSettings> {
  const { data: resData } = await client.GET<PayrollSettings>("/v1/payroll-settings");
  return resData;
}

export async function updatePayrollSettings(data: UpdatePayrollSettingsRequest): Promise<PayrollSettings> {
  const { data: resData } = await client.PATCH<PayrollSettings>("/v1/payroll-settings", {
    body: data
  });
  return resData;
}

export async function fetchPayrollSettingsHistory(): Promise<PayrollSettings[]> {
  const { data: resData } = await client.GET<PayrollSettings[]>("/v1/payroll-settings/history");
  return resData;
}

export async function fetchSalaryComponents(): Promise<SalaryComponentDefinition[]> {
  const { data: resData } = await client.GET<SalaryComponentDefinition[]>("/v1/salary-components");
  return resData;
}

export async function createSalaryComponent(
  data: CreateSalaryComponentRequest
): Promise<SalaryComponentDefinition> {
  const { data: resData } = await client.POST<SalaryComponentDefinition>("/v1/salary-components", {
    body: data
  });
  return resData;
}

export async function fetchEmployeeSalaryComponents(employmentId: string): Promise<EmployeeSalaryComponentDetail[]> {
  const { data: resData } = await fetch<EmployeeSalaryComponentDetail[]>("GET", "/v1/employments/{employment_id}/salary-components", {
    params: { path: { employment_id: employmentId } },
  });
  return resData;
}

export async function assignEmployeeSalaryComponent(
  employmentId: string,
  data: { amount: string; component_definition_id: string; effective_date: string; }
): Promise<EmployeeSalaryComponent> {
  const { data: resData } = await client.POST<EmployeeSalaryComponent>("/v1/employments/{employment_id}/salary-components", {
    params: { path: { employment_id: employmentId } },
    body: data
  });
  return resData;
}

export async function calculate_payroll(
  employeeId: string,
  year: number,
  month: number,
): Promise<Payslip> {
  const { data: resData } = await client.POST<Payslip>("/v1/employees/{employee_id}/calculate-payroll", {
    params: { 
      path: { employee_id: employeeId },
      query: { year, month }
    }
  });
  return resData;
}

export async function fetchEmployeePayslips(
  employeeId: string,
): Promise<Payslip[]> {
  const { data: resData } = await client.GET<Payslip[]>("/v1/employees/{employee_id}/payslips", {
    params: { path: { employee_id: employeeId } }
  });
  return resData;
}

export async function fetchEmployeePayrollConfigs(employeeId: string): Promise<ConfigRecord[]> {
  const { data: resData } = await fetch<ConfigRecord[]>("GET", "/v1/employees/{id}/payroll-config", {
    params: { path: { id: employeeId } },
  });
  return resData;
}

// Overtime Policies
export async function fetchOvertimePolicies(): Promise<OvertimePolicy[]> {
  const { data: resData } = await client.GET<{ data: OvertimePolicy[] }>("/v1/overtime/policies");
  return resData.data;
}

export async function fetchOvertimePolicy(policyId: string): Promise<[OvertimePolicy, OvertimeRule[]]> {
  const { data: resData } = await fetch<[OvertimePolicy, OvertimeRule[]]>("GET", "/v1/overtime/policies/{policy_id}", {
    params: { path: { policy_id: policyId } },
  });
  return resData;
}

export async function createOvertimePolicy(data: CreateOvertimePolicyRequest): Promise<OvertimePolicy> {
  const { data: resData } = await fetch<OvertimePolicy>("POST", "/v1/overtime/policies", {
    body: data,
  });
  return resData;
}

export async function addOvertimeRule(policyId: string, data: { days_of_week: number; is_holiday: boolean; multiplier: string; priority: number; time_end?: string | null; time_start?: string | null }): Promise<OvertimeRule> {
  const { data: resData } = await client.POST<OvertimeRule>("/v1/overtime/policies/{policy_id}/rules", {
    params: { path: { policy_id: policyId } },
    body: data
  });
  return resData;
}

export async function setEmployeePayrollConfig(employeeId: string, data: SetEmployeePayrollConfigRequest): Promise<EmployeePayrollConfig> {
  const { data: resData } = await client.POST<EmployeePayrollConfig>("/v1/employees/{id}/payroll-config", {
    params: { path: { id: employeeId } },
    body: data,
  });
  return resData;
}

export async function fetchDepartmentPayrollConfigs(deptId: string): Promise<DepartmentPayrollConfig[]> {
  const { data: resData } = await client.GET<DepartmentPayrollConfig[]>("/v1/departments/{id}/payroll-config", {
    params: { path: { id: deptId } },
  });
  return resData;
}

export async function setDepartmentPayrollConfig(deptId: string, data: SetDepartmentPayrollConfigRequest): Promise<DepartmentPayrollConfig> {
  const { data: resData } = await client.POST<DepartmentPayrollConfig>("/v1/departments/{id}/payroll-config", {
    params: { path: { id: deptId } },
    body: data,
  });
  return resData;
}

export async function calculateBatchPayslips(data: { employment_ids: string[]; year: number; month: number }): Promise<{ results: BatchCalculateResult[] }> {
  const { data: resData } = await client.POST<{ results: BatchCalculateResult[] }>("/v1/payroll/calculate-batch", {
    body: data
  });
  return resData;
}

export async function fetchWorkspacePayslips(year: number, month: number): Promise<Payslip[]> {
  const { data: resData } = await client.GET<Payslip[]>("/v1/payroll/payslips", {
    params: { 
      query: { year, month }
    }
  });
  return resData;
}

export async function assignOvertimePolicy(data: { assignee_id?: string | null; assignee_type: string; effective_from: string; policy_id: string }): Promise<void> {
  await client.POST("/v1/overtime/policies/assign", {
    body: data
  });
}

// --- Payroll Rules API ---

export async function fetchPayrollRules(): Promise<PayrollRuleDefinition[]> {
  const { data: resData } = await client.GET<PayrollRuleDefinition[]>("/v1/payroll-rules");
  return resData;
}

export async function createPayrollRule(data: CreatePayrollRuleDefinitionRequest): Promise<PayrollRuleDefinition> {
  const { data: resData } = await client.POST<PayrollRuleDefinition>("/v1/payroll-rules", {
    body: data
  });
  return resData;
}

export async function fetchPayrollRuleTiers(ruleId: string): Promise<PayrollRuleTier[]> {
  const { data: resData } = await client.GET<PayrollRuleTier[]>("/v1/payroll-rules/{rule_id}/tiers", {
    params: { path: { rule_id: ruleId } }
  });
  return resData;
}

export async function createPayrollRuleTier(ruleId: string, data: { fixed_amount?: string | null; max_amount?: string | null; min_amount: string; rate: string; tier_order: number }): Promise<PayrollRuleTier> {
  const { data: resData } = await client.POST<PayrollRuleTier>("/v1/payroll-rules/{rule_id}/tiers", {
    params: { path: { rule_id: ruleId } },
    body: data
  });
  return resData;
}

// --- Payroll Rule Assignments ---

export async function fetchPayrollRuleAssignments(ruleId: string): Promise<PayrollRuleAssignment[]> {
  const { data: resData } = await client.GET<PayrollRuleAssignment[]>("/v1/payroll-rules/{rule_id}/assignments", {
    params: { path: { rule_id: ruleId } }
  });
  return resData;
}

export async function createPayrollRuleAssignment(ruleId: string, data: { assignee_id?: string | null; assignee_type: string; effective_from: string; effective_to?: string | null; override_config?: unknown }): Promise<PayrollRuleAssignment> {
  const { data: resData } = await fetch<PayrollRuleAssignment>("POST", "/v1/payroll-rules/{rule_id}/assignments", {
    params: { path: { rule_id: ruleId } },
    body: data,
  });
  return resData;
}

export async function deletePayrollRuleAssignment(ruleId: string, assignmentId: string): Promise<void> {
  await client.DELETE("/v1/payroll-rules/{rule_id}/assignments/{assignment_id}", {
    params: { path: { rule_id: ruleId, assignment_id: assignmentId } }
  });
}
