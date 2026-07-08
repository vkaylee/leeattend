import type {
  WorkSegment,
  CreateWorkSegmentRequest,
  UpdateWorkSegmentRequest,
  BulkCreateWorkSegmentsRequest,
  ShiftDefaultInterval,
  CreateShiftIntervalRequest,
  UpdateShiftIntervalRequest,
} from "./types";

import { http } from "./untyped";

// Paths not yet registered in the generated OpenAPI types.
// Use the untyped `http` accessor (single `as any` cast at the boundary)
// so call-sites stay fully typed and eslint stays quiet.

async function getJson<T>(p: Promise<unknown>): Promise<T> {
  const res = (await p) as { data: T };
  if (!res.data) throw new Error("No response data");
  return res.data;
}

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
  return getJson<{ data: WorkSegment[] }>(
    http.GET("/workspaces/{workspace_id}/work-segments", {
      params: {
        path: { workspace_id: wsId },
        query: params as unknown as Record<string, unknown>,
      },
    }),
  ).then(r => r.data);
}

export async function createWorkSegment(
  wsId: string,
  body: CreateWorkSegmentRequest
): Promise<WorkSegment> {
  return getJson<WorkSegment>(http.POST("/workspaces/{workspace_id}/work-segments", {
    params: { path: { workspace_id: wsId } },
    body: body as unknown as Record<string, unknown>,
  }));
}

export async function getWorkSegment(
  wsId: string,
  segmentId: string
): Promise<WorkSegment> {
  return getJson<WorkSegment>(http.GET(
    "/workspaces/{workspace_id}/work-segments/{segment_id}",
    { params: { path: { workspace_id: wsId, segment_id: segmentId } } },
  ));
}

export async function updateWorkSegment(
  wsId: string,
  segmentId: string,
  body: UpdateWorkSegmentRequest
): Promise<WorkSegment> {
  return getJson<WorkSegment>(http.PUT(
    "/workspaces/{workspace_id}/work-segments/{segment_id}",
    {
      params: { path: { workspace_id: wsId, segment_id: segmentId } },
      body: body as unknown as Record<string, unknown>,
    },
  ));
}

export async function deleteWorkSegment(
  wsId: string,
  segmentId: string
): Promise<void> {
  await http.DELETE("/workspaces/{workspace_id}/work-segments/{segment_id}", {
    params: { path: { workspace_id: wsId, segment_id: segmentId } },
  });
}

export async function bulkCreateWorkSegments(
  wsId: string,
  body: BulkCreateWorkSegmentsRequest
): Promise<WorkSegment[]> {
  return getJson<WorkSegment[]>(http.POST("/workspaces/{workspace_id}/work-segments/bulk", {
    params: { path: { workspace_id: wsId } },
    body: body as unknown as Record<string, unknown>,
  }));
}

export async function scheduleWorkSegments(
  wsId: string,
  params: { start_date: string; end_date: string; employment_ids?: string[] }
): Promise<{ scheduled: number; skipped: number }> {
  return getJson<{ scheduled: number; skipped: number }>(
    http.GET("/workspaces/{workspace_id}/work-segments/schedule", {
      params: {
        path: { workspace_id: wsId },
        query: params as unknown as Record<string, unknown>,
      },
    }),
  );
}

// ── Shift Intervals ─────────────────────────────────────────────────────────

export async function fetchShiftIntervals(
  wsId: string,
  shiftId: string
): Promise<ShiftDefaultInterval[]> {
  return getJson<ShiftDefaultInterval[]>(
    http.GET("/workspaces/{workspace_id}/shifts/{shift_id}/intervals", {
      params: { path: { workspace_id: wsId, shift_id: shiftId } },
    }),
  );
}

export async function createShiftInterval(
  wsId: string,
  shiftId: string,
  body: CreateShiftIntervalRequest
): Promise<ShiftDefaultInterval> {
  return getJson<ShiftDefaultInterval>(http.POST(
    "/workspaces/{workspace_id}/shifts/{shift_id}/intervals",
    {
      params: { path: { workspace_id: wsId, shift_id: shiftId } },
      body: body as unknown as Record<string, unknown>,
    },
  ));
}

export async function updateShiftInterval(
  wsId: string,
  shiftId: string,
  intervalId: string,
  body: UpdateShiftIntervalRequest
): Promise<ShiftDefaultInterval> {
  return getJson<ShiftDefaultInterval>(http.PUT(
    "/workspaces/{workspace_id}/shifts/{shift_id}/intervals/{interval_id}",
    {
      params: { path: { workspace_id: wsId, shift_id: shiftId, interval_id: intervalId } },
      body: body as unknown as Record<string, unknown>,
    },
  ));
}

export async function deleteShiftInterval(
  wsId: string,
  shiftId: string,
  intervalId: string
): Promise<void> {
  await http.DELETE(
    "/workspaces/{workspace_id}/shifts/{shift_id}/intervals/{interval_id}",
    { params: { path: { workspace_id: wsId, shift_id: shiftId, interval_id: intervalId } } },
  );
}

export async function reorderShiftIntervals(
  wsId: string,
  shiftId: string,
  intervalIds: string[]
): Promise<void> {
  await http.POST("/workspaces/{workspace_id}/shifts/{shift_id}/intervals/reorder", {
    params: { path: { workspace_id: wsId, shift_id: shiftId } },
    body: { interval_ids: intervalIds } as unknown as Record<string, unknown>,
  });
}
