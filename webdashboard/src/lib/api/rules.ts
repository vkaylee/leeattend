import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { apiClient } from "./client";

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
  const { data: resData } = await apiClient.get<AssignmentRule[]>(`/v1/workspaces/${wsId}/assignment-rules`);
    return resData;
}

export async function createAssignmentRule(wsId: string, data: CreateAssignmentRuleRequest): Promise<AssignmentRule> {
  const { data: resData } = await apiClient.post<AssignmentRule>(`/v1/workspaces/${wsId}/assignment-rules`, data);
    return resData;
}

export async function updateAssignmentRule(wsId: string, ruleId: string, data: UpdateAssignmentRuleRequest): Promise<AssignmentRule> {
  const { data: resData } = await apiClient.patch<AssignmentRule>(`/v1/workspaces/${wsId}/assignment-rules/${ruleId}`, data);
    return resData;
}

export async function deleteAssignmentRule(wsId: string, ruleId: string): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/assignment-rules/${ruleId}`);
}
