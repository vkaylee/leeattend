import type { paths } from "@/types/api-generated";

import type { LeavePolicyAssignmentDetail, LeaveRequest, EmployeeLeaveBalance, WorkspaceLeaveBalance, LeaveHistoryItem, LeavePolicy, LeaveType, LeaveTypeConfig, LeaveDurationType, PaginatedResponse, BehalfAssignment, CreateBehalfAssignmentRequest } from './types';

import { client } from "./client";
import { API_BASE_URL } from './config';
import { apiFetch } from "./helpers";

export async function submitLeaveRequest(
  wsId: string,
  data: {
    employee_id: string;
    leave_type_id: string;
    start_date: string;
    end_date: string;
    reason?: string;
    duration_type?: LeaveDurationType;
    start_time?: string;
    end_time?: string;
  },
): Promise<LeaveRequest> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/leave/submit", {
    params: {
      path: { ws_id: wsId },
    },
    body: data,
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchWorkspaceLeaveRequests(
  wsId: string,
  status?: "pending" | "approved" | "rejected" | "cancelled" | "pre_approved"
): Promise<LeaveRequest[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/requests", {
    params: {
      path: { ws_id: wsId },
      query: { status }
    },
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchMyLeaveRequests(
  employeeId: string
): Promise<LeaveRequest[]> {
  const { data: resData } = await client.GET("/employees/{id}/leave/requests", {
    params: {
      path: { id: employeeId },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function approveLeaveRequest(
  wsId: string,
  requestId: string,
  data: paths["/workspaces/{ws_id}/leave/requests/{id}/approve"]["post"]["requestBody"]["content"]["application/json"]
): Promise<void> {
  await client.POST("/workspaces/{ws_id}/leave/requests/{id}/approve", {
    params: {
      path: { ws_id: wsId, id: requestId },
    },
    body: data,
  });
}

export async function fetchEmployeeLeaveBalances(
  wsId: string,
  employeeId: string,
): Promise<EmployeeLeaveBalance[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/employees/{id}/leave/balances", {
    params: {
      path: { ws_id: wsId, id: employeeId },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchWorkspaceLeaveBalances(
  wsId: string,
  params?: { page?: number; limit?: number; q?: string; year?: number }
): Promise<PaginatedResponse<WorkspaceLeaveBalance>> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/balances", {
    params: {
      path: { ws_id: wsId },
      query: {
              page: params?.page,
              limit: params?.limit,
              q: params?.q,
              year: params?.year,
            },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchLeaveHistory(
  wsId: string,
  employeeId: string,
  leaveTypeId?: string,
): Promise<LeaveHistoryItem[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/employees/{id}/leave/history", {
    params: {
      path: { ws_id: wsId, id: employeeId },
      query: leaveTypeId ? { leave_type_id: leaveTypeId } : undefined,
    },
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function adjustLeaveBalance(
  wsId: string,
  employeeId: string,
  data: { leave_type_id: string; amount: number; reason: string; transaction_date: string }
): Promise<void> {
  await client.POST("/workspaces/{ws_id}/employees/{id}/leave/adjust", {
    params: {
      path: { ws_id: wsId, id: employeeId },
    },
    body: data,
  });
}

export async function fetchLeavePolicies(wsId: string): Promise<LeavePolicy[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/policies", {
    params: {
      path: { ws_id: wsId },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as LeavePolicy[];
}

export async function createLeavePolicy(
  wsId: string,
  data: { description?: string | null; leave_type_id: string; name: string; rule_config: Record<string, unknown> }
): Promise<LeavePolicy> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/leave/policies", {
    params: {
      path: { ws_id: wsId },
    },
    body: data as unknown as paths["/workspaces/{ws_id}/leave/policies"]["post"]["requestBody"]["content"]["application/json"],
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as LeavePolicy;
}

export async function updateLeavePolicy(
  wsId: string,
  policyId: string,
  data: Partial<LeavePolicy>
): Promise<LeavePolicy> {
  const { data: resData } = await client.PATCH("/workspaces/{ws_id}/leave/policies/{policy_id}", {
    params: {
      path: { ws_id: wsId, policy_id: policyId },
    },
    body: data as unknown as paths["/workspaces/{ws_id}/leave/policies/{policy_id}"]["patch"]["requestBody"]["content"]["application/json"],
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as LeavePolicy;
}

export async function deleteLeavePolicy(
  wsId: string,
  policyId: string
): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/leave/policies/{policy_id}", {
    params: {
      path: { ws_id: wsId, policy_id: policyId },
    },
  });
}

export async function fetchLeaveTypes(wsId: string): Promise<LeaveType[]> {
  const { data: resData } = await client.GET("/workspaces/{workspace_id}/leave/types", {
    params: {
      path: { workspace_id: wsId },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createLeaveType(
  wsId: string,
  data: { code: string; is_paid: boolean; name: string; policy_config?: Record<string, unknown> }
): Promise<LeaveType> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/leave/types", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data,
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export function updateLeaveType(
  _wsId: string,
  _id: string,
  _data: Partial<LeaveType>
): Promise<LeaveType> {
  throw new Error("updateLeaveType is deprecated. Use updateLeaveTypeConfig instead.");
}

export async function deleteLeaveType(
  wsId: string,
  id: string
): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/leave/types/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
  });
}

export async function fetchLeavePolicyAssignments(wsId: string): Promise<LeavePolicyAssignmentDetail[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/policies/assignments", {
    params: {
      path: { ws_id: wsId },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as LeavePolicyAssignmentDetail[];
}

export async function assignLeavePolicy(
  wsId: string,
  data: { assignee_type: string; assignee_id: string | null; policy_id: string; effective_from: string }
): Promise<void> {
  await client.POST("/workspaces/{ws_id}/leave/policies/assign", {
    params: {
      path: { ws_id: wsId },
    },
    body: data,
  });
}

export interface AccrualJobResponse {
  job_id: string;
  message: string;
  status: string;
}

export async function triggerAccrualRun(
  wsId: string,
  data: { date: string }
): Promise<AccrualJobResponse> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/leave/accrual/run", {
    params: {
      path: { ws_id: wsId },
    },
    body: data,
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export interface JobStatusResponse {
  completed_at: string | null;
  error_message: string | null;
  job_id: string;
  status: string;
}

export async function fetchJobStatus(
  wsId: string,
  jobId: string
): Promise<JobStatusResponse> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/jobs/{job_id}/status", {
    params: {
      path: { ws_id: wsId, job_id: jobId },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as JobStatusResponse;
}

export async function updateLeaveTypeConfig(
  wsId: string,
  id: string,
  config: LeaveTypeConfig
): Promise<LeaveType> {
  const { data: resData } = await client.PUT("/workspaces/{workspace_id}/leave/types/{id}/config", {
    params: {
      path: { workspace_id: wsId, id },
    },
    body: { policy_config: config },
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

// ========================
// Leave Import API
// ========================

export interface LeaveImportJob {
  created_at: string;
  error_log: string | null;
  id: string;
  processed_rows: number;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  total_rows: number;
  updated_at: string;
}

/** Upload an Excel file for bulk leave request import. */
export async function uploadLeaveExcel(
  wsId: string,
  file: File,
): Promise<{ job_id: string; status: string }> {
  const formData = new FormData();
  formData.append('file', file);
  const { data: resData } = await client.POST("/workspaces/{ws_id}/leave/upload-excel", {
    params: {
      path: { ws_id: wsId },
    },
    body: formData as unknown as undefined,
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as { job_id: string; status: string };
}

/** Fetch the latest leave import job for the workspace (page resume). */
export async function fetchLatestLeaveImport(
  wsId: string,
): Promise<LeaveImportJob | null> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/import/latest", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return resData as LeaveImportJob | null;
}

/** Fetch a specific import job by its ID (for polling). */
export async function fetchLeaveImportStatus(
  wsId: string,
  _jobId: string,
): Promise<LeaveImportJob | null> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/import/latest", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return resData as LeaveImportJob | null;
}

/** Get the URL for downloading the error Excel file (URL-only, for non-fetch usage). */
export function getLeaveImportErrorsUrl(wsId: string, jobId: string): string {
  return `${API_BASE_URL}/v1/workspaces/${wsId}/leave/import/${jobId}/errors`;
}

/** Get the URL for downloading the import template Excel (URL-only, for non-fetch usage). */
export function getLeaveImportTemplateUrl(wsId: string, lang = "en"): string {
  return `${API_BASE_URL}/v1/workspaces/${wsId}/leave/upload-template?lang=${lang}`;
}

/** Download the leave import template Excel via authenticated fetch. */
export async function downloadLeaveImportTemplate(wsId: string, lang = "en"): Promise<void> {
  const response = await apiFetch(`/v1/workspaces/${wsId}/leave/upload-template?lang=${lang}`);
  if (!response.ok) throw new Error("Failed to download template");
  const blob = await response.blob();
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "leave_import_template.xlsx";
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  window.URL.revokeObjectURL(url);
}

/** Download the error Excel file via authenticated fetch. */
export async function downloadLeaveImportErrors(wsId: string, jobId: string): Promise<void> {
  const response = await apiFetch(`/v1/workspaces/${wsId}/leave/import/${jobId}/errors`);
  if (!response.ok) throw new Error("Failed to download errors");
  const blob = await response.blob();
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `leave_import_errors_${jobId}.xlsx`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  window.URL.revokeObjectURL(url);
}

export async function fetchLeaveBehalfAssignments(
  wsId: string
): Promise<BehalfAssignment[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/leave/behalf-assignments", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createLeaveBehalfAssignment(
  wsId: string,
  data: CreateBehalfAssignmentRequest
): Promise<BehalfAssignment> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/leave/behalf-assignments", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function deleteLeaveBehalfAssignment(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/leave/behalf-assignments/{id}", {
    params: { path: { ws_id: wsId, id } }
  });
}
