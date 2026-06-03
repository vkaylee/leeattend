import type { paths } from '@/types/api-generated';

import type { 
  ApproverType, 
  ApprovalWorkflow, 
  CreateWorkflowRequest, 
  ApprovalActionLog 
} from './types';

import { client } from './client';

export interface WorkflowStepRequest {
  action_required: string;
  approver_config: Record<string, unknown>;
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
    const { data: resData } = await client.GET("/workspaces/{ws_id}/approval-workflows", {
      params: {
        path: { ws_id: workspaceId },
        query: (entityType ? { entity_type: entityType } : undefined) as unknown as paths["/workspaces/{ws_id}/approval-workflows"]["get"]["parameters"]["query"]
      }
    });
    if (!resData) throw new Error("No response data");
    return resData;
  },

  getWorkflow: async (workspaceId: string, id: string): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.GET("/workspaces/{ws_id}/approval-workflows/{id}", {
      params: {
        path: { ws_id: workspaceId, id }
      }
    });
    if (!resData) throw new Error("No response data");
    return resData;
  },

  createWorkflow: async (workspaceId: string, payload: CreateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.POST("/workspaces/{ws_id}/approval-workflows", {
      params: {
        path: { ws_id: workspaceId }
      },
      body: payload
    });
    if (!resData) throw new Error("No response data");
    return resData;
  },

  updateWorkflow: async (workspaceId: string, id: string, payload: UpdateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.PUT("/workspaces/{ws_id}/approval-workflows/{id}", {
      params: {
        path: { ws_id: workspaceId, id }
      },
      body: payload as unknown as paths["/workspaces/{ws_id}/approval-workflows/{id}"]["put"]["requestBody"]["content"]["application/json"]
    });
    if (!resData) throw new Error("No response data");
    return resData;
  },

  deleteWorkflow: async (workspaceId: string, id: string): Promise<void> => {
    await client.DELETE("/workspaces/{ws_id}/approval-workflows/{id}", {
      params: {
        path: { ws_id: workspaceId, id }
      }
    });
  },

  getApprovalHistory: async (workspaceId: string, entityType: string, entityId: string): Promise<ApprovalActionLog[]> => {
    const { data: resData } = await client.GET("/workspaces/{ws_id}/approval-workflows/history/{type}/{id}", {
      params: {
        path: { ws_id: workspaceId, type: entityType, id: entityId }
      }
    });
    if (!resData) throw new Error("No response data");
    return resData;
  },

  processApproval: async (workspaceId: string, payload: {
    entity_type: string;
    entity_id: string;
    action: string;
    note?: string;
  }): Promise<void> => {
    await client.POST("/workspaces/{ws_id}/approval-workflows/approve", {
      params: {
        path: { ws_id: workspaceId }
      },
      body: payload
    });
  }
};
