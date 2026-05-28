import { client } from './client';
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
    const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/approval-workflows", {
      params: {
        path: { ws_id: workspaceId },
        query: entityType ? { entity_type: entityType } : undefined
      } as any
    });
    if (!resData) throw new Error("No response data");
    return resData as ApprovalWorkflow[];
  },

  getWorkflow: async (workspaceId: string, id: string): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/approval-workflows/{id}", {
      params: {
        path: { ws_id: workspaceId, id }
      }
    });
    if (!resData) throw new Error("No response data");
    return resData as ApprovalWorkflow;
  },

  createWorkflow: async (workspaceId: string, payload: CreateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/approval-workflows", {
      params: {
        path: { ws_id: workspaceId }
      },
      body: payload
    });
    if (!resData) throw new Error("No response data");
    return resData as ApprovalWorkflow;
  },

  updateWorkflow: async (workspaceId: string, id: string, payload: UpdateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.PUT("/v1/workspaces/{ws_id}/approval-workflows/{id}", {
      params: {
        path: { ws_id: workspaceId, id }
      },
      body: payload as any
    });
    if (!resData) throw new Error("No response data");
    return resData as ApprovalWorkflow;
  },

  deleteWorkflow: async (workspaceId: string, id: string): Promise<void> => {
    await client.DELETE("/v1/workspaces/{ws_id}/approval-workflows/{id}", {
      params: {
        path: { ws_id: workspaceId, id }
      }
    });
  },

  getApprovalHistory: async (workspaceId: string, entityType: string, entityId: string): Promise<ApprovalActionLog[]> => {
    const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/approval-workflows/history/{type}/{id}", {
      params: {
        path: { ws_id: workspaceId, type: entityType, id: entityId }
      }
    });
    if (!resData) throw new Error("No response data");
    return resData as ApprovalActionLog[];
  },

  processApproval: async (workspaceId: string, payload: {
    entity_type: string;
    entity_id: string;
    action: string;
    note?: string;
  }): Promise<void> => {
    await client.POST("/v1/workspaces/{ws_id}/approval-workflows/approve", {
      params: {
        path: { ws_id: workspaceId }
      },
      body: payload
    });
  }
};
