import { apiClient } from "./client";
import { Shift, EmployeeSchedule, AssignmentHistory, ShiftRoster } from "./types";

export async function fetchShifts(wsId: string): Promise<Shift[]> {
  const { data } = await apiClient.get<Shift[]>(`/workspaces/${wsId}/shifts`);
  return data;
}

export async function createShift(wsId: string, data: Partial<Shift>): Promise<Shift> {
  const { data: result } = await apiClient.post<Shift>(`/workspaces/${wsId}/shifts`, data);
  return result;
}

export async function updateShift(
  wsId: string, 
  shiftId: string, 
  data: Partial<Shift> & { is_correction?: boolean, effective_date?: string }
): Promise<Shift> {
  const { data: result } = await apiClient.put<Shift>(`/workspaces/${wsId}/shifts/${shiftId}`, data);
  return result;
}

export async function assignShift(
  wsId: string,
  data: { employee_id: string; shift_id: string; start_date: string; end_date?: string | null }
): Promise<void> {
  await apiClient.post(`/workspaces/${wsId}/shifts/assign`, data);
}

export const fetchWorkspaceSchedules = async (wsId: string, startDate: string, endDate: string): Promise<EmployeeSchedule[]> => {
  const params = new URLSearchParams({ start_date: startDate, end_date: endDate });
  const { data } = await apiClient.get<EmployeeSchedule[]>(`/workspaces/${wsId}/schedules?${params}`);
  return data;
};

export const fetchAssignmentHistory = async (wsId: string): Promise<AssignmentHistory[]> => {
  const { data } = await apiClient.get<AssignmentHistory[]>(`/workspaces/${wsId}/assignment-history`);
  return data;
};

export const fetchEmployeeRosters = async (
  wsId: string, 
  employeeId: string, 
  startDate?: string, 
  endDate?: string
): Promise<ShiftRoster[]> => {
  const params = new URLSearchParams();
  if (startDate) params.append("start_date", startDate);
  if (endDate) params.append("end_date", endDate);
  
  const query = params.toString() ? `?${params.toString()}` : "";
  const { data } = await apiClient.get<ShiftRoster[]>(`/workspaces/${wsId}/employees/${employeeId}/rosters${query}`);
  return data;
}

export async function upsertRoster(wsId: string, employeeId: string, data: Partial<ShiftRoster>): Promise<ShiftRoster> {
  const { data: result } = await apiClient.post<ShiftRoster>(`/workspaces/${wsId}/employees/${employeeId}/rosters`, data);
  return result;
}

export async function bulkUpsertRoster(wsId: string, employeeId: string, items: Partial<ShiftRoster>[]): Promise<void> {
  await apiClient.post(`/workspaces/${wsId}/employees/${employeeId}/rosters/bulk`, { items });
}

export async function deleteRoster(wsId: string, employeeId: string, date: string): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/employees/${employeeId}/rosters/${date}`);
}
