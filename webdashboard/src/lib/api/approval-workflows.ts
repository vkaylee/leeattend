import type {
  ApproverType,
  ApprovalWorkflow,
  CreateWorkflowRequest,
  ApprovalActionLog,
  ApproverConfig,
} from './types';

import { client } from './client';

export interface WorkflowStepRequest {
  action_required: string;
  approver_config: ApproverConfig;
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
        query: entityType ? { entity_type: entityType } : undefined
      }
    });
    return resData.data;
  },

  getWorkflow: async (workspaceId: string, id: string): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.GET("/workspaces/{ws_id}/approval-workflows/{id}", {
      params: {
        path: { ws_id: workspaceId, id }
      }
    });
    return resData;
  },

  createWorkflow: async (workspaceId: string, payload: CreateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.POST("/workspaces/{ws_id}/approval-workflows", {
      params: {
        path: { ws_id: workspaceId }
      },
      body: payload
    });
    return resData;
  },

  updateWorkflow: async (workspaceId: string, id: string, payload: UpdateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.PUT("/workspaces/{ws_id}/approval-workflows/{id}", {
      params: { path: { ws_id: workspaceId, id } },
      // @ts-expect-error — schema expects CreateWorkflowRequest but backend accepts partial UpdateWorkflowRequest (TODO: fix OpenAPI schema)
      body: payload,
    });
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
