import { client } from './client';
import { ApprovalActionLog } from './types';

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
  quota_warning?: any;
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
  listRequests: async (workspaceId: string, params?: any): Promise<OvertimeRequest[]> => {
    const { data } = await client.GET("/workspaces/{ws_id}/overtime/requests", {
      params: {
        path: { ws_id: workspaceId },
        query: params,
      },
    });
    if (!data) throw new Error("No response data");
    return data as unknown as OvertimeRequest[];
  },

  getRequest: async (workspaceId: string, id: string): Promise<OvertimeRequestDetail> => {
    const { data } = await client.GET("/workspaces/{ws_id}/overtime/requests/{id}", {
      params: {
        path: { ws_id: workspaceId, id },
      },
    });
    if (!data) throw new Error("No response data");
    return data as unknown as OvertimeRequestDetail;
  },

  createRequest: async (workspaceId: string, payload: CreateOvertimeRequestPayload): Promise<OvertimeRequest> => {
    const { data } = await client.POST("/workspaces/{ws_id}/overtime/requests", {
      params: {
        path: { ws_id: workspaceId },
      },
      body: payload as any,
    });
    if (!data) throw new Error("No response data");
    return data as unknown as OvertimeRequest;
  },

  submitRequest: async (workspaceId: string, id: string): Promise<void> => {
    await client.POST("/workspaces/{ws_id}/overtime/requests/{id}/submit", {
      params: {
        path: { ws_id: workspaceId, id },
      },
    });
  },

  cancelRequest: async (workspaceId: string, id: string): Promise<void> => {
    await client.POST("/workspaces/{ws_id}/overtime/requests/{id}/cancel" as any, {
      params: {
        path: { ws_id: workspaceId, id } as any,
      },
    });
  },

  employeeConsent: async (workspaceId: string, id: string, payload: {
    employee_id: string;
    status: ConsentStatus;
    decline_reason?: string;
  }): Promise<void> => {
    await client.POST("/workspaces/{ws_id}/overtime/requests/{id}/consent/{emp_id}", {
      params: {
        path: { ws_id: workspaceId, id, emp_id: payload.employee_id },
      },
      body: {
        status: payload.status,
        decline_reason: payload.decline_reason,
      } as any,
    });
  },

  getMyAssignments: async (employeeId: string): Promise<OvertimeRequest[]> => {
    const { data } = await client.GET("/employees/{employeeId}/overtime/assignments" as any, {
      params: {
        path: { employeeId } as any,
      },
    });
    if (!data) throw new Error("No response data");
    return data as unknown as OvertimeRequest[];
  },

  getWorkspaceQuotaSummary: async (workspaceId: string, month: string): Promise<QuotaSummary[]> => {
    const { data } = await client.GET("/workspaces/{ws_id}/overtime/quota/summary" as any, {
      params: {
        path: { ws_id: workspaceId } as any,
        query: { month } as any,
      },
    });
    if (!data) throw new Error("No response data");
    return data as unknown as QuotaSummary[];
  },

  getEmployeeQuota: async (workspaceId: string, employeeId: string, date: string): Promise<QuotaSummary> => {
    const { data } = await client.GET("/workspaces/{ws_id}/overtime/quota/{employeeId}" as any, {
      params: {
        path: { ws_id: workspaceId, employeeId } as any,
        query: { date } as any,
      },
    });
    if (!data) throw new Error("No response data");
    return data as unknown as QuotaSummary;
  }
};
