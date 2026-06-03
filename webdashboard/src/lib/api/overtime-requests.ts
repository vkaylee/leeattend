import type { ApprovalActionLog } from './types';

import { client } from './client';

const httpClient = client as unknown as {
  GET: (url: string, options?: Record<string, unknown>) => Promise<{ data?: unknown; error?: unknown }>;
  POST: (url: string, options?: Record<string, unknown>) => Promise<{ data?: unknown; error?: unknown }>;
  PUT: (url: string, options?: Record<string, unknown>) => Promise<{ data?: unknown; error?: unknown }>;
  DELETE: (url: string, options?: Record<string, unknown>) => Promise<{ data?: unknown; error?: unknown }>;
};

export type OtRequestStatus = 
  | 'draft' 
  | 'pending_approval' 
  | 'approved' 
  | 'partially_accepted' 
  | 'active' 
  | 'completed' 
  | 'cancelled' 
  | 'rejected';

export type ConsentStatus = 'pending' | 'accepted' | 'declined';

export interface OvertimeRequestEmployee {
  actual_ot_minutes?: number;
  consent_at?: string;
  consent_status: ConsentStatus;
  decline_reason?: string;
  employee_id: string;
  employee_id_number?: string;
  // Included from joined tables
  full_name?: string;
  id: string;
}

export interface OvertimeRequest {
  created_at: string;
  current_step: number;
  department_id?: string;
  employees?: OvertimeRequestEmployee[];
  id: string;
  planned_date: string;
  planned_end_time?: string;
  planned_minutes: number;
  planned_start_time?: string;
  quota_warning?: Record<string, unknown> | null;
  reason: string;
  requested_by_user_id: string;
  status: OtRequestStatus;
  updated_at: string;
  workflow_id?: string;
  workspace_id: string;
}

export interface OvertimeRequestDetail extends OvertimeRequest {
  approval_history?: ApprovalActionLog[];
}

export interface MyOvertimeAssignment {
  employee_statuses?: {
    employee_id: string;
    status: ConsentStatus;
  }[];
  id: string;
  my_employee_id: string;
  planned_date: string;
  planned_end_time: string;
  planned_start_time: string;
  reason?: string | null;
  source_type: 'employee_request' | 'manager_assignment';
  status: OtRequestStatus;
}

export interface CreateOvertimeRequestPayload {
  department_id?: string;
  employees: string[];
  planned_date: string;
  planned_end_time?: string;
  planned_minutes: number;
  planned_start_time?: string;
  reason: string;
}

export interface QuotaSummary {
  employee_id: string;
  monthly_limit: number | null;
  monthly_used: number;
  yearly_limit: number | null;
  yearly_used: number;
}

export const overtimeRequestsApi = {
  listRequests: async (workspaceId: string, params?: Record<string, unknown>): Promise<OvertimeRequest[]> => {
    const { data } = await client.GET("/v1/workspaces/{ws_id}/overtime/requests", {
      params: {
        path: { ws_id: workspaceId },
        query: params as unknown as undefined,
      },
    });
    if (!data) throw new Error("No response data");
    return data as unknown as OvertimeRequest[];
  },

  getRequest: async (workspaceId: string, id: string): Promise<OvertimeRequestDetail> => {
    const { data } = await client.GET("/v1/workspaces/{ws_id}/overtime/requests/{id}", {
      params: {
        path: { ws_id: workspaceId, id },
      },
    });
    if (!data) throw new Error("No response data");
    return data as unknown as OvertimeRequestDetail;
  },

  createRequest: async (workspaceId: string, payload: CreateOvertimeRequestPayload): Promise<OvertimeRequest> => {
    const { data } = await client.POST("/v1/workspaces/{ws_id}/overtime/requests", {
      params: {
        path: { ws_id: workspaceId },
      },
      body: payload,
    });
    if (!data) throw new Error("No response data");
    return data as unknown as OvertimeRequest;
  },

  submitRequest: async (workspaceId: string, id: string): Promise<void> => {
    await client.POST("/v1/workspaces/{ws_id}/overtime/requests/{id}/submit", {
      params: {
        path: { ws_id: workspaceId, id },
      },
    });
  },

  cancelRequest: async (workspaceId: string, id: string): Promise<void> => {
    await httpClient.PUT("/v1/workspaces/{ws_id}/overtime/requests/{id}/cancel", {
      params: {
        path: { ws_id: workspaceId, id },
      },
    });
  },

  employeeConsent: async (workspaceId: string, id: string, payload: {
    employee_id: string;
    status: ConsentStatus;
    decline_reason?: string;
  }): Promise<void> => {
    await client.POST("/v1/workspaces/{ws_id}/overtime/requests/{id}/consent/{emp_id}", {
      params: {
        path: { ws_id: workspaceId, id, emp_id: payload.employee_id },
      },
      body: {
              status: payload.status,
              decline_reason: payload.decline_reason,
            },
    });
  },

  getMyAssignments: async (employeeId: string): Promise<MyOvertimeAssignment[]> => {
    const { data } = await httpClient.GET("/v1/workspaces/{ws_id}/payroll/overtime/assignments", {
      params: {
        path: { employeeId },
      },
    });
    if (!data) throw new Error("No response data");
    return data as MyOvertimeAssignment[];
  },

  getWorkspaceQuotaSummary: async (workspaceId: string, month: string): Promise<QuotaSummary[]> => {
    const { data } = await httpClient.GET("/v1/workspaces/{ws_id}/overtime/quota/summary", {
      params: {
        path: { ws_id: workspaceId },
        query: { month },
      },
    });
    if (!data) throw new Error("No response data");
    return data as QuotaSummary[];
  },

  getEmployeeQuota: async (workspaceId: string, employeeId: string, date: string): Promise<QuotaSummary> => {
    const { data } = await httpClient.GET("/v1/workspaces/{ws_id}/overtime/quota/{employee_id}", {
      params: {
        path: { ws_id: workspaceId, employee_id: employeeId },
        query: { date },
      },
    });
    if (!data) throw new Error("No response data");
    return data as QuotaSummary;
  }
};
