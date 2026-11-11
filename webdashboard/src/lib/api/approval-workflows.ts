import type {
  ApproverType,
  ApprovalWorkflow,
  CreateWorkflowRequest,
  UpdateWorkflowRequest,
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

export const approvalWorkflowsApi = {
  listWorkflows: async (entityType?: string): Promise<ApprovalWorkflow[]> => {
    const { data: resData } = await client.GET<{ data: ApprovalWorkflow[] }>("/api/v1/approval-workflows", {
      params: {
        query: entityType ? { entity_type: entityType } : undefined,
      },
    });
    return resData.data;
  },

  getWorkflow: async (id: string): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.GET<ApprovalWorkflow>("/api/v1/approval-workflows/{id}", {
      params: {
        path: { id },
      },
    });
    return resData;
  },

  createWorkflow: async (payload: CreateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.POST<ApprovalWorkflow>("/api/v1/approval-workflows", {
      body: payload,
    });
    return resData;
  },

  updateWorkflow: async (id: string, payload: UpdateWorkflowRequest): Promise<ApprovalWorkflow> => {
    const { data: resData } = await client.PUT<ApprovalWorkflow>("/api/v1/approval-workflows/{id}", {
      params: { path: { id } },
      body: payload,
    });
    return resData;
  },

  deleteWorkflow: async (id: string): Promise<void> => {
    await client.DELETE("/api/v1/approval-workflows/{id}", {
      params: {
        path: { id },
      },
    });
  },

  getApprovalHistory: async (entityType: string, entityId: string): Promise<ApprovalActionLog[]> => {
    const { data: resData } = await client.GET<ApprovalActionLog[]>("/api/v1/approval-workflows/history/{type}/{id}", {
      params: {
        path: { type: entityType, id: entityId },
      },
    });
    return resData;
  },

  processApproval: async (payload: {
    entity_type: string;
    entity_id: string;
    action: string;
    note?: string;
  }): Promise<void> => {
    await client.POST("/api/v1/approval-workflows/approve", {
      body: payload,
    });
  },
};
