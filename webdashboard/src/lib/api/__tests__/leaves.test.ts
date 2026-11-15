import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";

vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from "../helpers";
import {
  submitLeaveRequest,
  fetchLeaveRequests,
  fetchMyLeaveRequests,
  approveLeaveRequest,
  fetchEmployeeLeaveBalances,
  fetchLeaveBalances,
  fetchLeaveHistory,
  adjustLeaveBalance,
  fetchLeavePolicies,
  createLeavePolicy,
  updateLeavePolicy,
  deleteLeavePolicy,
  fetchLeaveTypes,
  createLeaveType,
  deleteLeaveType,
  fetchLeavePolicyAssignments,
  assignLeavePolicy,
  deleteLeavePolicyAssignment,
  triggerAccrualRun,
  fetchJobStatus,
  downloadLeaveImportTemplate,
  downloadLeaveImportErrors,
  updateLeaveTypeConfig,
  uploadLeaveExcel,
  fetchLatestLeaveImport,
  fetchLeaveImportStatus,
  getLeaveImportErrorsUrl,
  getLeaveImportTemplateUrl,
  fetchLeaveBehalfAssignments,
  createLeaveBehalfAssignment,
  deleteLeaveBehalfAssignment,
} from "../leaves";

beforeAll(() => {
  if (typeof window !== "undefined" && !window.URL.createObjectURL) {
    window.URL.createObjectURL = () => "blob:mock";
    window.URL.revokeObjectURL = () => {};
  }
});

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

describe("leaves API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("submitLeaveRequest: POST returns LeaveRequest", async () => {
    const mock = { id: "lr-1", status: "pending" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await submitLeaveRequest({
      employment_id: "emp-1",
      leave_type_id: "lt-1",
      start_date: "2026-01-01",
      end_date: "2026-01-02",
    });
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
    expect(JSON.parse(opts.body as string).employment_id).toBe("emp-1");
  });

  it("submitLeaveRequest: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(
      submitLeaveRequest({ employment_id: "e", leave_type_id: "l", start_date: "s", end_date: "e" }),
    ).rejects.toThrow();
  });

  it("fetchLeaveRequests: paginated", async () => {
    const mock = { data: [{ id: "lr-1" }], total: 1, page: 1, limit: 50 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLeaveRequests({ status: "pending", page: 1, limit: 50 });
    expect(result).toEqual(mock);
  });

  it("fetchLeaveRequests: normalizes legacy array response", async () => {
    const mock = [{ id: "lr-legacy" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));

    await expect(fetchLeaveRequests({ page: 2, limit: 20 })).resolves.toEqual({
      data: mock,
      total: 1,
      page: 2,
      limit: 20,
    });
  });

  it("fetchLeaveRequests: employee_id 'all' becomes undefined", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: [], total: 0 }));
    await fetchLeaveRequests({ employee_id: "all" });
    expect(apiFetch).toHaveBeenCalledTimes(1);
  });

  it("fetchMyLeaveRequests: GET list", async () => {
    const mock = [{ id: "lr-1" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchMyLeaveRequests("emp-1");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/employees/emp-1/leave/requests");
  });

  it("approveLeaveRequest: POST approve", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await approveLeaveRequest("lr-1", { decision: "approved" } as never);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/leave/requests/lr-1/approve");
  });

  it("fetchEmployeeLeaveBalances: GET list", async () => {
    const mock = [{ leave_type_id: "lt-1", balance: 10 }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchEmployeeLeaveBalances("emp-1");
    expect(result).toEqual(mock);
  });

  it("fetchLeaveBalances: paginated with q/year", async () => {
    const mock = { data: [], total: 0 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLeaveBalances({ q: "bob", year: 2026 });
    expect(result).toEqual(mock);
  });

  it("fetchLeaveHistory: GET with optional leave_type_id", async () => {
    const mock = [{ id: "lr-1", date: "2026-01-01" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLeaveHistory("emp-1", "lt-1");
    expect(result).toEqual(mock);
  });

  it("fetchLeaveHistory: GET without leave_type_id", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes([]));
    const result = await fetchLeaveHistory("emp-1");
    expect(result).toEqual([]);
  });

  it("adjustLeaveBalance: POST adjust", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await adjustLeaveBalance("emp-1", {
      leave_type_id: "lt-1",
      amount: 1,
      reason: "correction",
      transaction_date: "2026-01-01",
    });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
    expect(JSON.parse(opts.body as string).amount).toBe(1);
  });

  it("fetchLeavePolicies: GET list", async () => {
    const mock = [{ id: "lp-1", name: "Annual" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLeavePolicies();
    expect(result).toEqual(mock);
  });

  it("createLeavePolicy: POST returns policy", async () => {
    const mock = { id: "lp-new", name: "Sick" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createLeavePolicy({
      leave_type_id: "lt-1",
      name: "Sick",
      rule_config: {} as never,
    });
    expect(result).toEqual(mock);
  });

  it("updateLeavePolicy: PATCH returns policy", async () => {
    const mock = { id: "lp-1", name: "Updated" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updateLeavePolicy("lp-1", { name: "Updated" });
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("PATCH");
  });

  it("deleteLeavePolicy: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteLeavePolicy("lp-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("fetchLeaveTypes: GET list", async () => {
    const mock = [{ id: "lt-1", name: "Annual", code: "ANN" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLeaveTypes();
    expect(result).toEqual(mock);
  });

  it("createLeaveType: POST returns type", async () => {
    const mock = { id: "lt-new", name: "Compassionate" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createLeaveType({ code: "COM", is_paid: true, name: "Compassionate" });
    expect(result).toEqual(mock);
  });

  it("deleteLeaveType: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteLeaveType("lt-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("fetchLeavePolicyAssignments: GET list", async () => {
    const mock = [{ id: "pa-1", policy_id: "lp-1" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLeavePolicyAssignments();
    expect(result).toEqual(mock);
  });

  it("assignLeavePolicy: POST assign", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await assignLeavePolicy({
      assignee_type: "employee",
      assignee_id: "emp-1",
      policy_id: "lp-1",
      effective_from: "2026-01-01",
    });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).assignee_type).toBe("employee");
  });

  it("deleteLeavePolicyAssignment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteLeavePolicyAssignment("pa-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("triggerAccrualRun: POST returns job info", async () => {
    const mock = { job_id: "j-1", status: "pending", message: "started" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await triggerAccrualRun({ date: "2026-01-01" });
    expect(result).toEqual(mock);
  });

  it("fetchJobStatus: GET status", async () => {
    const mock = { job_id: "j-1", status: "completed", completed_at: "2026-01-01T00:00:00Z", error_message: null };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchJobStatus("j-1");
    expect(result).toEqual(mock);
  });

  it("updateLeaveTypeConfig: PUT returns type", async () => {
    const mock = { id: "lt-1", name: "Annual" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updateLeaveTypeConfig("lt-1", { max_days: 12 } as never);
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("PUT");
    expect(JSON.parse(opts.body as string).policy_config).toEqual({ max_days: 12 });
  });

  it("uploadLeaveExcel: POST FormData returns job_id", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ job_id: "j-1", status: "pending" }));
    const file = new File(["x"], "leave.xlsx");
    const result = await uploadLeaveExcel(file);
    expect(result).toEqual({ job_id: "j-1", status: "pending" });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
    expect(opts.body).toBeInstanceOf(FormData);
  });

  it("fetchLatestLeaveImport: returns import job or null", async () => {
    const mock = { id: "j-1", status: "completed", total_rows: 10, processed_rows: 10 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLatestLeaveImport();
    expect(result).toEqual(mock);
  });

  it("fetchLatestLeaveImport: returns null when server sends null", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(null));
    const result = await fetchLatestLeaveImport();
    expect(result).toBeNull();
  });

  it("fetchLeaveImportStatus: returns import job or null", async () => {
    const mock = { id: "j-1", status: "failed" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLeaveImportStatus();
    expect(result).toEqual(mock);
  });

  it("getLeaveImportErrorsUrl: builds URL", () => {
    expect(getLeaveImportErrorsUrl("j-1")).toContain("/leave/import/j-1/errors");
  });

  it("getLeaveImportTemplateUrl: builds URL with lang", () => {
    expect(getLeaveImportTemplateUrl("vi")).toContain("/leave/upload-template?lang=vi");
    expect(getLeaveImportTemplateUrl()).toContain("lang=en");
  });

  it("fetchLeaveBehalfAssignments: GET list", async () => {
    const mock = [{ id: "ba-1" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchLeaveBehalfAssignments();
    expect(result).toEqual(mock);
  });

  it("createLeaveBehalfAssignment: POST returns record", async () => {
    const mock = { id: "ba-new" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createLeaveBehalfAssignment({ assignee_user_id: "u1" } as never);
    expect(result).toEqual(mock);
  });

  it("deleteLeaveBehalfAssignment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteLeaveBehalfAssignment("ba-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("downloadLeaveImportTemplate: happy path", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce({
      ok: true,
      status: 200,
      blob: () => Promise.resolve(new Blob(["xlsx"])),
    } as unknown as Response);
    await expect(downloadLeaveImportTemplate("vi")).resolves.toBeUndefined();
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/leave/upload-template?lang=vi");
  });

  it("downloadLeaveImportTemplate: throws on non-ok", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce({
      ok: false,
      status: 500,
    } as unknown as Response);
    await expect(downloadLeaveImportTemplate()).rejects.toThrow("Failed to download template");
  });

  it("downloadLeaveImportErrors: happy path", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce({
      ok: true,
      status: 200,
      blob: () => Promise.resolve(new Blob(["xlsx"])),
    } as unknown as Response);
    await expect(downloadLeaveImportErrors("j-1")).resolves.toBeUndefined();
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/leave/import/j-1/errors");
  });

  it("downloadLeaveImportErrors: throws on non-ok", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce({
      ok: false,
      status: 404,
    } as unknown as Response);
    await expect(downloadLeaveImportErrors("j-1")).rejects.toThrow("[404]");
  });
});
