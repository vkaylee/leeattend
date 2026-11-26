import type {
  CreatePayrollRuleDefinitionRequest,
  PayrollRuleAssignment,
  PayrollRuleDefinition,
  PayrollRuleTier,
} from "./types";
import type { PayrollFormulaRuleVersion } from "./types-payroll";

import { client, fetch } from "./client";

export async function fetchPayrollFormulaRules(): Promise<PayrollRuleDefinition[]> {
  const { data: response } = await fetch<PayrollRuleDefinition[]>("GET", "/api/v1/payroll/formulas");
  return response;
}

export async function fetchPayrollFormulaVersions(ruleId: string): Promise<PayrollFormulaRuleVersion[]> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion[]>("GET", "/api/v1/payroll/formulas/{rule_id}/versions", {
    params: { path: { rule_id: ruleId } },
  });
  return response;
}

export async function createPayrollFormulaVersion(
  ruleId: string,
  data: { effective_from: string; expression: string },
): Promise<PayrollFormulaRuleVersion> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion>("POST", "/api/v1/payroll/formulas/{rule_id}/versions", {
    params: { path: { rule_id: ruleId } },
    body: data,
  });
  return response;
}

export async function publishPayrollFormulaRule(ruleId: string): Promise<PayrollFormulaRuleVersion> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion>("POST", "/api/v1/payroll/formulas/{rule_id}/publish", {
    params: { path: { rule_id: ruleId } },
  });
  return response;
}

export async function retirePayrollFormulaRule(ruleId: string): Promise<PayrollFormulaRuleVersion> {
  const { data: response } = await fetch<PayrollFormulaRuleVersion>("POST", "/api/v1/payroll/formulas/{rule_id}/retire", {
    params: { path: { rule_id: ruleId } },
  });
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

export async function createPayrollRuleTier(
  ruleId: string,
  data: { fixed_amount?: string | null; max_amount?: string | null; min_amount: string; rate: string; tier_order: number },
): Promise<PayrollRuleTier> {
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

export async function createPayrollRuleAssignment(
  ruleId: string,
  data: { assignee_id?: string | null; assignee_type: string; effective_from: string; effective_to?: string | null; override_config?: unknown },
): Promise<PayrollRuleAssignment> {
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
