import { apiClient } from './client';
import { ApprovalActionLog } from './approval-workflows';

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
  id: string;
  employee_id: string;
  consent_status: ConsentStatus;
  consent_at?: string;
  decline_reason?: string;
  actual_ot_minutes?: number;
  // Included from joined tables
  full_name?: string;
  employee_id_number?: string;
}

export interface OvertimeRequest {
  id: string;
  workspace_id: string;
  requested_by_user_id: string;
  department_id?: string;
  planned_date: string;
  planned_start_time?: string;
  planned_end_time?: string;
  planned_minutes: number;
  reason: string;
  status: OtRequestStatus;
  workflow_id?: string;
  current_step: number;
  quota_warning?: any;
  created_at: string;
  updated_at: string;
  employees?: OvertimeRequestEmployee[];
}

export interface OvertimeRequestDetail extends OvertimeRequest {
  approval_history?: ApprovalActionLog[];
}

export interface CreateOvertimeRequestPayload {
  department_id?: string;
  planned_date: string;
  planned_start_time?: string;
  planned_end_time?: string;
  planned_minutes: number;
  reason: string;
  employees: string[];
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
    const searchParams = new URLSearchParams(params);
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/overtime/requests?${searchParams.toString()}`);
    return response.data.items || response.data; // Handle paginated or unpaginated response
  },

  getRequest: async (workspaceId: string, id: string): Promise<OvertimeRequestDetail> => {
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/overtime/requests/${id}`);
    return response.data;
  },

  createRequest: async (workspaceId: string, payload: CreateOvertimeRequestPayload): Promise<OvertimeRequest> => {
    const response = await apiClient.post(`/api/workspaces/${workspaceId}/overtime/requests`, payload);
    return response.data;
  },

  submitRequest: async (workspaceId: string, id: string): Promise<void> => {
    await apiClient.post(`/api/workspaces/${workspaceId}/overtime/requests/${id}/submit`);
  },

  cancelRequest: async (workspaceId: string, id: string): Promise<void> => {
    await apiClient.post(`/api/workspaces/${workspaceId}/overtime/requests/${id}/cancel`);
  },

  employeeConsent: async (workspaceId: string, id: string, payload: {
    employee_id: string;
    status: ConsentStatus;
    decline_reason?: string;
  }): Promise<void> => {
    await apiClient.post(`/api/workspaces/${workspaceId}/overtime/requests/${id}/consent`, payload);
  },

  getMyAssignments: async (employeeId: string): Promise<OvertimeRequest[]> => {
    const response = await apiClient.get(`/api/employees/${employeeId}/overtime/assignments`);
    return response.data;
  },

  getWorkspaceQuotaSummary: async (workspaceId: string, month: string): Promise<QuotaSummary[]> => {
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/overtime/quota/summary?month=${month}`);
    return response.data;
  },

  getEmployeeQuota: async (workspaceId: string, employeeId: string, date: string): Promise<QuotaSummary> => {
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/overtime/quota/${employeeId}?date=${date}`);
    return response.data;
  }
};
