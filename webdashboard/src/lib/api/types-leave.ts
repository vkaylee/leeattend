export type LeaveDurationType = 'full_day' | 'first_half' | 'second_half' | 'custom_hours';

export interface EmployeeLeaveBalance {
  accrued_amount: number;
  carried_over: number;
  entitlement: number;
  is_paid?: boolean;
  leave_type?: string;
  leave_type_code?: string;
  leave_type_id?: string;
  leave_type_name?: string;
  remaining: number;
  used: number;
}

export interface LeavePolicy {
  created_at: string;
  description?: string;
  id: string;
  leave_type_id: string;
  name: string;
  rule_config: LeavePolicyConfig;
  updated_at: string;
}

export type AccrualFrequency = "monthly" | "quarterly" | "yearly";

export interface SeniorityBonus {
  bonus_amount?: number;
  months?: number;
  min_years?: number;
  extra_days?: number;
}

export interface LeavePolicyConfig {
  accrual_amount: number;
  accrual_timing: string;
  carry_over_limit?: number | null;
  frequency: AccrualFrequency;
  max_balance?: number | null;
  min_work_days_threshold?: number | null;
  reset_month?: number | null;
  seniority_bonus?: SeniorityBonus[];
}

export interface LeavePolicyAssignmentDetail {
  assignee_id?: string | null;
  assignee_type: string;
  created_at: string;
  effective_from: string;
  effective_to: string | null;
  id: string;
  leave_type_name: string;
  policy_id: string;
  policy_name: string;
}
