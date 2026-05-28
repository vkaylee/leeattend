import { client } from "./client";
import type { Shift, EmployeeSchedule, AssignmentHistory, ShiftRoster } from "./types";

export type { Shift, EmployeeSchedule, AssignmentHistory, ShiftRoster };

export async function fetchShifts(wsId: string): Promise<Shift[]> {
  const { data } = await client.GET("/v1/workspaces/{workspace_id}/shifts", {
    params: {
      path: { workspace_id: wsId },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as Shift[];
}

export async function createShift(wsId: string, data: Partial<Shift>): Promise<Shift> {
  const { data: result } = await client.POST("/v1/workspaces/{workspace_id}/shifts", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data as any,
  });
  if (!result) throw new Error("No response data");
  return result as unknown as Shift;
}

export async function updateShift(
  wsId: string, 
  shiftId: string, 
  data: Partial<Shift> & { is_correction?: boolean, effective_date?: string }
): Promise<Shift> {
  const { data: result } = await client.PUT("/v1/workspaces/{workspace_id}/shifts/{shift_id}", {
    params: {
      path: { workspace_id: wsId, shift_id: shiftId },
    },
    body: data as any,
  });
  if (!result) throw new Error("No response data");
  return result as unknown as Shift;
}

export async function assignShift(
  wsId: string,
  data: { employee_id: string; shift_id: string; start_date: string; end_date?: string | null }
): Promise<void> {
  await client.POST("/v1/workspaces/{workspace_id}/shifts/assign", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data as any,
  });
}

export const fetchWorkspaceSchedules = async (wsId: string, startDate: string, endDate: string): Promise<EmployeeSchedule[]> => {
  const { data } = await client.GET("/v1/workspaces/{workspace_id}/schedules", {
    params: {
      path: { workspace_id: wsId },
      query: { start_date: startDate, end_date: endDate },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as EmployeeSchedule[];
};

export const fetchAssignmentHistory = async (wsId: string): Promise<AssignmentHistory[]> => {
  const { data } = await client.GET("/v1/workspaces/{workspace_id}/assignment-history", {
    params: {
      path: { workspace_id: wsId },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as AssignmentHistory[];
};

export const fetchEmployeeRosters = async (
  wsId: string, 
  employeeId: string, 
  startDate?: string, 
  endDate?: string
): Promise<ShiftRoster[]> => {
  const { data } = await client.GET("/v1/workspaces/{workspace_id}/employees/{id}/rosters", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
      query: {
        start_date: startDate || undefined,
        end_date: endDate || undefined,
      },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as ShiftRoster[];
}

export async function upsertRoster(wsId: string, employeeId: string, data: Partial<ShiftRoster>): Promise<ShiftRoster> {
  const { data: result } = await client.POST("/v1/workspaces/{workspace_id}/employees/{id}/rosters", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
    body: data as any,
  });
  if (!result) throw new Error("No response data");
  return result as unknown as ShiftRoster;
}

export async function bulkUpsertRoster(wsId: string, employeeId: string, items: Partial<ShiftRoster>[]): Promise<void> {
  await client.POST("/v1/workspaces/{workspace_id}/employees/{id}/rosters/bulk", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
    body: { items: items as any[] },
  });
}

export async function deleteRoster(wsId: string, employeeId: string, date: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{workspace_id}/employees/{id}/rosters/{date}", {
    params: {
      path: { workspace_id: wsId, id: employeeId, date },
    },
  });
}
