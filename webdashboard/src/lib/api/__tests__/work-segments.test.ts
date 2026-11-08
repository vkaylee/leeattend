import { describe, it, expect, vi, beforeEach } from "vitest";

// work-segments.ts uses the untyped fetch/fire wrappers. Both ultimately call
// client[method] -> customFetch -> apiFetch, so mocking apiFetch intercepts them.
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
    const mock = [{ id: "seg-1", start_time: "09:00" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ segments: mock, total_count: 1 }));
    const result = await fetchWorkSegments();
    expect(result).toEqual(mock);
  });

  it("fetchWorkSegments: forwards query params", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ segments: [], total_count: 0 }));
    await fetchWorkSegments({ start_date: "2026-01-01", page: 1, limit: 20 });
    expect(apiFetch).toHaveBeenCalledTimes(1);
  });

  it("fetchWorkSegments: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(fetchWorkSegments()).rejects.toThrow();
  });

  it("createWorkSegment: POST returns segment", async () => {
    const mock = { id: "seg-new", start_time: "09:00", end_time: "17:00" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createWorkSegment({ start_time: "09:00" } as never);
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
  });

  it("getWorkSegment: GET single", async () => {
    const mock = { id: "seg-1" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await getWorkSegment("seg-1");
    expect(result).toEqual(mock);
  });

  it("updateWorkSegment: PUT returns updated", async () => {
    const mock = { id: "seg-1", end_time: "18:00" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updateWorkSegment("seg-1", { end_time: "18:00" } as never);
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("PUT");
  });

  it("deleteWorkSegment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteWorkSegment("seg-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/work-segments/seg-1");
  });

  it("bulkCreateWorkSegments: POST returns list", async () => {
    const mock = [{ id: "seg-1" }, { id: "seg-2" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await bulkCreateWorkSegments({ segments: [] } as never);
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/work-segments/bulk");
  });

  it("scheduleWorkSegments: returns scheduled/skipped", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ scheduled: 5, skipped: 1 }));
    const result = await scheduleWorkSegments({ start_date: "2026-01-01", end_date: "2026-01-31" });
    expect(result).toEqual({ scheduled: 5, skipped: 1 });
  });

  it("createWorkSegment: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(createWorkSegment({} as never)).rejects.toThrow();
  });

  it("bulkCreateWorkSegments: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(bulkCreateWorkSegments({ segments: [] } as never)).rejects.toThrow();
  });

  it("scheduleWorkSegments: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(scheduleWorkSegments({ start_date: "2026-01-01", end_date: "2026-01-31" })).rejects.toThrow();
  });

  it("fetchShiftIntervals: returns interval list", async () => {
    const mock = [{ id: "si-1", start_time: "09:00" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchShiftIntervals("shift-1");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/shifts/shift-1/intervals");
  });

  it("createShiftInterval: POST returns interval", async () => {
    const mock = { id: "si-new" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createShiftInterval("shift-1", { start_time: "09:00" } as never);
    expect(result).toEqual(mock);
  });

  it("updateShiftInterval: PUT returns interval", async () => {
    const mock = { id: "si-1", end_time: "18:00" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updateShiftInterval("shift-1", "si-1", { end_time: "18:00" } as never);
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("PUT");
  });

  it("deleteShiftInterval: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteShiftInterval("shift-1", "si-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("reorderShiftIntervals: POST forwards interval_ids", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await reorderShiftIntervals("shift-1", ["si-1", "si-2"]);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/shifts/shift-1/intervals/reorder");
    const body = JSON.parse((call[1] as RequestInit).body as string);
    expect(body.interval_ids).toEqual(["si-1", "si-2"]);
  });
});
