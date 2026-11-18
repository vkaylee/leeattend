import type { EmployeeSalaryComponent, IncentiveEvaluationResult, SalaryComponentWithDefinition } from "./types";

export interface ConfigRecord {
  created_at?: string;
  effective_date: string;
  hours_per_day: string | number;
  id?: string;
  is_virtual?: boolean;
  max_ot_hours_per_day?: number | null;
  max_ot_hours_per_month?: number | null;
  std_working_days: string | number;
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

export interface PayrollFormulaRuleVersion {
  created_at: string;
  created_by: string;
  effective_from: string;
  expression: string;
  published_at?: string | null;
  published_by?: string | null;
  rule_id: string;
  status: string;
  version: number;
  version_id: string;
}

export interface PayrollInput {
  employment_id: string;
  id?: string;
  metric_code: string;
  source_reference?: string | null;
  status?: string;
  unit: string;
  validation_message?: string | null;
  value: string;
}

export interface PayrollCalculationRun {
  calculation_trace: unknown;
  created_at: string;
  created_by?: string | null;
  employment_id: string;
  engine_version: string;
  formula_version_id?: string | null;
  id: string;
  input_snapshot: unknown;
  output_snapshot: unknown;
  payslip_id?: string | null;
  period_month: number;
  period_year: number;
}

export interface Payslip {
  base_pay_amount: string;
  created_at: string;
  deductions_amount: string;
  employment_id: string;
  holiday_work_premium_amount: string;
  id: string;
  incentives?: IncentiveEvaluationResult[];
  net_pay_amount: string;
  overtime_pay_amount: string;
  overtime_units: number;
  period_end: string;
  period_start: string;
  salary_components?: SalaryComponentWithDefinition[];
  segments?: PayrollSegment[];
  standard_units: number;
  status: string;
  updated_at: string;
}

export interface RuleCondition {
  metric: string;
  operator: string;
  value: number;
}

export interface PayrollRuleAssignment {
  assignee_id?: string | null;
  assignee_type: string;
  created_at: string;
  effective_from: string;
  effective_to?: string | null;
  id: string;
  override_config?: unknown;
  rule_definition_id: string;
}

// ── Typed payroll config request/response types ─────────────────────────
// These replace the previous Record<string, unknown> cop-outs with
// proper interfaces that match the generated OpenAPI schema.

/** Payroll config for a single employee (returned by GET) — alias for ConfigRecord. */
export type EmployeePayrollConfig = ConfigRecord;

/** Request body for setting employee payroll config. */
export interface SetEmployeePayrollConfigRequest {
  effective_date: string;
  hours_per_day: string | number;
  std_working_days: string | number;
}

/** Request body for creating an overtime policy with rules. */
export interface CreateOvertimePolicyRequest {
  description?: string | null;
  name: string;
  rules?: CreateOvertimeRuleRequest[];
}

export interface CreateOvertimeRuleRequest {
  days_of_week: number;
  is_holiday: boolean;
  multiplier: string;
  priority: number;
  time_end?: string | null;
  time_start?: string | null;
}

/** Request body for roster upsert.
 *  employment_id is the employee in the URL path, so body may not include it. */
export interface UpsertRosterRequest {
  date: string;
  employment_id?: string;
  notes?: string | null;
  shift_id?: string | null;
}
