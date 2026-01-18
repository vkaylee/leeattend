import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";

export interface AssignmentRuleConditions {
  department_id?: string;
  employee_id_numbers?: string[];
  tags?: string[];
}

export interface AssignmentRule {
  id: string;
  workspace_id: string;
  pattern_id: string;
  name: string;
  priority: number;
  conditions: AssignmentRuleConditions;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateAssignmentRuleRequest {
  pattern_id: string;
  name: string;
  priority: number;
  conditions: AssignmentRuleConditions;
}

export interface UpdateAssignmentRuleRequest {
  pattern_id?: string;
  name?: string;
  priority?: number;
  conditions?: AssignmentRuleConditions;
  is_active?: boolean;
}

export async function fetchAssignmentRules(wsId: string): Promise<AssignmentRule[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-rules`);
  return response.json();
}

export async function createAssignmentRule(wsId: string, data: CreateAssignmentRuleRequest): Promise<AssignmentRule> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-rules`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  return response.json();
}

export async function updateAssignmentRule(wsId: string, ruleId: string, data: UpdateAssignmentRuleRequest): Promise<AssignmentRule> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-rules/${ruleId}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  return response.json();
}

export async function deleteAssignmentRule(wsId: string, ruleId: string): Promise<void> {
  await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-rules/${ruleId}`, {
    method: "DELETE",
  });
}
