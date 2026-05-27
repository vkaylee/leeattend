import { apiClient } from './client';
import { 
  ApproverType, 
  ApprovalWorkflowStep, 
  ApprovalWorkflow, 
  CreateWorkflowRequest, 
  ApprovalActionLog 
} from './types';

export type {
  ApproverType,
  ApprovalWorkflowStep,
  ApprovalWorkflow,
  CreateWorkflowRequest,
  ApprovalActionLog
};

export interface WorkflowStepRequest {
  action_required: string;
  approver_config: any;
  approver_type: ApproverType;
  can_skip: boolean;
  step_name: string;
  step_order: number;
}

export interface UpdateWorkflowRequest {
  description?: string;
  is_active?: boolean;
  name?: string;
}

export const approvalWorkflowsApi = {
  listWorkflows: async (workspaceId: string, entityType?: string): Promise<ApprovalWorkflow[]> => {
    const params = new URLSearchParams();
    if (entityType) params.append('entity_type', entityType);
    
    const response = await apiClient.get(`/workspaces/${workspaceId}/approval-workflows?${params.toString()}`);
    return response.data;
  },

  getWorkflow: async (workspaceId: string, id: string): Promise<ApprovalWorkflow> => {
    const response = await apiClient.get(`/workspaces/${workspaceId}/approval-workflows/${id}`);
    return response.data;
  },

  createWorkflow: async (workspaceId: string, payload: CreateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const response = await apiClient.post(`/workspaces/${workspaceId}/approval-workflows`, payload);
    return response.data;
  },

  updateWorkflow: async (workspaceId: string, id: string, payload: UpdateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const response = await apiClient.put(`/workspaces/${workspaceId}/approval-workflows/${id}`, payload);
    return response.data;
  },

  deleteWorkflow: async (workspaceId: string, id: string): Promise<void> => {
    await apiClient.delete(`/workspaces/${workspaceId}/approval-workflows/${id}`);
  },

  getApprovalHistory: async (workspaceId: string, entityType: string, entityId: string): Promise<ApprovalActionLog[]> => {
    const response = await apiClient.get(`/workspaces/${workspaceId}/approval-workflows/history/${entityType}/${entityId}`);
    return response.data;
  },

  processApproval: async (workspaceId: string, payload: {
    entity_type: string;
    entity_id: string;
    action: string;
    note?: string;
  }): Promise<void> => {
    await apiClient.post(`/workspaces/${workspaceId}/approval-workflows/approve`, payload);
  }
};
