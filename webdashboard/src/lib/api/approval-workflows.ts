import { apiClient } from './client';

export type ApproverType = 'department_manager' | 'role' | 'permission' | 'specific_user';

export interface ApprovalWorkflowStep {
  id: string;
  workflow_id: string;
  step_order: number;
  step_name: string;
  approver_type: ApproverType;
  approver_config: any;
  action_required: string;
  can_skip: boolean;
}

export interface ApprovalWorkflow {
  id: string;
  workspace_id: string;
  entity_type: string;
  name: string;
  description?: string;
  total_steps: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
  steps?: ApprovalWorkflowStep[];
}

export interface WorkflowStepRequest {
  step_order: number;
  step_name: string;
  approver_type: ApproverType;
  approver_config: any;
  action_required: string;
  can_skip: boolean;
}

export interface CreateWorkflowRequest {
  entity_type: string;
  name: string;
  description?: string;
  is_active: boolean;
  total_steps: number;
  steps: WorkflowStepRequest[];
}

export interface UpdateWorkflowRequest {
  name?: string;
  description?: string;
  is_active?: boolean;
}

export interface ApprovalActionLog {
  id: string;
  workspace_id: string;
  entity_type: string;
  entity_id: string;
  step_order: number;
  step_name?: string;
  acted_by_user_id: string;
  action: string;
  note?: string;
  acted_at: string;
}

export const approvalWorkflowsApi = {
  listWorkflows: async (workspaceId: string, entityType?: string): Promise<ApprovalWorkflow[]> => {
    const params = new URLSearchParams();
    if (entityType) params.append('entity_type', entityType);
    
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/approval-workflows?${params.toString()}`);
    return response.data;
  },

  getWorkflow: async (workspaceId: string, id: string): Promise<ApprovalWorkflow> => {
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/approval-workflows/${id}`);
    return response.data;
  },

  createWorkflow: async (workspaceId: string, payload: CreateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const response = await apiClient.post(`/api/workspaces/${workspaceId}/approval-workflows`, payload);
    return response.data;
  },

  updateWorkflow: async (workspaceId: string, id: string, payload: UpdateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const response = await apiClient.put(`/api/workspaces/${workspaceId}/approval-workflows/${id}`, payload);
    return response.data;
  },

  deleteWorkflow: async (workspaceId: string, id: string): Promise<void> => {
    await apiClient.delete(`/api/workspaces/${workspaceId}/approval-workflows/${id}`);
  },

  getApprovalHistory: async (workspaceId: string, entityType: string, entityId: string): Promise<ApprovalActionLog[]> => {
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/approval-workflows/history/${entityType}/${entityId}`);
    return response.data;
  },

  processApproval: async (workspaceId: string, payload: {
    entity_type: string;
    entity_id: string;
    action: string;
    note?: string;
  }): Promise<void> => {
    await apiClient.post(`/api/workspaces/${workspaceId}/approval-workflows/approve`, payload);
  }
};
