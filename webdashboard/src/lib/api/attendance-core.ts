import { client, fetch } from "./client";
import { getCurrentWorkspaceId } from "./helpers";
import {
  TimeEntry,
  AttendanceAdjustmentRequest,
  RawAttendanceLog,
  PaginatedResponse,
} from "./types";

export async function punchIn(
  wsId: string,
  data: { employment_id: string; entry_type?: "Work" | "Break" | "Overtime"; note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/time/punch-in", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data,
  });
  return resData;
}

export async function punchOut(
  wsId: string,
  data: { employee_id: string; note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/time/punch-out/{employee_id}", {
    params: {
      path: { workspace_id: wsId, employee_id: data.employee_id },
    },
    body: {
      note: data.note,
    },
  });
  return resData;
}

export async function submitAttendanceAdjustment(
  wsId: string,
  data: {
    employment_id: string;
    time_entry_id?: string;
    requested_punch_time: string;
    reason?: string;
    attachment_file_ids?: string[];
  }
): Promise<AttendanceAdjustmentRequest> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/adjustments", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  return resData;
}

export async function fetchWorkspaceAttendanceAdjustments(
  wsId: string,
  params?: {
    status?: string;
    employee_id?: string;
    page?: number;
    limit?: number;
  }
): Promise<PaginatedResponse<AttendanceAdjustmentRequest>> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/adjustments", {
    params: {
      path: { ws_id: wsId },
      query: {
        status: params?.status as "pending" | "approved" | "rejected" | null | undefined,
        employee_id: params?.employee_id === "all" ? undefined : params?.employee_id,
        page: params?.page,
        limit: params?.limit,
      },
    }
  });
  return resData;
}

export async function approveAttendanceAdjustment(wsId: string, requestId: string, data: { status: "pending" | "approved" | "rejected" }): Promise<void> {
  await client.PUT("/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve", {
    params: { path: { ws_id: wsId, id: requestId } },
    body: data
  });
}

export async function fetchRawAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }> {
  const wsId = getCurrentWorkspaceId();
  if (!wsId) throw new Error("Workspace not selected");

  const { data: resData } = await fetch<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }>("GET", "/workspaces/{workspace_id}/attendance/raw-logs", {
    params: {
      path: { workspace_id: wsId },
      query: {
        employee_id: params.employee_id ?? undefined,
        from: params.from ?? undefined,
        to: params.to ?? undefined,
        page: params.page ?? undefined,
        limit: params.limit ?? undefined,
      },
    },
  });
  return resData;
}
