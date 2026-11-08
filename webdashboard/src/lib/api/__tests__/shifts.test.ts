import { describe, it, expect, vi, beforeEach } from "vitest";

// bulkUpsertRoster bypasses the openapi client and calls apiFetch directly with
// the long report-operation timeout. Mock at the apiFetch boundary.
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
  TIMEOUT: { DEFAULT: 10000, REPORT_OPERATION: 180000 },
}));

import { apiFetch, TIMEOUT } from "../helpers";
import { bulkUpsertRoster } from "../shifts";

function okRes(payload: unknown, status = 200) {
  return {
    ok: true,
    status,
    statusText: "OK",
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(payload),
    text: () => Promise.resolve(JSON.stringify(payload)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

function errRes(status: number, body: unknown) {
  return {
    ok: false,
    status,
    statusText: "Error",
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

describe("bulkUpsertRoster", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("POSTs items to the bulk roster endpoint", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    const items = [
      { date: "2026-07-01", shift_id: "s1" },
      { date: "2026-07-02", shift_id: "s1" },
    ];
    await bulkUpsertRoster("emp-1", items);

    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toBe("/v1/employees/emp-1/rosters/bulk");
    const opts = call[1] as RequestInit;
    expect(opts.method).toBe("POST");
    expect(JSON.parse(opts.body as string).items).toEqual(items);
  });

  it("uses the long report-operation timeout, not the 10s default", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await bulkUpsertRoster("emp-1", [{ date: "2026-07-01", shift_id: null }]);

    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit & { timeout?: number };
    expect(opts.timeout).toBe(180000);
  });

  it("resolves on 202 Accepted (write durable, reprocess enqueued)", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined, 202));
    await expect(
      bulkUpsertRoster("emp-1", [{ date: "2026-07-01", shift_id: "s1" }]),
    ).resolves.toBeUndefined();
  });

  it("throws when the response is not ok", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, { key: "error.internal" }));
    await expect(
      bulkUpsertRoster("emp-1", [{ date: "2026-07-01", shift_id: "s1" }]),
    ).rejects.toThrow();
  });
});
