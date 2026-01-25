import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { LeaveRequest, EmployeeLeaveBalance, WorkspaceLeaveBalance, LeaveHistoryItem, LeavePolicy, LeaveType, LeaveTypeConfig, PaginatedResponse } from "./types";

export async function submitLeaveRequest(
  wsId: string,
  data: { employee_id: string; leave_type_id: string; start_date: string; end_date: string; reason?: string },
): Promise<LeaveRequest> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/submit`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to submit leave request");
  return response.json();
}

export async function fetchWorkspaceLeaveRequests(
  wsId: string,
  status?: string,
): Promise<LeaveRequest[]> {
  const url = status
    ? `${API_BASE_URL}/workspaces/${wsId}/leave/requests?status=${status}`
    : `${API_BASE_URL}/workspaces/${wsId}/leave/requests`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch leave requests");
  return response.json();
}

export async function fetchMyLeaveRequests(
  employeeId: string
): Promise<LeaveRequest[]> {
  const response = await apiFetch(`${API_BASE_URL}/employees/${employeeId}/leave/requests`);
  if (!response.ok) throw new Error("Failed to fetch my leave requests");
  return response.json();
}

export async function approveLeaveRequest(
  wsId: string,
  requestId: string,
  data: { status: string },
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/requests/${requestId}/approve`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to approve leave request");
}

export async function fetchEmployeeLeaveBalances(
  wsId: string,
  employeeId: string,
): Promise<EmployeeLeaveBalance[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/balances`);
  if (!response.ok) throw new Error("Failed to fetch leave balances");
  return response.json();
}

export async function fetchWorkspaceLeaveBalances(
  wsId: string,
  params?: { page?: number; limit?: number; q?: string }
): Promise<PaginatedResponse<WorkspaceLeaveBalance>> {
  const query = new URLSearchParams();
  if (params?.page) query.append("page", params.page.toString());
  if (params?.limit) query.append("limit", params.limit.toString());
  if (params?.q) query.append("q", params.q);

  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/balances?${query.toString()}`);
  if (!response.ok) throw new Error("Failed to fetch workspace leave balances");
  return response.json();
}

export async function fetchLeaveHistory(
  wsId: string,
  employeeId: string,
): Promise<LeaveHistoryItem[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/history`);
  if (!response.ok) throw new Error("Failed to fetch leave history");
  return response.json();
}

export async function adjustLeaveBalance(
  wsId: string,
  employeeId: string,
  data: { leave_type_id: string; amount: number; reason: string; transaction_date: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/adjust`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to adjust leave balance");
}

export async function fetchLeavePolicies(wsId: string): Promise<LeavePolicy[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies`);
  if (!response.ok) throw new Error("Failed to fetch leave policies");
  return response.json();
}

export async function createLeavePolicy(
  wsId: string,
  data: Partial<LeavePolicy>
): Promise<LeavePolicy> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create leave policy");
  return response.json();
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
  if (!response.ok) throw new Error("Failed to update leave policy");
  return response.json();
}

export async function deleteLeavePolicy(
  wsId: string,
  policyId: string
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies/${policyId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete leave policy");
}

export async function assignLeavePolicy(
  wsId: string,
  data: { 
    assignee_type: string; 
    assignee_id: string | null; 
    policy_id: string; 
    effective_from: string; 
    effective_to?: string 
  }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies/assign`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to assign leave policy");
}

export async function triggerAccrualRun(
  wsId: string,
  data: { date: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/accrual/run`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to trigger accrual run");
}

export async function fetchLeaveTypes(wsId: string): Promise<LeaveType[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types`);
  if (!response.ok) throw new Error("Failed to fetch leave types");
  return response.json();
}

export async function updateLeaveTypeConfig(
  wsId: string,
  typeId: string,
  config: LeaveTypeConfig
): Promise<LeaveType> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types/${typeId}/config`, {
    method: "PUT",
    body: JSON.stringify({ policy_config: config }),
  });
  if (!response.ok) throw new Error("Failed to update leave type config");
  return response.json();
}

export async function createLeaveType(
  wsId: string,
  data: { code: string; name: string; is_paid: boolean }
): Promise<LeaveType> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create leave type");
  return response.json();
}

export async function deleteLeaveType(
  wsId: string,
  typeId: string
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/types/${typeId}`, {
    method: "DELETE",
  });
  if (response.status === 409) {
    const error = await response.text();
    throw new Error(error);
  }
  if (!response.ok) throw new Error("Failed to delete leave type");
}
