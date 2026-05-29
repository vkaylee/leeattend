import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { client } from "./client";

export interface AssignmentRuleConditions {
  department_id?: string;
  employee_id_numbers?: string[];
  tags?: string[];
}

export interface AssignmentRule {
  conditions: AssignmentRuleConditions;
  created_at: string;
  id: string;
  is_active: boolean;
  name: string;
  pattern_id: string;
  priority: number;
  updated_at: string;
  workspace_id: string;
}

export interface CreateAssignmentRuleRequest {
  conditions: AssignmentRuleConditions;
  name: string;
  pattern_id: string;
  priority: number;
}

export interface UpdateAssignmentRuleRequest {
  conditions?: AssignmentRuleConditions;
  is_active?: boolean;
  name?: string;
  pattern_id?: string;
  priority?: number;
}

export async function fetchAssignmentRules(wsId: string): Promise<AssignmentRule[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/assignment-rules", {
            params: { path: { ws_id: wsId } }
      });
    return resData as any;
}

export async function createAssignmentRule(wsId: string, data: CreateAssignmentRuleRequest): Promise<AssignmentRule> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/assignment-rules", {
            params: { path: { ws_id: wsId } },
          body: data });
    return resData as any;
}

export async function updateAssignmentRule(wsId: string, ruleId: string, data: UpdateAssignmentRuleRequest): Promise<AssignmentRule> {
  const { data: resData } = await client.PUT("/v1/workspaces/{ws_id}/assignment-rules/{rule_id}", {
            params: { path: { ws_id: wsId, rule_id: ruleId } },
          body: data });
    return resData as any;
}

export async function deleteAssignmentRule(wsId: string, ruleId: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/assignment-rules/{rule_id}", {
            params: { path: { ws_id: wsId, rule_id: ruleId } }
      });
}
