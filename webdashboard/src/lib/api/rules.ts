import { client } from "./client";

export interface AssignmentRuleConditions {
  department_id?: string | null;
  employee_id_numbers?: string[] | null;
  employer_id?: string | null;
  location_id?: string | null;
  tags?: string[] | null;
}

export interface AssignmentRule {
  conditions: AssignmentRuleConditions;
  created_at: string;
  end_date?: string | null;
  id: string;
  is_active: boolean;
  name: string;
  pattern_id: string;
  priority: number;
  start_date?: string | null;
  updated_at: string;
}

export interface CreateAssignmentRuleRequest {
  conditions: AssignmentRuleConditions;
  end_date?: string | null;
  name: string;
  pattern_id: string;
  priority: number;
  start_date?: string | null;
}

export interface UpdateAssignmentRuleRequest {
  conditions?: AssignmentRuleConditions;
  end_date?: string | null;
  is_active?: boolean;
  name?: string;
  pattern_id?: string;
  priority?: number;
  start_date?: string | null;
}

export async function fetchAssignmentRules(): Promise<AssignmentRule[]> {
  const { data: resData } = await client.GET<AssignmentRule[]>("/v1/assignment-rules");
  return resData;
}

export async function createAssignmentRule(data: CreateAssignmentRuleRequest): Promise<AssignmentRule> {
  const { data: resData } = await client.POST<AssignmentRule>("/v1/assignment-rules", { body: data });
  return resData;
}

export async function updateAssignmentRule(ruleId: string, data: UpdateAssignmentRuleRequest): Promise<AssignmentRule> {
  const { data: resData } = await client.PATCH<AssignmentRule>("/v1/assignment-rules/{rule_id}", {
    params: { path: { rule_id: ruleId } },
    body: data,
  });
  return resData;
}

export async function deleteAssignmentRule(ruleId: string): Promise<void> {
  await client.DELETE("/v1/assignment-rules/{rule_id}", {
    params: { path: { rule_id: ruleId } },
  });
}
