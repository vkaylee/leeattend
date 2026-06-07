import { client } from "./client";

export interface AssignmentRuleConditions {
  department_id?: string;
  employee_id_numbers?: string[];
  employer_id?: string;
  location_id?: string;
  tags?: string[];
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
  workspace_id: string;
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

export async function fetchAssignmentRules(wsId: string): Promise<AssignmentRule[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/assignment-rules", {
            params: { path: { ws_id: wsId } }
      });
  if (!resData) throw new Error("No response data");
  return resData as unknown as AssignmentRule[];
}

export async function createAssignmentRule(wsId: string, data: CreateAssignmentRuleRequest): Promise<AssignmentRule> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/assignment-rules", {
            params: { path: { ws_id: wsId } },
          body: data });
  if (!resData) throw new Error("No response data");
  return resData as unknown as AssignmentRule;
}

export async function updateAssignmentRule(wsId: string, ruleId: string, data: UpdateAssignmentRuleRequest): Promise<AssignmentRule> {
  const { data: resData } = await client.PUT("/v1/workspaces/{ws_id}/assignment-rules/{rule_id}", {
            params: { path: { ws_id: wsId, rule_id: ruleId } },
          body: data });
  if (!resData) throw new Error("No response data");
  return resData as unknown as AssignmentRule;
}

export async function deleteAssignmentRule(wsId: string, ruleId: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/assignment-rules/{rule_id}", {
            params: { path: { ws_id: wsId, rule_id: ruleId } }
      });
}
