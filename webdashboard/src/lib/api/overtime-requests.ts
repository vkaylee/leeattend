import type { ApprovalActionLog, BehalfAssignment, CreateBehalfAssignmentRequest, PaginatedResponse } from './types';

import { client, fetch, fire } from './client';

export interface OvertimeRequestQueryParams {
  consent?: string;
  department_id?: string;
  employee_id?: string;
  limit?: number;
  page?: number;
  planned_date?: string;
  status?: OtRequestStatus;
  [key: string]: unknown;
}

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

export interface OvertimeQuotaWarning {
  employee_id: string;
  limit: number;
  message: string;
  type: 'daily' | 'monthly' | 'yearly';
  used: number;
}

export interface OvertimeRequest {
  attachment_file_ids?: string[] | null;
  created_at: string;
  current_step: number;
  department_id?: string;
  employees?: OvertimeRequestEmployee[];
  id: string;
  planned_date: string;
  planned_end_time?: string;
  planned_minutes: number;
  planned_start_time?: string;
  quota_warning?: OvertimeQuotaWarning | null;
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
  attachment_file_ids?: string[];
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
  listRequests: async (workspaceId: string, params?: OvertimeRequestQueryParams): Promise<PaginatedResponse<OvertimeRequest>> => {
    const { data } = await fetch<PaginatedResponse<OvertimeRequest>>("GET", "/v1/workspaces/{ws_id}/overtime/requests", {
      params: {
        path: { ws_id: workspaceId },
        query: params,
      },
    });
    return data;
  },

  getRequest: async (workspaceId: string, id: string): Promise<OvertimeRequestDetail> => {
    const { data } = await fetch<OvertimeRequestDetail>("GET", "/v1/workspaces/{ws_id}/overtime/requests/{id}", {
      params: { path: { ws_id: workspaceId, id } },
    });
    return data;
  },

  createRequest: async (workspaceId: string, payload: CreateOvertimeRequestPayload): Promise<OvertimeRequest> => {
    const { data } = await fetch<OvertimeRequest>("POST", "/v1/workspaces/{ws_id}/overtime/requests", {
      params: { path: { ws_id: workspaceId } },
      body: payload,
    });
    return data;
  },

  submitRequest: async (workspaceId: string, id: string): Promise<void> => {
    await client.POST("/v1/workspaces/{ws_id}/overtime/requests/{id}/submit", {
      params: {
        path: { ws_id: workspaceId, id },
      },
    });
  },

  cancelRequest: async (workspaceId: string, id: string): Promise<void> => {
    await fire("PUT", "/v1/workspaces/{ws_id}/overtime/requests/{id}/cancel", {
      params: {
        path: { ws_id: workspaceId, id },
      },
    });
  },

  employeeConsent: async (workspaceId: string, id: string, payload: {
    employment_id: string;
    status: ConsentStatus;
    decline_reason?: string;
  }): Promise<void> => {
    await client.POST("/v1/workspaces/{ws_id}/overtime/requests/{id}/consent/{emp_id}", {
      params: {
        path: { ws_id: workspaceId, id, emp_id: payload.employment_id },
      },
      body: {
              status: payload.status,
              decline_reason: payload.decline_reason,
            },
    });
  },

  getMyAssignments: async (employmentId: string): Promise<MyOvertimeAssignment[]> => {
    // `fetch<T>` return type is `{ data: T }` (Assertion), so TS flags the guard
    // below as "unnecessary". Kept deliberately: openapi-fetch can return
    // `{ data: undefined }` on 204/empty bodies, and the response typing is
    // erased by the generic cast inside the helper — the guard is the last line
    // of runtime defence against a missing payload.
    const { data } = await fetch<MyOvertimeAssignment[]>(
      "GET",
      "/v1/workspaces/{ws_id}/payroll/overtime/assignments",
      { params: { path: { ws_id: employmentId } } },
    );
     
    return data;
  },

  getWorkspaceQuotaSummary: async (workspaceId: string, month: string): Promise<QuotaSummary[]> => {
    const { data } = await fetch<QuotaSummary[]>(
      "GET",
      "/v1/workspaces/{ws_id}/overtime/quota/summary",
      { params: { path: { ws_id: workspaceId }, query: { month } } },
    );
     
    return data;
  },

  getEmployeeQuota: async (workspaceId: string, employeeId: string, date: string): Promise<QuotaSummary> => {
    const { data } = await fetch<QuotaSummary>(
      "GET",
      "/v1/workspaces/{ws_id}/overtime/quota/{employee_id}",
      { params: { path: { ws_id: workspaceId, employee_id: employeeId }, query: { date } } },
    );
     
    return data;
  },

  listRequestBehalfAssignments: async (workspaceId: string): Promise<BehalfAssignment[]> => {
    const { data } = await client.GET("/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments", {
      params: {
        path: { ws_id: workspaceId },
      },
    });
    return data.data;
  },

  createRequestBehalfAssignment: async (workspaceId: string, payload: CreateBehalfAssignmentRequest): Promise<BehalfAssignment> => {
    const { data } = await client.POST("/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments", {
      params: {
        path: { ws_id: workspaceId },
      },
      body: payload,
    });
    return data;
  },

  deleteRequestBehalfAssignment: async (workspaceId: string, id: string): Promise<void> => {
    await client.DELETE("/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/{id}", {
      params: {
        path: { ws_id: workspaceId, id },
      },
    });
  }
};
