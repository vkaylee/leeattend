import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";

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
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-rules`);
  return response.json() as Promise<AssignmentRule[]>;
}

export async function createAssignmentRule(wsId: string, data: CreateAssignmentRuleRequest): Promise<AssignmentRule> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-rules`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  return response.json() as Promise<AssignmentRule>;
}

export async function updateAssignmentRule(wsId: string, ruleId: string, data: UpdateAssignmentRuleRequest): Promise<AssignmentRule> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-rules/${ruleId}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  return response.json() as Promise<AssignmentRule>;
}

export async function deleteAssignmentRule(wsId: string, ruleId: string): Promise<void> {
  await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-rules/${ruleId}`, {
    method: "DELETE",
  });
}
