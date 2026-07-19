import type { components } from "@/types/api-generated";

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
  workspace_id: string;
}

export type AccrualFrequency = "monthly" | "quarterly" | "yearly";

export type SeniorityBonus = Omit<components["schemas"]["SeniorityBonus"], "bonus_amount" | "months"> & {
  bonus_amount?: number;
  months?: number;
  min_years?: number;
  extra_days?: number;
};

export type LeavePolicyConfig = Omit<components["schemas"]["LeavePolicyConfig"], "seniority_bonus"> & {
  seniority_bonus?: SeniorityBonus[];
};

export type LeavePolicyAssignmentDetail = Omit<components["schemas"]["LeavePolicyAssignmentDetail"], "effective_to"> & {
  effective_to: string | null;
};
