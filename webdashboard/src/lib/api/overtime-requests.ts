import type { ApprovalActionLog, PaginatedResponse } from './types';

import { createBehalfAssignmentApi } from './behalf-assignment-factory';
import { client } from './client';

export interface OvertimeRequestQueryParams {
  [key: string]: unknown;
  consent?: string;
  department_id?: string;
  employee_id?: string;
  limit?: number;
  page?: number;
  planned_date?: string;
  status?: OtRequestStatus;
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

export interface QuotaSummary {
  employee_id: string;
  monthly_limit: number | null;
  monthly_used: number;
  yearly_limit: number | null;
  yearly_used: number;
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

const otRequestBehalfApi = createBehalfAssignmentApi({
  basePath: "/api/v1/overtime/requests/behalf-assignments",
  dataWrapper: true,
});

export const overtimeRequestsApi = {
  listRequests: async (params?: OvertimeRequestQueryParams): Promise<PaginatedResponse<OvertimeRequest>> => {
    const { data } = await client.GET<PaginatedResponse<OvertimeRequest>>("/api/v1/overtime/requests", {
      params: {
        query: params,
      },
    });
    return data;
  },

  getRequest: async (id: string): Promise<OvertimeRequestDetail> => {
    const { data } = await client.GET<OvertimeRequestDetail>("/api/v1/overtime/requests/{id}", {
      params: { path: { id } },
    });
    return data;
  },

  createRequest: async (payload: CreateOvertimeRequestPayload): Promise<OvertimeRequest> => {
    const { data } = await client.POST<OvertimeRequest>("/api/v1/overtime/requests", {
      body: payload,
    });
    return data;
  },

  submitRequest: async (id: string): Promise<void> => {
    await client.POST("/api/v1/overtime/requests/{id}/submit", {
      params: {
        path: { id },
      },
    });
  },

  cancelRequest: async (id: string): Promise<void> => {
    await client.PUT("/api/v1/overtime/requests/{id}/cancel", {
      params: {
        path: { id },
      },
    });
  },

  employeeConsent: async (id: string, payload: {
    employment_id: string;
    status: ConsentStatus;
    decline_reason?: string;
  }): Promise<void> => {
    await client.POST("/api/v1/overtime/requests/{id}/consent/{emp_id}", {
      params: {
        path: { id, emp_id: payload.employment_id },
      },
      body: {
        status: payload.status,
        decline_reason: payload.decline_reason,
      },
    });
  },

  getMyAssignments: async (employmentId: string): Promise<MyOvertimeAssignment[]> => {
    const { data } = await client.GET<MyOvertimeAssignment[]>(
      "/api/v1/payroll/overtime/assignments",
      { params: { query: { employment_id: employmentId } } },
    );
    return data;
  },

  // ── OT Request Behalf Assignments (factory-generated CRUD) ──────────────

  listRequestBehalfAssignments: () => otRequestBehalfApi.list(),
  createRequestBehalfAssignment: (payload: Parameters<typeof otRequestBehalfApi.create>[0]) => otRequestBehalfApi.create(payload),
  deleteRequestBehalfAssignment: (id: string) => otRequestBehalfApi.delete(id),
};
