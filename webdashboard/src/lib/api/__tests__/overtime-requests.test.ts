import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from "../helpers";
import { overtimeRequestsApi } from "../overtime-requests";

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

describe("overtimeRequestsApi", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("listRequests: happy path returns paginated requests", async () => {
    const mock = { data: [{ id: "or-1" }], total: 1, page: 1, limit: 50 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await overtimeRequestsApi.listRequests();
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/overtime/requests");
  });

  it("listRequests: forwards query params", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: [], total: 0 }));
    await overtimeRequestsApi.listRequests({ status: "pending" } as never);
    expect(apiFetch).toHaveBeenCalledTimes(1);
  });

  it("listRequests: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(overtimeRequestsApi.listRequests()).rejects.toThrow();
  });

  it("getRequest: happy path returns detail", async () => {
    const mock = { id: "or-1", status: "pending_approval" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await overtimeRequestsApi.getRequest("or-1");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/overtime/requests/or-1");
  });

  it("getRequest: ApiError on 404", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(404, "Not Found", {}));
    await expect(overtimeRequestsApi.getRequest("or-1")).rejects.toThrow();
  });

  it("createRequest: POST forwards payload", async () => {
    const mock = { id: "or-new", status: "draft" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const payload = {
      employees: ["e1"],
      planned_date: "2026-01-01",
      planned_minutes: 60,
      reason: "deadline",
    };
    const result = await overtimeRequestsApi.createRequest(payload);
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
    expect(JSON.parse(opts.body as string).reason).toBe("deadline");
  });

  it("submitRequest: POST submit", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await overtimeRequestsApi.submitRequest("or-1");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/overtime/requests/or-1/submit");
  });

  it("cancelRequest: PUT cancel", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await overtimeRequestsApi.cancelRequest("or-1");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/overtime/requests/or-1/cancel");
  });

  it("employeeConsent: POST forwards emp_id path + body", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await overtimeRequestsApi.employeeConsent("or-1", {
      employment_id: "emp-1",
      status: "accepted",
      decline_reason: "none",
    });
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/overtime/requests/or-1/consent/emp-1");
    const body = JSON.parse((call[1] as RequestInit).body as string);
    expect(body.status).toBe("accepted");
    expect(body.decline_reason).toBe("none");
  });

  it("employeeConsent: ApiError on 403", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(403, "Forbidden", {}));
    await expect(
      overtimeRequestsApi.employeeConsent("or-1", { employment_id: "emp-1", status: "declined" }),
    ).rejects.toThrow();
  });

  it("getMyAssignments: returns assignment list", async () => {
    const mock = [{ id: "oe-1", status: "pending" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await overtimeRequestsApi.getMyAssignments("emp-1");
    expect(result).toEqual(mock);
  });

  it("getWorkspaceQuotaSummary: returns quota list", async () => {
    const mock = [{ employee_id: "e1", monthly_used: 10 }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await overtimeRequestsApi.getWorkspaceQuotaSummary("2026-03");
    expect(result).toEqual(mock);
  });

  it("getEmployeeQuota: returns quota", async () => {
    const mock = { employee_id: "e1", yearly_used: 5 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await overtimeRequestsApi.getEmployeeQuota("emp-1", "2026-03-01");
    expect(result).toEqual(mock);
  });

  it("listRequestBehalfAssignments: returns data array", async () => {
    const mock = [{ id: "ba-1", assignee_user_id: "u1" }];
    // Implementation accesses resData.data.
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: mock }));
    const result = await overtimeRequestsApi.listRequestBehalfAssignments();
    expect(result).toEqual(mock);
  });

  it("createRequestBehalfAssignment: POST returns record", async () => {
    const mock = { id: "ba-new", assignee_user_id: "u1" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await overtimeRequestsApi.createRequestBehalfAssignment({
      assignee_user_id: "u1",
    } as never);
    expect(result).toEqual(mock);
  });

  it("deleteRequestBehalfAssignment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await overtimeRequestsApi.deleteRequestBehalfAssignment("ba-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });
});
