import { API_BASE_URL } from "./config";
import { apiFetch, getCurrentWorkspaceId } from "./helpers";
import { TimeEntry, AttendanceAdjustmentRequest, RawAttendanceLog, AttendanceImportJob } from "./types";

export async function punchIn(
  wsId: string,
  data: { employee_id: string; entry_type?: string; note?: string },
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/punch-in`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to punch in");
  return response.json();
}

export async function punchOut(
  wsId: string,
  employeeId: string,
  data: { note?: string },
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/punch-out/${employeeId}`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to punch out");
  return response.json();
}

export async function fetchTimeHistory(
  wsId: string,
  employeeId: string,
  start: string,
  end: string,
): Promise<TimeEntry[]> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/time/history/${employeeId}?start=${start}&end=${end}`,
  );
  if (!response.ok) throw new Error("Failed to fetch time history");
  return response.json();
}

export async function updateTimeEntry(
  wsId: string,
  id: string,
  data: Partial<TimeEntry>,
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/entries/${id}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update time entry");
  return response.json();
}

export async function deleteTimeEntry(wsId: string, id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/entries/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete time entry");
}

export async function submitAttendanceAdjustment(
  wsId: string,
  data: {
    employee_id: string;
    time_entry_id?: string;
    requested_punch_in_time: string;
    requested_punch_out_time?: string;
    reason?: string;
  }
): Promise<AttendanceAdjustmentRequest> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to submit attendance adjustment");
  return response.json();
}

export async function fetchWorkspaceAttendanceAdjustments(
  wsId: string,
  status?: string
): Promise<AttendanceAdjustmentRequest[]> {
  const url = status 
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments?status=${status}`
    : `${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch attendance adjustments");
  return response.json();
}

export async function fetchMyAttendanceAdjustments(
  employeeId: string
): Promise<AttendanceAdjustmentRequest[]> {
  const response = await apiFetch(`${API_BASE_URL}/employees/${employeeId}/attendance/adjustments`);
  if (!response.ok) throw new Error("Failed to fetch my attendance adjustments");
  return response.json();
}

export async function approveAttendanceAdjustment(
  wsId: string,
  requestId: string,
  data: { status: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments/${requestId}/approve`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to approve attendance adjustment");
}

export async function fetchAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: TimeEntry[]; total: number; page: number; limit: number }> {
  const searchParams = new URLSearchParams();
  if (params.employee_id) searchParams.append("employee_id", params.employee_id);
  if (params.from) searchParams.append("from", params.from);
  if (params.to) searchParams.append("to", params.to);
  if (params.page) searchParams.append("page", params.page.toString());
  if (params.limit) searchParams.append("limit", params.limit.toString());
  
  const wsId = getCurrentWorkspaceId();
  const url = wsId 
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/logs?${searchParams.toString()}`
    : `${API_BASE_URL}/attendance/logs?${searchParams.toString()}`;

  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch attendance logs");
  return response.json();
}

export async function fetchRawAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }> {
  const searchParams = new URLSearchParams();
  if (params.employee_id) searchParams.append("employee_id", params.employee_id);
  if (params.from) searchParams.append("from", params.from);
  if (params.to) searchParams.append("to", params.to);
  if (params.page) searchParams.append("page", params.page.toString());
  if (params.limit) searchParams.append("limit", params.limit.toString());
  
  const wsId = getCurrentWorkspaceId();
  if (!wsId) throw new Error("Workspace not selected");
  
  const url = `${API_BASE_URL}/workspaces/${wsId}/attendance/raw-logs?${searchParams.toString()}`;

  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch raw attendance logs");
  return response.json();
}

export async function fetchAttendanceReport(
  wsId: string,
  start: string,
  end: string,
  type: "summary" | "daily" = "summary",
): Promise<any> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/reports/attendance?start_date=${start}&end_date=${end}&type=${type}`,
  );
  if (!response.ok) throw new Error("Failed to fetch attendance report");
  return response.json();
}

export async function uploadAttendanceExcel(
  wsId: string,
  file: File
): Promise<{ job_id: string }> {
  return uploadAttendance(wsId, file, "/attendance/upload");
}

export async function uploadAttendance(
  wsId: string,
  file: File,
  endpoint: string
): Promise<{ job_id: string }> {
  const formData = new FormData();
  formData.append("file", file);

  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}${endpoint}`, {
    method: "POST",
    body: formData,
  });
  if (!response.ok) throw new Error("Failed to upload attendance");
  return response.json();
}

export async function fetchLatestImportJob(
  wsId: string,
  targetType?: string
): Promise<AttendanceImportJob | null> {
  const url = targetType 
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/latest?target_type=${targetType}`
    : `${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/latest`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch latest import job");
  return response.json();
}

export async function fetchImportJobStatus(
  wsId: string,
  jobId: string
): Promise<AttendanceImportJob> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/${jobId}`);
  if (!response.ok) throw new Error("Failed to fetch import job status");
  return response.json();
}
