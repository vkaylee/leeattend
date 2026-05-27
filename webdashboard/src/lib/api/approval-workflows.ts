import { apiClient } from './client';

export type ApproverType = 'department_manager' | 'role' | 'permission' | 'specific_user';

export interface ApprovalWorkflowStep {
  action_required: string;
  approver_config: any;
  approver_type: ApproverType;
  can_skip: boolean;
  id: string;
  step_name: string;
  step_order: number;
  workflow_id: string;
}

export interface ApprovalWorkflow {
  created_at: string;
  description?: string;
  entity_type: string;
  id: string;
  is_active: boolean;
  name: string;
  steps?: ApprovalWorkflowStep[];
  total_steps: number;
  updated_at: string;
  workspace_id: string;
}

export interface WorkflowStepRequest {
  action_required: string;
  approver_config: any;
  approver_type: ApproverType;
  can_skip: boolean;
  step_name: string;
  step_order: number;
}

export interface CreateWorkflowRequest {
  description?: string;
  entity_type: string;
  is_active: boolean;
  name: string;
  steps: WorkflowStepRequest[];
  total_steps: number;
}

export interface UpdateWorkflowRequest {
  description?: string;
  is_active?: boolean;
  name?: string;
}

export interface ApprovalActionLog {
  acted_at: string;
  acted_by_user_id: string;
  action: string;
  entity_id: string;
  entity_type: string;
  id: string;
  note?: string;
  step_name?: string;
  step_order: number;
  workspace_id: string;
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
