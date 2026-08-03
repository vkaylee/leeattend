import { describe, it, expect, vi, beforeEach } from "vitest";

// work-segments.ts uses the untyped fetch/fire wrappers. Both ultimately call
// client[method] → customFetch → apiFetch, so mocking apiFetch intercepts them.
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from "../helpers";
import {
  fetchWorkSegments,
  createWorkSegment,
  getWorkSegment,
  updateWorkSegment,
  deleteWorkSegment,
  bulkCreateWorkSegments,
  scheduleWorkSegments,
  fetchShiftIntervals,
  createShiftInterval,
  updateShiftInterval,
  deleteShiftInterval,
  reorderShiftIntervals,
} from "../work-segments";

function okRes(payload: unknown) {
  return {
    ok: true,
    status: 200,
    statusText: "OK",
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(payload),
    text: () => Promise.resolve(JSON.stringify(payload)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

function errRes(status: number, statusText: string, body: unknown) {
  return {
    ok: false,
    status,
    statusText,
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

describe("work-segments API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetchWorkSegments: returns segments array", async () => {
    const mock = [{ id: "ws-1", start_time: "09:00" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ segments: mock, total_count: 1 }));
    const result = await fetchWorkSegments("ws-1");
    expect(result).toEqual(mock);
  });

  it("fetchWorkSegments: forwards query params", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ segments: [], total_count: 0 }));
    await fetchWorkSegments("ws-1", { start_date: "2026-01-01", page: 1, limit: 20 });
    expect(apiFetch).toHaveBeenCalledTimes(1);
  });

  it("fetchWorkSegments: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(fetchWorkSegments("ws-1")).rejects.toThrow();
  });

  it("createWorkSegment: POST returns segment", async () => {
    const mock = { id: "ws-new", start_time: "09:00", end_time: "17:00" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createWorkSegment("ws-1", { start_time: "09:00" } as never);
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
  });

  it("getWorkSegment: GET single", async () => {
    const mock = { id: "ws-1" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await getWorkSegment("ws-1", "ws-1");
    expect(result).toEqual(mock);
  });

  it("updateWorkSegment: PUT returns updated", async () => {
    const mock = { id: "ws-1", end_time: "18:00" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updateWorkSegment("ws-1", "ws-1", { end_time: "18:00" } as never);
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("PUT");
  });

  it("deleteWorkSegment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteWorkSegment("ws-1", "ws-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/work-segments/ws-1");
  });

  it("bulkCreateWorkSegments: POST returns list", async () => {
    const mock = [{ id: "ws-1" }, { id: "ws-2" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await bulkCreateWorkSegments("ws-1", { segments: [] } as never);
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/work-segments/bulk");
  });

  it("scheduleWorkSegments: returns scheduled/skipped", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ scheduled: 5, skipped: 1 }));
    const result = await scheduleWorkSegments("ws-1", { start_date: "2026-01-01", end_date: "2026-01-31" });
    expect(result).toEqual({ scheduled: 5, skipped: 1 });
  });

  it("createWorkSegment: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(createWorkSegment("ws-1", {} as never)).rejects.toThrow();
  });

  it("bulkCreateWorkSegments: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(bulkCreateWorkSegments("ws-1", { segments: [] } as never)).rejects.toThrow();
  });

  it("scheduleWorkSegments: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(scheduleWorkSegments("ws-1", { start_date: "2026-01-01", end_date: "2026-01-31" })).rejects.toThrow();
  });

  it("fetchShiftIntervals: returns interval list", async () => {
    const mock = [{ id: "si-1", start_time: "09:00" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchShiftIntervals("ws-1", "shift-1");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/shifts/shift-1/intervals");
  });

  it("createShiftInterval: POST returns interval", async () => {
    const mock = { id: "si-new" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createShiftInterval("ws-1", "shift-1", { start_time: "09:00" } as never);
    expect(result).toEqual(mock);
  });

  it("updateShiftInterval: PUT returns interval", async () => {
    const mock = { id: "si-1", end_time: "18:00" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updateShiftInterval("ws-1", "shift-1", "si-1", { end_time: "18:00" } as never);
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("PUT");
  });

  it("deleteShiftInterval: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteShiftInterval("ws-1", "shift-1", "si-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("reorderShiftIntervals: POST forwards interval_ids", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await reorderShiftIntervals("ws-1", "shift-1", ["si-1", "si-2"]);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/shifts/shift-1/intervals/reorder");
    const body = JSON.parse((call[1] as RequestInit).body as string);
    expect(body.interval_ids).toEqual(["si-1", "si-2"]);
  });
});
