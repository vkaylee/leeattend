/* eslint-disable max-lines -- API wrapper module: 300 lines of exported functions; splitting would scatter imports */
import type { PayrollCalculationRun, PayrollFormulaRuleVersion, PayrollInput } from "./types-payroll";
import { fetchWorkingPatterns } from "./working-patterns";
import type { AttendancePolicyVersion } from "./attendance-rules";

export interface WorkPolicyGroupVersion {
  id: string;
  group_id: string;
  name: string;
  work_credit_policy_id?: string;
  pay_basis_policy_id?: string;
  attendance_policy_id?: string;
  working_pattern_id?: string;
  minutes_per_credit: number;
  pay_method: "monthly" | "hourly" | "shift";
  monthly_denominator?: "fixed" | "schedule";
  monthly_standard_days?: string;
  rate_component_definition_id?: string | null;
  overtime_policy_id: string;
  effective_from: string;
  effective_to?: string;
}

export async function fetchWorkPolicyReferenceOptions(): Promise<{ attendance: { id: string; label: string }[]; patterns: { id: string; label: string }[]; overtime: { id: string; label: string }[]; salaryComponents: { id: string; label: string }[] }> {
  const [attendance, patterns, overtime, salaryComponents] = await Promise.all([
    client.GET<AttendancePolicyVersion[]>("/api/v1/attendance/rules/versions").then((response) => response.data),
    fetchWorkingPatterns(),
    fetchOvertimePolicies(),
    fetchSalaryComponents(),
  ]);
  return {
    attendance: (attendance ?? []).map((item) => ({ id: item.id, label: `${item.id} (${item.valid_from})` })),
    patterns: patterns.map((item) => ({ id: item.id, label: item.name })),
    overtime: overtime.map((item) => ({ id: item.id, label: item.name })),
    salaryComponents: salaryComponents.map((item) => ({ id: item.id, label: `${item.name} (${item.code})` })),
  };
}

export interface WorkPolicyValidationIssue {
  code: string;
  field: string;
  severity: "blocker" | "warning";
  message: string;
}

export interface WorkPolicyValidationResponse {
  version_id: string;
  valid: boolean;
  blockers: WorkPolicyValidationIssue[];
  warnings: WorkPolicyValidationIssue[];
  checked_at: string;
}

export interface WorkPolicyMembership {
  id: string;
  employment_id: string;
  group_id: string;
  effective_from: string;
  effective_to?: string;
}

export interface CreateWorkPolicyGroupVersionRequest {
  name: string;
  minutes_per_credit: number;
  attendance_policy_id: string;
  working_pattern_id: string;
  pay_method: "monthly" | "hourly" | "shift";
  monthly_denominator?: "fixed" | "schedule";
  monthly_standard_days?: string;
  rate_component_definition_id?: string | null;
  overtime_policy_id: string;
  effective_from: string;
  effective_to?: string;
}

interface WorkPolicyGroupListResponse {
  data: WorkPolicyGroupVersion[];
  total: number;
  page: number;
  limit: number;
}

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
  OvertimePolicy,
  OvertimeRule,
  PayrollRuleDefinition,
  PayrollRuleTier,
  BatchCalculateResult,
  ConfigRecord,
  CreateOvertimePolicyRequest,
  EmployeePayrollConfig,
  SetEmployeePayrollConfigRequest,
  IncentiveTier,
} from "./types";

export async function fetchIncentivePolicies(): Promise<PayrollIncentivePolicyResponse[]> {
  const { data: resData } = await client.GET<PayrollIncentivePolicyResponse[]>("/api/v1/payroll/incentive-policies");
  return resData;
}

export async function createIncentivePolicy(data: { base_amount: number; effective_from: string; name: string; payout_type: string; tiers: Omit<IncentiveTier, 'id' | 'policy_id'>[] }): Promise<void> {
  await client.POST("/api/v1/payroll/incentive-policies", {
    body: data,
  });
}

export async function cloneIncentivePolicy(policyId: string): Promise<void> {
  await client.POST("/api/v1/payroll/incentive-policies/{policy_id}/clone", {
    params: { path: { policy_id: policyId } }
  });
}

export async function deleteIncentivePolicy(policyId: string): Promise<void> {
  await client.DELETE("/api/v1/payroll/incentive-policies/{policy_id}", {
    params: { path: { policy_id: policyId } }
  });
}

export async function fetchWorkPolicyGroups(): Promise<WorkPolicyGroupVersion[]> {
  const { data } = await fetch<WorkPolicyGroupListResponse>("GET", "/api/v1/payroll/work-policy-groups", {
    params: { query: { page: 1, limit: 100 } },
  });
  return data?.data ?? [];
}

export async function fetchWorkPolicyGroupHistory(groupId: string): Promise<WorkPolicyGroupVersion[]> {
  const { data } = await fetch<WorkPolicyGroupListResponse>("GET", "/api/v1/payroll/work-policy-groups/{group_id}/history", {
    params: { path: { group_id: groupId }, query: { page: 1, limit: 100 } },
  });
  return data?.data ?? [];
}

export async function validateWorkPolicyGroupVersion(groupId: string, versionId: string): Promise<WorkPolicyValidationResponse> {
  const { data } = await fetch<WorkPolicyValidationResponse>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/validate", {
    params: { path: { group_id: groupId, version_id: versionId } },
  });
  return data;
}

export async function createWorkPolicyMembership(groupId: string, body: { employment_id: string; effective_from: string; effective_to?: string }): Promise<WorkPolicyMembership> {
  const { data } = await fetch<WorkPolicyMembership>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/memberships", {
    params: { path: { group_id: groupId } }, body,
  });
  return data;
}

export async function fetchWorkPolicyMemberships(groupId: string): Promise<WorkPolicyMembership[]> {
  const { data } = await fetch<WorkPolicyMembership[]>("GET", "/api/v1/payroll/work-policy-groups/{group_id}/memberships", {
    params: { path: { group_id: groupId } },
  });
  return data ?? [];
}

export async function resolveWorkPolicy(employmentId: string, asOf: string): Promise<WorkPolicyGroupVersion> {
  const { data } = await fetch<WorkPolicyGroupVersion>("GET", "/api/v1/employments/{employment_id}/work-policy", {
    params: { path: { employment_id: employmentId }, query: { as_of: asOf } },
  });
  return data;
}

export async function createWorkPolicyGroupVersion(groupId: string, body: CreateWorkPolicyGroupVersionRequest): Promise<WorkPolicyGroupVersion> {
  const { data } = await fetch<WorkPolicyGroupVersion>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions", {
    params: { path: { group_id: groupId } }, body,
  });
  return data;
}

export async function fetchPayrollSettings(): Promise<PayrollSettings> {
  const { data: resData } = await client.GET<PayrollSettings>("/api/v1/payroll-settings");
  return resData;
}

export async function updatePayrollSettings(data: UpdatePayrollSettingsRequest): Promise<PayrollSettings> {
  const { data: resData } = await client.PATCH<PayrollSettings>("/api/v1/payroll-settings", {
    body: data
  });
  return resData;
}

export async function fetchPayrollSettingsHistory(): Promise<PayrollSettings[]> {
  const { data: resData } = await client.GET<PayrollSettings[]>("/api/v1/payroll-settings/history");
  return resData;
}

export async function fetchSalaryComponents(): Promise<SalaryComponentDefinition[]> {
  const { data: resData } = await client.GET<SalaryComponentDefinition[]>("/api/v1/salary-components");
  return resData;
}

export async function createSalaryComponent(
  data: CreateSalaryComponentRequest
): Promise<SalaryComponentDefinition> {
  const { data: resData } = await client.POST<SalaryComponentDefinition>("/api/v1/salary-components", {
    body: data
  });
  return resData;
}

export async function fetchEmployeeSalaryComponents(employmentId: string): Promise<EmployeeSalaryComponentDetail[]> {
  const { data: resData } = await fetch<EmployeeSalaryComponentDetail[]>("GET", "/api/v1/employments/{employment_id}/salary-components", {
    params: { path: { employment_id: employmentId } },
  });
  return resData;
}

export async function assignEmployeeSalaryComponent(
  employmentId: string,
  data: { amount: string; component_definition_id: string; effective_date: string; }
): Promise<EmployeeSalaryComponent> {
  const { data: resData } = await client.POST<EmployeeSalaryComponent>("/api/v1/employments/{employment_id}/salary-components", {
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
  const { data: resData } = await client.POST<Payslip>("/api/v1/employees/{employee_id}/calculate-payroll", {
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
  const { data: resData } = await client.GET<Payslip[]>("/api/v1/employees/{employee_id}/payslips", {
    params: { path: { employee_id: employeeId } }
  });
  return resData;
}

export async function fetchEmployeePayrollConfigs(employeeId: string): Promise<ConfigRecord[]> {
  const { data: resData } = await fetch<ConfigRecord[]>("GET", "/api/v1/employees/{id}/payroll-config", {
    params: { path: { id: employeeId } },
  });
  return resData;
}

// Overtime Policies
export async function fetchOvertimePolicies(): Promise<OvertimePolicy[]> {
  const { data: resData } = await client.GET<{ data: OvertimePolicy[] }>("/api/v1/overtime/policies");
  return resData.data;
}

export async function fetchOvertimePolicy(policyId: string): Promise<[OvertimePolicy, OvertimeRule[]]> {
  const { data: resData } = await fetch<[OvertimePolicy, OvertimeRule[]]>("GET", "/api/v1/overtime/policies/{policy_id}", {
    params: { path: { policy_id: policyId } },
  });
  return resData;
}

export async function createOvertimePolicy(data: CreateOvertimePolicyRequest): Promise<OvertimePolicy> {
  const { data: resData } = await fetch<OvertimePolicy>("POST", "/api/v1/overtime/policies", {
    body: data,
  });
  return resData;
}

export async function addOvertimeRule(policyId: string, data: { days_of_week: number; is_holiday: boolean; multiplier: string; priority: number; time_end?: string | null; time_start?: string | null }): Promise<OvertimeRule> {
  const { data: resData } = await client.POST<OvertimeRule>("/api/v1/overtime/policies/{policy_id}/rules", {
    params: { path: { policy_id: policyId } },
    body: data
  });
  return resData;
}

export async function setEmployeePayrollConfig(employeeId: string, data: SetEmployeePayrollConfigRequest): Promise<EmployeePayrollConfig> {
  const { data: resData } = await client.POST<EmployeePayrollConfig>("/api/v1/employees/{id}/payroll-config", {
    params: { path: { id: employeeId } },
    body: data,
  });
  return resData;
}

export async function calculateBatchPayslips(data: { employment_ids: string[]; year: number; month: number }): Promise<{ results: BatchCalculateResult[] }> {
  const { data: resData } = await client.POST<{ results: BatchCalculateResult[] }>("/api/v1/payroll/calculate-batch", {
    body: data
  });
  return resData;
}

export async function fetchPayslips(year: number, month: number): Promise<Payslip[]> {
  const { data: resData } = await client.GET<Payslip[]>("/api/v1/payroll/payslips", {
    params: { 
      query: { year, month }
    }
  });
  return resData;
}

export async function assignOvertimePolicy(data: { assignee_id?: string | null; assignee_type: string; effective_from: string; policy_id: string }): Promise<void> {
  await client.POST("/api/v1/overtime/policies/assign", {
    body: data
  });
}

// --- Payroll Rules API ---

export async function fetchPayrollFormulaRules(): Promise<PayrollRuleDefinition[]> {
  const { data: response } = await fetch<PayrollRuleDefinition[]>("GET", "/api/v1/payroll/formulas");
  return response;
}

export async function fetchPayrollFormulaVersions(ruleId: string): Promise<PayrollFormulaRuleVersion[]> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion[]>("GET", "/api/v1/payroll/formulas/{rule_id}/versions", { params: { path: { rule_id: ruleId } } });
  return response;
}

export async function createPayrollFormulaVersion(ruleId: string, data: { expression: string; effective_from: string }): Promise<PayrollFormulaRuleVersion> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion>("POST", "/api/v1/payroll/formulas/{rule_id}/versions", { params: { path: { rule_id: ruleId } }, body: data });
  return response;
}

export async function publishPayrollFormulaRule(ruleId: string): Promise<PayrollFormulaRuleVersion> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion>("POST", "/api/v1/payroll/formulas/{rule_id}/publish", { params: { path: { rule_id: ruleId } } });
  return response;
}

export async function retirePayrollFormulaRule(ruleId: string): Promise<PayrollFormulaRuleVersion> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion>("POST", "/api/v1/payroll/formulas/{rule_id}/retire", { params: { path: { rule_id: ruleId } } });
  return response;
}

export interface PayrollImpactResponse {
  employment_id: string;
  period_year: number;
  period_month: number;
  before_run_id: string;
  after_run_id: string;
  computed: boolean;
  stale: boolean;
  stale_reasons?: string[];
  reason?: string;
  base_pay_delta: string;
  overtime_pay_delta: string;
  deductions_delta: string;
  net_pay_delta: string;
  standard_units_delta: number;
  overtime_units_delta: number;
}

export interface PayrollImpactBatchItem {
  employment_id: string;
  computed: boolean;
  stale: boolean;
  stale_reasons?: string[];
  reason?: string;
  delta?: PayrollImpactResponse;
}

export interface PayrollImpactBatchResponse {
  total: number;
  computed: number;
  affected: number;
  uncomputable: number;
  items: PayrollImpactBatchItem[];
}
export async function fetchPayrollCalculationRuns(employmentId: string, year: number, month: number): Promise<PayrollCalculationRun[]> {
  const { data: response } = await fetch<PayrollCalculationRun[]>("GET", "/api/v1/employments/{id}/payroll/calculation-runs", { params: { path: { id: employmentId }, query: { year, month } } });
  return response;
}

export async function fetchPayrollImpact(employmentId: string, year: number, month: number, beforeRunId: string, afterRunId: string): Promise<PayrollImpactResponse> {
  const { data: response } = await fetch<PayrollImpactResponse>("GET", "/api/v1/employments/{id}/payroll/impact", {
    params: { path: { id: employmentId }, query: { year, month, before_run_id: beforeRunId, after_run_id: afterRunId } },
  });
  return response;
}

export async function fetchPayrollImpactBatch(data: { employment_ids: string[]; year: number; month: number; before_run_id: string; after_run_id: string }): Promise<PayrollImpactBatchResponse> {
  const { data: response } = await fetch<PayrollImpactBatchResponse>("POST", "/api/v1/payroll/impact/batch", { body: data });
  return response;
}

export async function simulatePayroll(employeeId: string, year: number, month: number): Promise<Payslip> {
  const { data: response } = await client.POST<Payslip>("/api/v1/employees/{id}/simulate-payroll", {
    params: { path: { id: employeeId }, query: { year, month } },
  });
  return response;
}

export async function simulatePayrollWithPolicy(employeeId: string, year: number, month: number, policyVersionId: string): Promise<Payslip> {
  const { data: response } = await client.POST<Payslip>("/api/v1/employees/{id}/simulate-payroll", {
    params: { path: { id: employeeId }, query: { year, month, policy_version_id: policyVersionId } },
  });
  return response;
}

export async function fetchPayrollInputs(batchId: string): Promise<PayrollInput[]> {
  const { data: response } = await fetch<PayrollInput[]>("GET", "/api/v1/payroll/inputs/{batch_id}", { params: { path: { batch_id: batchId } } });
  return response;
}

export async function fetchPayrollRules(): Promise<PayrollRuleDefinition[]> {
  const { data: resData } = await client.GET<PayrollRuleDefinition[]>("/api/v1/payroll-rules");
  return resData;
}

export async function createPayrollRule(data: CreatePayrollRuleDefinitionRequest): Promise<PayrollRuleDefinition> {
  const { data: resData } = await client.POST<PayrollRuleDefinition>("/api/v1/payroll-rules", {
    body: data
  });
  return resData;
}

export async function fetchPayrollRuleTiers(ruleId: string): Promise<PayrollRuleTier[]> {
  const { data: resData } = await client.GET<PayrollRuleTier[]>("/api/v1/payroll-rules/{rule_id}/tiers", {
    params: { path: { rule_id: ruleId } }
  });
  return resData;
}

export async function createPayrollRuleTier(ruleId: string, data: { fixed_amount?: string | null; max_amount?: string | null; min_amount: string; rate: string; tier_order: number }): Promise<PayrollRuleTier> {
  const { data: resData } = await client.POST<PayrollRuleTier>("/api/v1/payroll-rules/{rule_id}/tiers", {
    params: { path: { rule_id: ruleId } },
    body: data
  });
  return resData;
}

// --- Payroll Rule Assignments ---

export async function fetchPayrollRuleAssignments(ruleId: string): Promise<PayrollRuleAssignment[]> {
  const { data: resData } = await client.GET<PayrollRuleAssignment[]>("/api/v1/payroll-rules/{rule_id}/assignments", {
    params: { path: { rule_id: ruleId } }
  });
  return resData;
}

export async function createPayrollRuleAssignment(ruleId: string, data: { assignee_id?: string | null; assignee_type: string; effective_from: string; effective_to?: string | null; override_config?: unknown }): Promise<PayrollRuleAssignment> {
  const { data: resData } = await fetch<PayrollRuleAssignment>("POST", "/api/v1/payroll-rules/{rule_id}/assignments", {
    params: { path: { rule_id: ruleId } },
    body: data,
  });
  return resData;
}

export async function deletePayrollRuleAssignment(ruleId: string, assignmentId: string): Promise<void> {
  await client.DELETE("/api/v1/payroll-rules/{rule_id}/assignments/{assignment_id}", {
    params: { path: { rule_id: ruleId, assignment_id: assignmentId } }
  });
}
