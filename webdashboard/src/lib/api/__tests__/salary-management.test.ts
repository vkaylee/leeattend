import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../helpers", () => ({ apiFetch: vi.fn() }));

import { apiFetch } from "../helpers";
import { fetchSalaryManagement } from "../salary-management";

function okResponse(payload: unknown): Response {
  return { ok: true, status: 200, headers: new Headers({ "content-type": "application/json" }), json: () => Promise.resolve(payload) } as unknown as Response;
}

describe("salary-management API", () => {
  beforeEach(() => vi.clearAllMocks());

  it("passes as-of filters and server pagination without exposing client-only state", async () => {
    const payload = { as_of: "2026-11-12", data: [], limit: 25, page: 2, total: 30 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okResponse(payload));

    await expect(fetchSalaryManagement({ as_of: "2026-11-12", page: 2, limit: 25, search: "EMP-7", status: "active" })).resolves.toEqual(payload);
    expect(vi.mocked(apiFetch).mock.calls[0]?.[0]).toContain("/salary-management?");
    expect(vi.mocked(apiFetch).mock.calls[0]?.[0]).toContain("as_of=2026-11-12");
    expect(vi.mocked(apiFetch).mock.calls[0]?.[0]).toContain("page=2");
    expect(vi.mocked(apiFetch).mock.calls[0]?.[0]).toContain("search=EMP-7");
  });

  it("propagates forbidden responses", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce({ ok: false, status: 403, statusText: "Forbidden", headers: new Headers(), json: () => Promise.resolve({}) } as unknown as Response);
    await expect(fetchSalaryManagement({ page: 1, limit: 25 })).rejects.toThrow();
  });
});
