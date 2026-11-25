/* eslint-disable max-lines -- API wrapper module for payroll, work policies, and overtime versions */
import type { PayrollCalculationRun, PayrollFormulaRuleVersion, PayrollInput } from "./types-payroll";

import { client, fetch } from "./client";
import {
  Payslip,
  SalaryComponentDefinition,
  EmployeeSalaryComponent,
  EmployeeSalaryComponentDetail,
  CreateSalaryComponentRequest,
  CreatePayrollRuleDefinitionRequest,
  PayrollIncentivePolicyResponse,
  PayrollRuleAssignment,
  PayrollRuleDefinition,
  PayrollRuleTier,
  BatchCalculateResult,
  IncentiveTier,
} from "./types";
import { fetchWorkingPatterns } from "./working-patterns";

// --- Work Policy v2 Types ---

export type WorkPolicyVersionStatus = "draft" | "published" | "archived";
export type PayMethod = "monthly" | "hourly" | "shift";
export type MonthlyDenominator = "fixed" | "schedule";
export type PaidLeaveCreditMode = "standard_day" | "scheduled_minutes" | "unpaid";
export type RoundingPolicy = "none" | "floor" | "round" | "ceil";
export type EarlyArrivalStrategy = "allow" | "clamp" | "block";
export type OTAggregationMode = "aggregate" | "per_segment";
export type OvertimeDayType = "weekday" | "weekend" | "holiday" | "any";

export interface OvertimeRuleV2 {
  day_type: OvertimeDayType;
  days_of_week: number;
  multiplier: string;
  priority: number;
  time_end?: string | null;
  time_start?: string | null;
}

export interface OvertimePolicyV2 {
  created_at: string;
  description?: string | null;
  id: string;
  latest_published_version_id?: string | null;
  latest_published_version_no?: number | null;
  name: string;
  updated_at: string;
}

export interface OvertimePolicyVersion {
  created_at: string;
  created_by: string;
  daily_cap_minutes?: number | null;
  id: string;
  monthly_cap_hours?: string | null;
  policy_id: string;
  published_at?: string | null;
  published_by?: string | null;
  row_version: number;
  rules: OvertimeRuleV2[];
  status: WorkPolicyVersionStatus;
  updated_at: string;
  version_no: number;
  yearly_cap_hours?: string | null;
}

export interface OvertimePolicyDetail {
  policy: OvertimePolicyV2;
  versions: OvertimePolicyVersion[];
}

export interface WorkPolicyVersionFields {
  break_auto_deduct: boolean;
  break_is_overtime: boolean;
  early_arrival_strategy: EarlyArrivalStrategy;
  early_ot_enabled: boolean;
  early_ot_max_minutes?: number | null;
  early_ot_qualification_grace_minutes: number;
  effective_from: string;
  effective_to?: string | null;
  late_arrival_grace_minutes: number;
  minutes_per_credit: number;
  monthly_denominator?: MonthlyDenominator | null;
  monthly_standard_days?: string | null;
  ot_aggregation_mode: OTAggregationMode;
  overtime_policy_version_id: string;
  paid_leave_credit_mode: PaidLeaveCreditMode;
  pay_method: PayMethod;
  rate_component_definition_id?: string | null;
  rounding_policy: RoundingPolicy;
  work_unit_minutes: number;
  working_pattern_id?: string | null;
}

export interface WorkPolicyVersion extends WorkPolicyVersionFields {
  created_at: string;
  created_by: string;
  group_code: string;
  group_id: string;
  group_name: string;
  id: string;
  published_at?: string | null;
  published_by?: string | null;
  row_version: number;
  status: WorkPolicyVersionStatus;
  updated_at: string;
  version_no: number;
}

export interface WorkPolicyGroupSummary {
  active_membership_count: number;
  archived_at?: string | null;
  code: string;
  created_at: string;
  current_published?: WorkPolicyVersion | null;
  description?: string | null;
  id: string;
  latest_draft?: WorkPolicyVersion | null;
  name: string;
  updated_at: string;
}

export interface WorkPolicyValidationIssue {
  code: string;
  field: string;
  message: string;
  severity: "blocker" | "warning";
}

export interface WorkPolicyValidationResponse {
  blockers: WorkPolicyValidationIssue[];
  checked_at: string;
  valid: boolean;
  version_id: string;
  warnings: WorkPolicyValidationIssue[];
}

export interface WorkPolicyMembership {
  created_at?: string;
  effective_from: string;
  effective_to?: string | null;
  employment_relationship_id: string;
  group_id: string;
  id: string;
}

export interface TransferWorkPolicyMembershipResponse {
  closed_membership?: WorkPolicyMembership | null;
  new_membership: WorkPolicyMembership;
}

export interface ResolvedWorkPolicyResponse extends WorkPolicyVersionFields {
  daily_cap_minutes?: number | null;
  employment_relationship_id: string;
  group_code: string;
  group_id: string;
  group_name: string;
  membership_id: string;
  monthly_cap_hours?: string | null;
  overtime_policy_id: string;
  overtime_policy_version_no: number;
  overtime_rules: OvertimeRuleV2[];
  status: WorkPolicyVersionStatus;
  version_id: string;
  version_no: number;
  yearly_cap_hours?: string | null;
}

export interface SimulationTraceStep {
  expression: string;
  result: string;
  rule: string;
  step: string;
}

export interface WorkPolicyBatchSimulationItem {
  affected: boolean;
  baseline_gross?: string;
  baseline_net?: string;
  baseline_ot_pay?: string;
  candidate_gross?: string;
  candidate_net?: string;
  candidate_ot_pay?: string;
  employment_id?: string;
  employment_relationship_id: string;
  gross_delta?: string;
  input_fingerprint?: string;
  net_delta?: string;
  ot_pay_delta?: string;
  reason?: string;
  status: "computed" | "uncomputable";
  trace?: SimulationTraceStep[];
}

export interface WorkPolicyBatchSimulationResponse {
  affected: number;
  computed: number;
  evaluated_at: string;
  expires_at: string;
  group_id: string;
  input_fingerprint: string;
  items: WorkPolicyBatchSimulationItem[];
  period_month: number;
  period_year: number;
  stale_after_seconds: number;
  total: number;
  uncomputable: number;
  version_id: string;
}

export interface CreateWorkPolicyGroupRequest extends WorkPolicyVersionFields {
  code: string;
  description?: string | null;
  name: string;
}

export interface CreateWorkPolicyVersionRequest extends Partial<WorkPolicyVersionFields> {
  clone_from_version_id?: string;
}

export interface PatchWorkPolicyVersionRequest extends WorkPolicyVersionFields {
  expected_row_version: number;
}

export interface PublishWorkPolicyVersionRequest {
  auto_close_previous: boolean;
  expected_row_version: number;
  idempotency_key?: string;
}

interface PaginatedResponse<T> {
  data: T[];
  limit: number;
  page: number;
  total: number;
}

// --- Work Policy v2 APIs ---

export async function fetchWorkPolicyReferenceOptions(): Promise<{
  overtimeVersions: { id: string; label: string }[];
  patterns: { id: string; label: string }[];
  salaryComponents: { id: string; label: string }[];
}> {
  const [patterns, overtime, salaryComponents] = await Promise.all([
    fetchWorkingPatterns(),
    fetchOvertimePolicies(),
    fetchSalaryComponents(),
  ]);
  const overtimeVersions = overtime.flatMap((item) =>
    item.latest_published_version_id
      ? [{ id: item.latest_published_version_id, label: `${item.name} (v${item.latest_published_version_no ?? 1})` }]
      : [],
  );
  return {
    overtimeVersions,
    patterns: patterns.map((item) => ({ id: item.id, label: item.name })),
    salaryComponents: salaryComponents.map((item) => ({ id: item.id, label: `${item.name} (${item.code})` })),
  };
}

export async function fetchWorkPolicyGroups(): Promise<WorkPolicyGroupSummary[]> {
  const { data } = await fetch<PaginatedResponse<WorkPolicyGroupSummary>>("GET", "/api/v1/payroll/work-policy-groups", {
    params: { query: { page: 1, limit: 100 } },
  });
  return data?.data ?? [];
}

export async function createWorkPolicyGroup(body: CreateWorkPolicyGroupRequest): Promise<WorkPolicyVersion> {
  const { data } = await fetch<WorkPolicyVersion>("POST", "/api/v1/payroll/work-policy-groups", { body });
  return data;
}

export async function fetchWorkPolicyGroupVersions(groupId: string, status?: WorkPolicyVersionStatus): Promise<WorkPolicyVersion[]> {
  const { data } = await fetch<PaginatedResponse<WorkPolicyVersion>>("GET", "/api/v1/payroll/work-policy-groups/{group_id}/versions", {
    params: { path: { group_id: groupId }, query: { page: 1, limit: 100, ...(status ? { status } : {}) } },
  });
  return data?.data ?? [];
}

export async function createWorkPolicyVersion(groupId: string, body: CreateWorkPolicyVersionRequest): Promise<WorkPolicyVersion> {
  const { data } = await fetch<WorkPolicyVersion>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions", {
    params: { path: { group_id: groupId } },
    body,
  });
  return data;
}

export async function patchWorkPolicyVersion(groupId: string, versionId: string, body: PatchWorkPolicyVersionRequest): Promise<WorkPolicyVersion> {
  const { data } = await fetch<WorkPolicyVersion>("PATCH", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}", {
    params: { path: { group_id: groupId, version_id: versionId } },
    body,
  });
  return data;
}

export async function validateWorkPolicyVersion(groupId: string, versionId: string): Promise<WorkPolicyValidationResponse> {
  const { data } = await fetch<WorkPolicyValidationResponse>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/validate", {
    params: { path: { group_id: groupId, version_id: versionId } },
  });
  return data;
}

export async function publishWorkPolicyVersion(
  groupId: string,
  versionId: string,
  body: PublishWorkPolicyVersionRequest,
  idempotencyKey?: string,
): Promise<WorkPolicyVersion> {
  const { data } = await fetch<WorkPolicyVersion>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/publish", {
    params: { path: { group_id: groupId, version_id: versionId } },
    headers: idempotencyKey ? { "Idempotency-Key": idempotencyKey } : undefined,
    body,
  });
  return data;
}

export async function fetchWorkPolicyMemberships(groupId: string): Promise<WorkPolicyMembership[]> {
  const { data } = await fetch<WorkPolicyMembership[]>("GET", "/api/v1/payroll/work-policy-groups/{group_id}/memberships", {
    params: { path: { group_id: groupId } },
  });
  return data ?? [];
}

export async function createWorkPolicyMembership(
  groupId: string,
  body: { effective_from: string; effective_to?: string | null; employment_relationship_id: string },
): Promise<WorkPolicyMembership> {
  const { data } = await fetch<WorkPolicyMembership>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/memberships", {
    params: { path: { group_id: groupId } },
    body,
  });
  return data;
}

export async function transferWorkPolicyMembership(body: {
  effective_date: string;
  employment_relationship_id: string;
  target_group_id: string;
}): Promise<TransferWorkPolicyMembershipResponse> {
  const { data } = await fetch<TransferWorkPolicyMembershipResponse>("POST", "/api/v1/payroll/work-policy-memberships/transfer", { body });
  return data;
}

export async function resolveWorkPolicyByRelationship(relationshipId: string, asOf: string): Promise<ResolvedWorkPolicyResponse> {
  const { data } = await fetch<ResolvedWorkPolicyResponse>("GET", "/api/v1/employment-relationships/{relationship_id}/work-policy", {
    params: { path: { relationship_id: relationshipId }, query: { as_of: asOf } },
  });
  return data;
}

export async function simulateWorkPolicyVersionBatch(
  groupId: string,
  versionId: string,
  body: { employment_relationship_ids: string[]; month: number; year: number },
): Promise<WorkPolicyBatchSimulationResponse> {
  const { data } = await fetch<WorkPolicyBatchSimulationResponse>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/simulate", {
    params: { path: { group_id: groupId, version_id: versionId } },
    body,
  });
  return data;
}

// --- Overtime Policy v2 APIs ---

export async function fetchOvertimePolicies(): Promise<OvertimePolicyV2[]> {
  const { data: resData } = await client.GET<{ data: OvertimePolicyV2[] }>("/api/v1/overtime/policies");
  return resData?.data ?? [];
}

export async function fetchOvertimePolicy(policyId: string): Promise<OvertimePolicyDetail> {
  const { data: resData } = await fetch<OvertimePolicyDetail>("GET", "/api/v1/overtime/policies/{policy_id}", {
    params: { path: { policy_id: policyId } },
  });
  return resData;
}

export async function createOvertimePolicy(body: {
  daily_cap_minutes?: number | null;
  description?: string | null;
  monthly_cap_hours?: string | null;
  name: string;
  rules?: OvertimeRuleV2[];
  yearly_cap_hours?: string | null;
}): Promise<OvertimePolicyDetail> {
  const { data: resData } = await fetch<OvertimePolicyDetail>("POST", "/api/v1/overtime/policies", { body });
  return resData;
}

export async function createOvertimePolicyVersion(
  policyId: string,
  body: {
    daily_cap_minutes?: number | null;
    monthly_cap_hours?: string | null;
    rules: OvertimeRuleV2[];
    yearly_cap_hours?: string | null;
  },
): Promise<OvertimePolicyVersion> {
  const { data } = await fetch<OvertimePolicyVersion>("POST", "/api/v1/overtime/policies/{policy_id}/versions", {
    params: { path: { policy_id: policyId } },
    body,
  });
  return data;
}

export async function patchOvertimePolicyVersion(
  policyId: string,
  versionId: string,
  body: {
    daily_cap_minutes?: number | null;
    expected_row_version: number;
    monthly_cap_hours?: string | null;
    rules: OvertimeRuleV2[];
    yearly_cap_hours?: string | null;
  },
): Promise<OvertimePolicyVersion> {
  const { data } = await fetch<OvertimePolicyVersion>("PATCH", "/api/v1/overtime/policies/{policy_id}/versions/{version_id}", {
    params: { path: { policy_id: policyId, version_id: versionId } },
    body,
  });
  return data;
}

export async function publishOvertimePolicyVersion(
  policyId: string,
  versionId: string,
  expectedRowVersion: number,
): Promise<OvertimePolicyVersion> {
  const { data } = await fetch<OvertimePolicyVersion>("POST", "/api/v1/overtime/policies/{policy_id}/versions/{version_id}/publish", {
    params: { path: { policy_id: policyId, version_id: versionId } },
    body: { expected_row_version: expectedRowVersion },
  });
  return data;
}

// --- Incentives, Salary Components, Payslips, Rules, Formulas, Impact ---

export async function fetchIncentivePolicies(): Promise<PayrollIncentivePolicyResponse[]> {
  const { data: resData } = await client.GET<PayrollIncentivePolicyResponse[]>("/api/v1/payroll/incentive-policies");
  return resData;
}

export async function createIncentivePolicy(data: { base_amount: number; effective_from: string; name: string; payout_type: string; tiers: Omit<IncentiveTier, "id" | "policy_id">[] }): Promise<void> {
  await client.POST("/api/v1/payroll/incentive-policies", { body: data });
}

export async function cloneIncentivePolicy(policyId: string): Promise<void> {
  await client.POST("/api/v1/payroll/incentive-policies/{policy_id}/clone", { params: { path: { policy_id: policyId } } });
}

export async function deleteIncentivePolicy(policyId: string): Promise<void> {
  await client.DELETE("/api/v1/payroll/incentive-policies/{policy_id}", { params: { path: { policy_id: policyId } } });
}

export async function fetchSalaryComponents(): Promise<SalaryComponentDefinition[]> {
  const { data: resData } = await client.GET<SalaryComponentDefinition[]>("/api/v1/salary-components");
  return resData;
}

export async function createSalaryComponent(data: CreateSalaryComponentRequest): Promise<SalaryComponentDefinition> {
  const { data: resData } = await client.POST<SalaryComponentDefinition>("/api/v1/salary-components", { body: data });
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
  data: { amount: string; component_definition_id: string; effective_date: string },
): Promise<EmployeeSalaryComponent> {
  const { data: resData } = await client.POST<EmployeeSalaryComponent>("/api/v1/employments/{employment_id}/salary-components", {
    params: { path: { employment_id: employmentId } },
    body: data,
  });
  return resData;
}

export async function calculate_payroll(employeeId: string, year: number, month: number): Promise<Payslip> {
  const { data: resData } = await client.POST<Payslip>("/api/v1/employees/{employee_id}/calculate-payroll", {
    params: { path: { employee_id: employeeId }, query: { year, month } },
  });
  return resData;
}

export async function fetchEmployeePayslips(employeeId: string): Promise<Payslip[]> {
  const { data: resData } = await client.GET<Payslip[]>("/api/v1/employees/{employee_id}/payslips", {
    params: { path: { employee_id: employeeId } },
  });
  return resData;
}

export async function calculateBatchPayslips(data: { employment_ids: string[]; month: number; year: number }): Promise<{ results: BatchCalculateResult[] }> {
  const { data: resData } = await client.POST<{ results: BatchCalculateResult[] }>("/api/v1/payroll/calculate-batch", { body: data });
  return resData;
}

export async function fetchPayslips(year: number, month: number): Promise<Payslip[]> {
  const { data: resData } = await client.GET<Payslip[]>("/api/v1/payroll/payslips", {
    params: { query: { year, month } },
  });
  return resData;
}

export async function fetchPayrollFormulaRules(): Promise<PayrollRuleDefinition[]> {
  const { data: response } = await fetch<PayrollRuleDefinition[]>("GET", "/api/v1/payroll/formulas");
  return response;
}

export async function fetchPayrollFormulaVersions(ruleId: string): Promise<PayrollFormulaRuleVersion[]> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion[]>("GET", "/api/v1/payroll/formulas/{rule_id}/versions", { params: { path: { rule_id: ruleId } } });
  return response;
}

export async function createPayrollFormulaVersion(ruleId: string, data: { effective_from: string; expression: string }): Promise<PayrollFormulaRuleVersion> {
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
  after_run_id: string;
  base_pay_delta: string;
  before_run_id: string;
  computed: boolean;
  deductions_delta: string;
  employment_id: string;
  evaluated_at?: string;
  expires_at?: string;
  input_fingerprint?: string;
  net_pay_delta: string;
  overtime_pay_delta: string;
  overtime_units_delta: number;
  period_month: number;
  period_year: number;
  reason?: string;
  stale: boolean;
  stale_after_seconds?: number;
  stale_reasons?: string[];
  standard_units_delta: number;
}

export interface PayrollImpactBatchItem {
  computed: boolean;
  delta?: PayrollImpactResponse;
  employment_id: string;
  input_fingerprint?: string;
  reason?: string;
  stale: boolean;
  stale_reasons?: string[];
}

export interface PayrollImpactBatchResponse {
  affected: number;
  computed: number;
  evaluated_at?: string;
  expires_at?: string;
  items: PayrollImpactBatchItem[];
  stale_after_seconds?: number;
  total: number;
  uncomputable: number;
}

export async function fetchPayrollCalculationRuns(employmentId: string, year: number, month: number, limit = 50): Promise<PayrollCalculationRun[]> {
  const { data: response } = await fetch<PayrollCalculationRun[]>("GET", "/api/v1/employments/{id}/payroll/calculation-runs", {
    params: { path: { id: employmentId }, query: { year, month, limit } },
  });
  return response;
}

export async function fetchPayrollImpact(employmentId: string, year: number, month: number, beforeRunId: string, afterRunId: string): Promise<PayrollImpactResponse> {
  const { data: response } = await fetch<PayrollImpactResponse>("GET", "/api/v1/employments/{id}/payroll/impact", {
    params: { path: { id: employmentId }, query: { year, month, before_run_id: beforeRunId, after_run_id: afterRunId } },
  });
  return response;
}

export async function fetchPayrollImpactBatch(data: { after_run_id: string; before_run_id: string; employment_ids: string[]; month: number; year: number }): Promise<PayrollImpactBatchResponse> {
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
  const { data: resData } = await client.POST<PayrollRuleDefinition>("/api/v1/payroll-rules", { body: data });
  return resData;
}

export async function fetchPayrollRuleTiers(ruleId: string): Promise<PayrollRuleTier[]> {
  const { data: resData } = await client.GET<PayrollRuleTier[]>("/api/v1/payroll-rules/{rule_id}/tiers", {
    params: { path: { rule_id: ruleId } },
  });
  return resData;
}

export async function createPayrollRuleTier(ruleId: string, data: { fixed_amount?: string | null; max_amount?: string | null; min_amount: string; rate: string; tier_order: number }): Promise<PayrollRuleTier> {
  const { data: resData } = await client.POST<PayrollRuleTier>("/api/v1/payroll-rules/{rule_id}/tiers", {
    params: { path: { rule_id: ruleId } },
    body: data,
  });
  return resData;
}

export async function fetchPayrollRuleAssignments(ruleId: string): Promise<PayrollRuleAssignment[]> {
  const { data: resData } = await client.GET<PayrollRuleAssignment[]>("/api/v1/payroll-rules/{rule_id}/assignments", {
    params: { path: { rule_id: ruleId } },
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
    params: { path: { rule_id: ruleId, assignment_id: assignmentId } },
  });
}
