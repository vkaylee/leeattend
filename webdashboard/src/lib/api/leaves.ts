import { API_BASE_URL } from './config';
import { apiFetch } from "./helpers";
import { LeavePolicyAssignmentDetail, LeaveRequest, EmployeeLeaveBalance, WorkspaceLeaveBalance, LeaveHistoryItem, LeavePolicy, LeaveType, LeaveTypeConfig, LeaveDurationType, PaginatedResponse } from './types';

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
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/submit`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to submit leave request');
  return response.json() as Promise<LeaveRequest>;
}

export async function fetchWorkspaceLeaveRequests(
  wsId: string,
  status?: string,
): Promise<LeaveRequest[]> {
  const url = status
    ? `${API_BASE_URL}/workspaces/${wsId}/leave/requests?status=${status}`
    : `${API_BASE_URL}/workspaces/${wsId}/leave/requests`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error('Failed to fetch leave requests');
  return response.json() as Promise<LeaveRequest[]>;
}

export async function fetchMyLeaveRequests(
  employeeId: string
): Promise<LeaveRequest[]> {
  const response = await apiFetch(`${API_BASE_URL}/employees/${employeeId}/leave/requests`);
  if (!response.ok) throw new Error('Failed to fetch my leave requests');
  return response.json() as Promise<LeaveRequest[]>;
}

export async function approveLeaveRequest(
  wsId: string,
  requestId: string,
  data: { status: string },
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/requests/${requestId}/approve`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to approve leave request');
}

export async function fetchEmployeeLeaveBalances(
  wsId: string,
  employeeId: string,
): Promise<EmployeeLeaveBalance[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/balances`);
  if (!response.ok) throw new Error('Failed to fetch leave balances');
  return response.json() as Promise<EmployeeLeaveBalance[]>;
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

  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/balances?${query.toString()}`);
  if (!response.ok) throw new Error('Failed to fetch workspace leave balances');
  return response.json() as Promise<PaginatedResponse<WorkspaceLeaveBalance>>;
}

export async function fetchLeaveHistory(
  wsId: string,
  employeeId: string,
  leaveTypeId?: string,
): Promise<LeaveHistoryItem[]> {
  const url = leaveTypeId
    ? `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/history?leave_type_id=${leaveTypeId}`
    : `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/history`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error('Failed to fetch leave history');
  return response.json() as Promise<LeaveHistoryItem[]>;
}

export async function adjustLeaveBalance(
  wsId: string,
  employeeId: string,
  data: { leave_type_id: string; amount: number; reason: string; transaction_date: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/adjust`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to adjust leave balance');
}

export async function fetchLeavePolicies(wsId: string): Promise<LeavePolicy[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies`);
  if (!response.ok) throw new Error('Failed to fetch leave policies');
  return response.json() as Promise<LeavePolicy[]>;
}

export async function createLeavePolicy(
  wsId: string,
  data: Partial<LeavePolicy>
): Promise<LeavePolicy> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to create leave policy');
  return response.json() as Promise<LeavePolicy>;
}

export async function updateLeavePolicy(
  wsId: string,
  policyId: string,
  data: Partial<LeavePolicy>
): Promise<LeavePolicy> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies/${policyId}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to update leave policy');
  return response.json() as Promise<LeavePolicy>;
}

export async function deleteLeavePolicy(
  wsId: string,
  policyId: string
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies/${policyId}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error('Failed to delete leave policy');
}

export async function fetchLeaveTypes(wsId: string): Promise<LeaveType[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types`);
  if (!response.ok) throw new Error('Failed to fetch leave types');
  return response.json() as Promise<LeaveType[]>;
}

export async function createLeaveType(
  wsId: string,
  data: Partial<LeaveType>
): Promise<LeaveType> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to create leave type');
  return response.json() as Promise<LeaveType>;
}

export async function updateLeaveType(
  wsId: string,
  id: string,
  data: Partial<LeaveType>
): Promise<LeaveType> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types/${id}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to update leave type');
  return response.json() as Promise<LeaveType>;
}

export async function deleteLeaveType(
  wsId: string,
  id: string
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error('Failed to delete leave type');
}

export async function fetchLeavePolicyAssignments(wsId: string): Promise<LeavePolicyAssignmentDetail[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies/assignments`);
  if (!response.ok) throw new Error('Failed to fetch policy assignments');
  return response.json() as Promise<LeavePolicyAssignmentDetail[]>;
}

export async function assignLeavePolicy(
  wsId: string,
  data: { assignee_type: string; assignee_id: string | null; policy_id: string; effective_from: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies/assign`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to assign leave policy');
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
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/accrual/run`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to trigger accrual run');
  return response.json() as Promise<AccrualJobResponse>;
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
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/jobs/${jobId}/status`);
  if (!response.ok) throw new Error('Failed to fetch job status');
  return response.json() as Promise<JobStatusResponse>;
}

export async function updateLeaveTypeConfig(
  wsId: string,
  id: string,
  config: LeaveTypeConfig
): Promise<LeaveType> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types/${id}/config`, {
    method: "PATCH",
    body: JSON.stringify(config),
  });
  if (!response.ok) throw new Error('Failed to update leave type config');
  return response.json() as Promise<LeaveType>;
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
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/upload-excel`, {
    method: 'POST',
    body: formData,
    headers: {}, // Let browser set Content-Type for multipart
  });
  if (!response.ok) throw new Error('Failed to upload leave excel');
  return response.json() as Promise<{ job_id: string; status: string }>;
}

/** Fetch the latest leave import job for the workspace (page resume). */
export async function fetchLatestLeaveImport(
  wsId: string,
): Promise<LeaveImportJob | null> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/import/latest`);
  if (!response.ok) return null;
  return response.json() as Promise<LeaveImportJob | null>;
}

/** Fetch a specific import job by its ID (for polling). */
export async function fetchLeaveImportStatus(
  wsId: string,
  _jobId: string,
): Promise<LeaveImportJob | null> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/import/latest`);
  if (!response.ok) return null;
  return response.json() as Promise<LeaveImportJob | null>;
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
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/leave/upload-template?lang=${lang}`,
    { timeout: 30000 }
  );
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
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/leave/import/${jobId}/errors`,
    { timeout: 30000 }
  );
  if (!response.ok) throw new Error("Failed to download error file");
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
