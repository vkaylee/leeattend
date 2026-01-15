import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { Shift, EmployeeSchedule, AssignmentHistory, ShiftRoster } from "./types";

export async function fetchShifts(wsId: string): Promise<Shift[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/shifts`);
  if (!response.ok) throw new Error("Failed to fetch shifts");
  return response.json();
}

export async function createShift(wsId: string, data: Partial<Shift>): Promise<Shift> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/shifts`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create shift");
  return response.json();
}

export async function updateShift(
  wsId: string, 
  shiftId: string, 
  data: Partial<Shift> & { is_correction?: boolean, effective_date?: string }
): Promise<Shift> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/shifts/${shiftId}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update shift");
  return response.json();
}

export async function assignShift(
  wsId: string,
  data: { employee_id: string; shift_id: string; start_date: string; end_date?: string | null }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/shifts/assign`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to assign shift");
}

export const fetchWorkspaceSchedules = async (wsId: string, startDate: string, endDate: string): Promise<EmployeeSchedule[]> => {
  const params = new URLSearchParams({ start_date: startDate, end_date: endDate });
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/schedules?${params}`);
  if (!response.ok) throw new Error("Failed to fetch schedules");
  return response.json();
};

export const fetchAssignmentHistory = async (wsId: string): Promise<AssignmentHistory[]> => {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-history`);
  if (!response.ok) throw new Error("Failed to fetch assignment history");
  return response.json();
};

export const fetchEmployeeRosters = async (
  wsId: string, 
  employeeId: string, 
  startDate?: string, 
  endDate?: string
): Promise<ShiftRoster[]> => {
  let url = `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/rosters`;
  const params = new URLSearchParams();
  if (startDate) params.append("start_date", startDate);
  if (endDate) params.append("end_date", endDate);
  if (params.toString()) url += `?${params.toString()}`;
  
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch rosters");
  return response.json();
}

export async function upsertRoster(wsId: string, employeeId: string, data: any): Promise<ShiftRoster> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/rosters`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to upsert roster");
  return response.json();
}

export async function bulkUpsertRoster(wsId: string, employeeId: string, items: any[]): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/rosters/bulk`, {
    method: "POST",
    body: JSON.stringify({ items }),
  });
  if (!response.ok) throw new Error("Failed to bulk upsert roster");
}

export async function deleteRoster(wsId: string, employeeId: string, date: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/rosters/${date}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete roster");
}
