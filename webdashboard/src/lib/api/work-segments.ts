import type {
  WorkSegment,
  CreateWorkSegmentRequest,
  UpdateWorkSegmentRequest,
  BulkCreateWorkSegmentsRequest,
  ShiftDefaultInterval,
  CreateShiftIntervalRequest,
  UpdateShiftIntervalRequest,
} from "./types";
import { client } from "./client";

// Paths not yet registered in the generated OpenAPI types.
// Use per-call type assertions until the OpenAPI spec is updated.

// ── Work Segments ──────────────────────────────────────────────────────────

export async function fetchWorkSegments(
  wsId: string,
  params?: {
    employment_ids?: string[];
    start_date?: string;
    end_date?: string;
    segment_type?: string;
    segment_status?: string;
    source?: string;
    page?: number;
    limit?: number;
  }
): Promise<WorkSegment[]> {
  const { data } = (await client.GET("/workspaces/{workspace_id}/work-segments", {
    params: {
      path: { workspace_id: wsId },
      query: params as unknown as Record<string, unknown>,
    },
  } as never)) as { data: { data: WorkSegment[] } };
  if (!data) throw new Error("No response data");
  return data.data;
}

export async function createWorkSegment(
  wsId: string,
  body: CreateWorkSegmentRequest
): Promise<WorkSegment> {
  const { data } = (await client.POST("/workspaces/{workspace_id}/work-segments", {
    params: { path: { workspace_id: wsId } },
    body,
  } as never)) as { data: WorkSegment };
  if (!data) throw new Error("No response data");
  return data;
}

export async function getWorkSegment(
  wsId: string,
  segmentId: string
): Promise<WorkSegment> {
  const { data } = (await client.GET(
    "/workspaces/{workspace_id}/work-segments/{segment_id}",
    {
      params: {
        path: { workspace_id: wsId, segment_id: segmentId },
      },
    } as never
  )) as { data: WorkSegment };
  if (!data) throw new Error("No response data");
  return data;
}

export async function updateWorkSegment(
  wsId: string,
  segmentId: string,
  body: UpdateWorkSegmentRequest
): Promise<WorkSegment> {
  const { data } = (await client.PUT(
    "/workspaces/{workspace_id}/work-segments/{segment_id}",
    {
      params: { path: { workspace_id: wsId, segment_id: segmentId } },
      body,
    } as never
  )) as { data: WorkSegment };
  if (!data) throw new Error("No response data");
  return data;
}

export async function deleteWorkSegment(
  wsId: string,
  segmentId: string
): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/work-segments/{segment_id}", {
    params: { path: { workspace_id: wsId, segment_id: segmentId } },
  } as never);
}

export async function bulkCreateWorkSegments(
  wsId: string,
  body: BulkCreateWorkSegmentsRequest
): Promise<WorkSegment[]> {
  const { data } = (await client.POST("/workspaces/{workspace_id}/work-segments/bulk", {
    params: { path: { workspace_id: wsId } },
    body,
  } as never)) as { data: WorkSegment[] };
  if (!data) throw new Error("No response data");
  return data;
}

export async function scheduleWorkSegments(
  wsId: string,
  params: { start_date: string; end_date: string; employment_ids?: string[] }
): Promise<{ scheduled: number; skipped: number }> {
  const { data } = (await client.GET("/workspaces/{workspace_id}/work-segments/schedule", {
    params: { path: { workspace_id: wsId }, query: params as unknown as Record<string, unknown> },
  } as never)) as { data: { scheduled: number; skipped: number } };
  if (!data) throw new Error("No response data");
  return data;
}

// ── Shift Intervals ─────────────────────────────────────────────────────────

export async function fetchShiftIntervals(
  wsId: string,
  shiftId: string
): Promise<ShiftDefaultInterval[]> {
  const { data } = (await client.GET("/workspaces/{workspace_id}/shifts/{shift_id}/intervals", {
    params: { path: { workspace_id: wsId, shift_id: shiftId } },
  } as never)) as { data: ShiftDefaultInterval[] };
  if (!data) throw new Error("No response data");
  return data;
}

export async function createShiftInterval(
  wsId: string,
  shiftId: string,
  body: CreateShiftIntervalRequest
): Promise<ShiftDefaultInterval> {
  const { data } = (await client.POST("/workspaces/{workspace_id}/shifts/{shift_id}/intervals", {
    params: { path: { workspace_id: wsId, shift_id: shiftId } },
    body,
  } as never)) as { data: ShiftDefaultInterval };
  if (!data) throw new Error("No response data");
  return data;
}

export async function updateShiftInterval(
  wsId: string,
  shiftId: string,
  intervalId: string,
  body: UpdateShiftIntervalRequest
): Promise<ShiftDefaultInterval> {
  const { data } = (await client.PUT(
    "/workspaces/{workspace_id}/shifts/{shift_id}/intervals/{interval_id}",
    {
      params: { path: { workspace_id: wsId, shift_id: shiftId, interval_id: intervalId } },
      body,
    } as never
  )) as { data: ShiftDefaultInterval };
  if (!data) throw new Error("No response data");
  return data;
}

export async function deleteShiftInterval(
  wsId: string,
  shiftId: string,
  intervalId: string
): Promise<void> {
  await client.DELETE(
    "/workspaces/{workspace_id}/shifts/{shift_id}/intervals/{interval_id}",
    {
      params: { path: { workspace_id: wsId, shift_id: shiftId, interval_id: intervalId } },
    } as never
  );
}

export async function reorderShiftIntervals(
  wsId: string,
  shiftId: string,
  intervalIds: string[]
): Promise<void> {
  await client.POST("/workspaces/{workspace_id}/shifts/{shift_id}/intervals/reorder", {
    params: { path: { workspace_id: wsId, shift_id: shiftId } },
    body: { interval_ids: intervalIds },
  } as never);
}
