import type { OvertimeRuleV2, WorkPolicyVersionStatus } from "./payroll-overtime";

export type PayMethod = "monthly" | "hourly" | "shift";
export type MonthlyDenominator = "fixed" | "schedule";
export type PaidLeaveCreditMode = "standard_day" | "scheduled_minutes" | "unpaid";
export type RoundingPolicy = "none" | "floor" | "round" | "ceil";
export type EarlyArrivalStrategy = "allow" | "clamp" | "block";
export type OTAggregationMode = "aggregate" | "per_segment";

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
