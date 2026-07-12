import type { EmployeeSalaryComponent, IncentiveEvaluationResult, SalaryComponentWithDefinition } from "./types";

export interface ConfigRecord {
  created_at?: string;
  effective_date: string;
  hours_per_day: number;
  id?: string;
  is_virtual?: boolean;
  max_ot_hours_per_day?: number | null;
  max_ot_hours_per_month?: number | null;
  std_working_days: number;
}

export type EmployeeSalaryComponentDetail = EmployeeSalaryComponent & {
  name: string;
  code: string;
};

export interface PayrollSegment {
  actual_work_days: string | number;
  amount: string | number;
  applied_base_salary: string | number;
  applied_hours_per_day: string | number;
  applied_std_days: string | number;
  department_id?: string | null;
  end_date: string;
  id: string;
  paid_work_days: string | number;
  reason: string;
  start_date: string;
  std_days_source?: string | null;
}

export type Payslip = import("@/types/api-generated").components["schemas"]["Payslip"] & {
  incentives?: IncentiveEvaluationResult[];
  segments?: PayrollSegment[];
  salary_components?: SalaryComponentWithDefinition[];
};

export interface RuleCondition {
  metric: string;
  operator: string;
  value: number;
}

export type PayrollRuleAssignment = Omit<import("@/types/api-generated").components["schemas"]["PayrollRuleAssignment"], "override_config"> & {
  override_config?: unknown;
};
