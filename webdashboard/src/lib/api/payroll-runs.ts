import type {
  BatchCalculateResult,
  CreateSalaryComponentRequest,
  EmployeeSalaryComponent,
  EmployeeSalaryComponentDetail,
  IncentiveTier,
  PayrollIncentivePolicyResponse,
  Payslip,
  SalaryComponentDefinition,
} from "./types";
import type { PayrollCalculationRun, PayrollInput } from "./types-payroll";

import { client, fetch } from "./client";

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

export async function fetchIncentivePolicies(): Promise<PayrollIncentivePolicyResponse[]> {
  const { data: resData } = await client.GET<PayrollIncentivePolicyResponse[]>("/api/v1/payroll/incentive-policies");
  return resData;
}

export async function createIncentivePolicy(data: {
  base_amount: number;
  effective_from: string;
  name: string;
  payout_type: string;
  tiers: Omit<IncentiveTier, "id" | "policy_id">[];
}): Promise<void> {
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

export async function fetchPayrollImpactBatch(data: {
  after_run_id: string;
  before_run_id: string;
  employment_ids: string[];
  month: number;
  year: number;
}): Promise<PayrollImpactBatchResponse> {
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
