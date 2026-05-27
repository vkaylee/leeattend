import { API_BASE_URL } from './config';
import { apiFetch } from "./helpers";
import { LeavePolicyAssignmentDetail, LeaveRequest, EmployeeLeaveBalance, WorkspaceLeaveBalance, LeaveHistoryItem, LeavePolicy, LeaveType, LeaveTypeConfig, LeaveDurationType, PaginatedResponse } from './types';
import { apiClient } from "./client";

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
  const { data: resData } = await apiClient.post<LeaveRequest>(`/workspaces/${wsId}/leave/submit`, data);
    return resData;
}

export async function fetchWorkspaceLeaveRequests(
  wsId: string,
  status?: string,
): Promise<LeaveRequest[]> {
  const url = status
    ? `${API_BASE_URL}/workspaces/${wsId}/leave/requests?status=${status}`
    : `${API_BASE_URL}/workspaces/${wsId}/leave/requests`;
  const { data: resData } = await apiClient.get<LeaveRequest[]>(url);
    return resData;
}

export async function fetchMyLeaveRequests(
  employeeId: string
): Promise<LeaveRequest[]> {
  const { data: resData } = await apiClient.get<LeaveRequest[]>(`/employees/${employeeId}/leave/requests`);
    return resData;
}

export async function approveLeaveRequest(
  wsId: string,
  requestId: string,
  data: { status: string },
): Promise<void> {
  await apiClient.post(`/workspaces/${wsId}/leave/requests/${requestId}/approve`, data);
}

export async function fetchEmployeeLeaveBalances(
  wsId: string,
  employeeId: string,
): Promise<EmployeeLeaveBalance[]> {
  const { data: resData } = await apiClient.get<EmployeeLeaveBalance[]>(`/workspaces/${wsId}/employees/${employeeId}/leave/balances`);
    return resData;
}

export async function fetchWorkspaceLeaveBalances(
  wsId: string,
  params?: { page?: number; limit?: number; q?: string; year?: number }
): Promise<PaginatedResponse<WorkspaceLeaveBalance>> {
  const query = new URLSearchParams();
  if (params?.page) query.append('page', params.page.toString());
  if (params?.limit) query.append('limit', params.limit.toString());
  if (params?.q) query.append('q', params.q);
  if (params?.year) query.append('year', params.year.toString());

  const { data: resData } = await apiClient.get<PaginatedResponse<WorkspaceLeaveBalance>>(`/workspaces/${wsId}/leave/balances?${query.toString()}`);
    return resData;
}

export async function fetchLeaveHistory(
  wsId: string,
  employeeId: string,
  leaveTypeId?: string,
): Promise<LeaveHistoryItem[]> {
  const url = leaveTypeId
    ? `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/history?leave_type_id=${leaveTypeId}`
    : `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/history`;
  const { data: resData } = await apiClient.get<LeaveHistoryItem[]>(url);
    return resData;
}

export async function adjustLeaveBalance(
  wsId: string,
  employeeId: string,
  data: { leave_type_id: string; amount: number; reason: string; transaction_date: string }
): Promise<void> {
  await apiClient.post(`/workspaces/${wsId}/employees/${employeeId}/leave/adjust`, data);
}

export async function fetchLeavePolicies(wsId: string): Promise<LeavePolicy[]> {
  const { data: resData } = await apiClient.get<LeavePolicy[]>(`/workspaces/${wsId}/leave/policies`);
    return resData;
}

export async function createLeavePolicy(
  wsId: string,
  data: Partial<LeavePolicy>
): Promise<LeavePolicy> {
  const { data: resData } = await apiClient.post<LeavePolicy>(`/workspaces/${wsId}/leave/policies`, data);
    return resData;
}

export async function updateLeavePolicy(
  wsId: string,
  policyId: string,
  data: Partial<LeavePolicy>
): Promise<LeavePolicy> {
  const { data: resData } = await apiClient.patch<LeavePolicy>(`/workspaces/${wsId}/leave/policies/${policyId}`, data);
    return resData;
}

export async function deleteLeavePolicy(
  wsId: string,
  policyId: string
): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/leave/policies/${policyId}`);
}

export async function fetchLeaveTypes(wsId: string): Promise<LeaveType[]> {
  const { data: resData } = await apiClient.get<LeaveType[]>(`/workspaces/${wsId}/leave/types`);
    return resData;
}

export async function createLeaveType(
  wsId: string,
  data: Partial<LeaveType>
): Promise<LeaveType> {
  const { data: resData } = await apiClient.post<LeaveType>(`/workspaces/${wsId}/leave/types`, data);
    return resData;
}

export async function updateLeaveType(
  wsId: string,
  id: string,
  data: Partial<LeaveType>
): Promise<LeaveType> {
  const { data: resData } = await apiClient.patch<LeaveType>(`/workspaces/${wsId}/leave/types/${id}`, data);
    return resData;
}

export async function deleteLeaveType(
  wsId: string,
  id: string
): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/leave/types/${id}`);
}

export async function fetchLeavePolicyAssignments(wsId: string): Promise<LeavePolicyAssignmentDetail[]> {
  const { data: resData } = await apiClient.get<LeavePolicyAssignmentDetail[]>(`/workspaces/${wsId}/leave/policies/assignments`);
    return resData;
}

export async function assignLeavePolicy(
  wsId: string,
  data: { assignee_type: string; assignee_id: string | null; policy_id: string; effective_from: string }
): Promise<void> {
  await apiClient.post(`/workspaces/${wsId}/leave/policies/assign`, data);
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
  const { data: resData } = await apiClient.post<AccrualJobResponse>(`/workspaces/${wsId}/leave/accrual/run`, data);
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
  const { data: resData } = await apiClient.get<JobStatusResponse>(`/workspaces/${wsId}/jobs/${jobId}/status`);
    return resData;
}

export async function updateLeaveTypeConfig(
  wsId: string,
  id: string,
  config: LeaveTypeConfig
): Promise<LeaveType> {
  const { data: resData } = await apiClient.patch<LeaveType>(`/workspaces/${wsId}/leave/types/${id}/config`, config);
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
  const { data: resData } = await apiClient.post<{ job_id: string; status: string }>(`/workspaces/${wsId}/leave/upload-excel`, formData);
    return resData;
}

/** Fetch the latest leave import job for the workspace (page resume). */
export async function fetchLatestLeaveImport(
  wsId: string,
): Promise<LeaveImportJob | null> {
  const { data: resData } = await apiClient.get<LeaveImportJob | null>(`/workspaces/${wsId}/leave/import/latest`);
    return resData;
}

/** Fetch a specific import job by its ID (for polling). */
export async function fetchLeaveImportStatus(
  wsId: string,
  _jobId: string,
): Promise<LeaveImportJob | null> {
  const { data: resData } = await apiClient.get<LeaveImportJob | null>(`/workspaces/${wsId}/leave/import/latest`);
    return resData;
}

/** Get the URL for downloading the error Excel file (URL-only, for non-fetch usage). */
export function getLeaveImportErrorsUrl(wsId: string, jobId: string): string {
  return `${API_BASE_URL}/workspaces/${wsId}/leave/import/${jobId}/errors`;
}

/** Get the URL for downloading the import template Excel (URL-only, for non-fetch usage). */
export function getLeaveImportTemplateUrl(wsId: string, lang = "en"): string {
  return `${API_BASE_URL}/workspaces/${wsId}/leave/upload-template?lang=${lang}`;
}

/** Download the leave import template Excel via authenticated fetch. */
export async function downloadLeaveImportTemplate(wsId: string, lang = "en"): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/upload-template?lang=${lang}`);
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
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/import/${jobId}/errors`);
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
