import type { paths } from "@/types/api-generated";

import { client } from "./client";
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
  ConfigRecord
} from "./types";

export async function fetchIncentivePolicies(wsId: string): Promise<PayrollIncentivePolicyResponse[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/payroll/incentive-policies", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createIncentivePolicy(wsId: string, data: { base_amount: number; effective_from: string; name: string; payout_type: string; tiers: Record<string, unknown>[] }): Promise<void> {
  await client.POST("/workspaces/{ws_id}/payroll/incentive-policies", {
    params: { path: { ws_id: wsId } },
    body: data as unknown as paths["/workspaces/{ws_id}/payroll/incentive-policies"]["post"]["requestBody"]["content"]["application/json"]
  });
}

export async function cloneIncentivePolicy(wsId: string, policyId: string): Promise<void> {
  await client.POST("/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone", {
    params: { path: { ws_id: wsId, policy_id: policyId } }
  });
}

export async function deleteIncentivePolicy(wsId: string, policyId: string): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}", {
    params: { path: { ws_id: wsId, policy_id: policyId } }
  });
}

export async function createSalaryBaseGroup(wsId: string, data: { name: string; slug: string; description?: string }): Promise<SalaryBaseGroup> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/salary-bases", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchSalaryBaseGroups(wsId: string): Promise<{ groups: SalaryBaseGroup[], members: SalaryBaseGroupMember[] }> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/salary-bases", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function assignComponentToSalaryBase(wsId: string, baseId: string, componentId: string): Promise<void> {
  await client.POST("/workspaces/{ws_id}/salary-bases/{base_id}/members", {
    params: { path: { ws_id: wsId, base_id: baseId } },
    body: { component_definition_id: componentId }
  });
}

export async function unassignComponentFromSalaryBase(wsId: string, baseId: string, componentId: string): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}", {
    params: { path: { ws_id: wsId, base_id: baseId, component_id: componentId } }
  });
}

export async function fetchPayrollSettings(wsId: string): Promise<PayrollSettings> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/payroll-settings", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function updatePayrollSettings(wsId: string, data: UpdatePayrollSettingsRequest): Promise<PayrollSettings> {
  const { data: resData } = await client.PATCH("/workspaces/{ws_id}/payroll-settings", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchPayrollSettingsHistory(wsId: string): Promise<PayrollSettings[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/payroll-settings/history", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchSalaryComponents(wsId: string): Promise<SalaryComponentDefinition[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/salary-components", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createSalaryComponent(
  wsId: string, 
  data: CreateSalaryComponentRequest
): Promise<SalaryComponentDefinition> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/salary-components", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchEmployeeSalaryComponents(wsId: string, employmentId: string): Promise<EmployeeSalaryComponentDetail[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/employments/{employment_id}/salary-components", {
    params: { path: { ws_id: wsId, employment_id: employmentId } }
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as EmployeeSalaryComponentDetail[];
}

export async function assignEmployeeSalaryComponent(
  wsId: string,
  employmentId: string,
  data: { amount: string; component_definition_id: string; effective_date: string; }
): Promise<EmployeeSalaryComponent> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/employments/{employment_id}/salary-components", {
    params: { path: { ws_id: wsId, employment_id: employmentId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function calculate_payroll(
  wsId: string,
  employeeId: string,
  year: number,
  month: number,
): Promise<Payslip> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/employees/{employee_id}/calculate-payroll", {
    params: { 
      path: { ws_id: wsId, employee_id: employeeId },
      query: { year, month }
    }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchEmployeePayslips(
  wsId: string,
  employeeId: string,
): Promise<Payslip[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/employees/{employee_id}/payslips", {
    params: { path: { ws_id: wsId, employee_id: employeeId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchEmployeePayrollConfigs(wsId: string, employeeId: string): Promise<ConfigRecord[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/employees/{id}/payroll-config", {
    params: { path: { ws_id: wsId, id: employeeId } }
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as ConfigRecord[];
}

// Overtime Policies
export async function fetchOvertimePolicies(wsId: string): Promise<OvertimePolicy[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/overtime/policies", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchOvertimePolicy(wsId: string, policyId: string): Promise<[OvertimePolicy, OvertimeRule[]]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/overtime/policies/{policy_id}", {
    params: { path: { ws_id: wsId, policy_id: policyId } }
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as [OvertimePolicy, OvertimeRule[]];
}

export async function createOvertimePolicy(wsId: string, data: Record<string, unknown>): Promise<OvertimePolicy> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/overtime/policies", {
    params: { path: { ws_id: wsId } },
    body: data as unknown as paths["/workspaces/{ws_id}/overtime/policies"]["post"]["requestBody"]["content"]["application/json"]
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function addOvertimeRule(wsId: string, policyId: string, data: { days_of_week: number; is_holiday: boolean; multiplier: string; priority: number; time_end?: string | null; time_start?: string | null }): Promise<OvertimeRule> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/overtime/policies/{policy_id}/rules", {
    params: { path: { ws_id: wsId, policy_id: policyId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function setEmployeePayrollConfig(wsId: string, employeeId: string, data: { effective_date: string; hours_per_day: string; std_working_days: string } & Record<string, unknown>): Promise<Record<string, unknown>> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/employees/{id}/payroll-config", {
    params: { path: { ws_id: wsId, id: employeeId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchDepartmentPayrollConfigs(wsId: string, deptId: string): Promise<Record<string, unknown>> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/departments/{id}/payroll-config", {
    params: { path: { ws_id: wsId, id: deptId } }
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as Record<string, unknown>;
}

export async function setDepartmentPayrollConfig(wsId: string, deptId: string, data: { effective_date: string; hours_per_day: string; std_working_days: string } & Record<string, unknown>): Promise<Record<string, unknown>> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/departments/{id}/payroll-config", {
    params: { path: { ws_id: wsId, id: deptId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function calculateBatchPayslips(wsId: string, data: { employment_ids: string[]; year: number; month: number }): Promise<{ results: BatchCalculateResult[] }> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/payroll/calculate-batch", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchWorkspacePayslips(wsId: string, year: number, month: number): Promise<Payslip[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/payroll/payslips", {
    params: { 
      path: { ws_id: wsId },
      query: { year, month }
    }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function assignOvertimePolicy(wsId: string, data: { assignee_id?: string | null; assignee_type: string; effective_from: string; policy_id: string }): Promise<void> {
  await client.POST("/workspaces/{ws_id}/overtime/policies/assign", {
    params: { path: { ws_id: wsId } },
    body: data
  });
}

// --- Payroll Rules API ---

export async function fetchPayrollRules(wsId: string): Promise<PayrollRuleDefinition[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/payroll-rules", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createPayrollRule(wsId: string, data: CreatePayrollRuleDefinitionRequest): Promise<PayrollRuleDefinition> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/payroll-rules", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchPayrollRuleTiers(wsId: string, ruleId: string): Promise<PayrollRuleTier[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers", {
    params: { path: { ws_id: wsId, rule_id: ruleId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createPayrollRuleTier(wsId: string, ruleId: string, data: { fixed_amount?: string | null; max_amount?: string | null; min_amount: string; rate: string; tier_order: number }): Promise<PayrollRuleTier> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers", {
    params: { path: { ws_id: wsId, rule_id: ruleId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

// --- Payroll Rule Assignments ---

export async function fetchPayrollRuleAssignments(wsId: string, ruleId: string): Promise<PayrollRuleAssignment[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments", {
    params: { path: { ws_id: wsId, rule_id: ruleId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createPayrollRuleAssignment(wsId: string, ruleId: string, data: { assignee_id?: string | null; assignee_type: string; effective_from: string; effective_to?: string | null; override_config?: unknown }): Promise<PayrollRuleAssignment> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments", {
    params: { path: { ws_id: wsId, rule_id: ruleId } },
    body: data as unknown as paths["/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments"]["post"]["requestBody"]["content"]["application/json"]
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function deletePayrollRuleAssignment(wsId: string, ruleId: string, assignmentId: string): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments/{assignment_id}", {
    params: { path: { ws_id: wsId, rule_id: ruleId, assignment_id: assignmentId } }
  });
}
