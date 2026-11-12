import { describe, it, expect, vi, beforeEach } from "vitest";

// Intercept attendance requests at apiFetch boundary.
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
  TIMEOUT: { REPORT_OPERATION: 180000 },
}));

import {
  punchIn,
  punchOut,
  submitAttendanceAdjustment,
  fetchAttendanceAdjustments,
  approveAttendanceAdjustment,
  fetchRawAttendanceLogs,
  uploadAttendance,
  fetchLatestImportJob,
  fetchImportJobStatus,
  reprocessAttendance,
  reprocessAllAttendance,
  checkReprocessStatus,
  approveOt,
  fetchAttendanceSummaries,
  fetchAttendanceSummariesPaginated,
  enterPaperOt,
  bulkUploadPaperOt,
  fetchStagingLogs,
  resolveStagingLog,
  dismissStagingLog,
  createReportExportJob,
  fetchExportJobStatus,
  cancelExportJob,
  downloadExportReport,
  fetchMonitorOverview,
  fetchMonitorJobs,
  fetchMonitorAlerts,
  fetchMonitorAlert,
  acknowledgeMonitorAlert,
  resolveMonitorAlert,
  fetchDefaultAttendanceRules,
  fetchAttendancePolicyVersions,
  fetchPolicyAssignments,
  upsertPolicyAssignment,
  deletePolicyAssignment,
  updateDefaultAttendanceRules,
  fetchBehalfAssignments,
  createBehalfAssignment,
  deleteBehalfAssignment,
  claimOt,
  fetchAdjustmentBehalfAssignments,
  createAdjustmentBehalfAssignment,
  deleteAdjustmentBehalfAssignment,
  adminConfirmAttendance,
} from "../attendance";
import { apiFetch } from "../helpers";

function okRes(payload: unknown) {
  return {
    ok: true,
    status: 200,
    statusText: "OK",
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(payload),
    text: () => Promise.resolve(JSON.stringify(payload)),
    blob: () => Promise.resolve(new Blob()),
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

describe("attendance API (extended)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("punchIn: POST returns TimeEntry", async () => {
    const mock = { id: "te-1", employment_id: "e1", punch_in: "09:00:00" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await punchIn({ employment_id: "e1" });
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).employment_id).toBe("e1");
  });

  it("punchIn: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(punchIn({ employment_id: "e1" })).rejects.toThrow();
  });

  it("punchOut: POST forwards employee_id + note", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "te-2" }));
    const result = await punchOut({ employee_id: "e1", note: "bye" });
    expect(result).toEqual({ id: "te-2" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/time/punch-out/e1");
  });

  it("submitAttendanceAdjustment: POST returns adjustment", async () => {
    const mock = { id: "adj-1", status: "pending" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await submitAttendanceAdjustment({
      employment_id: "e1",
      requested_punch_time: "2026-01-01T09:00:00Z",
    });
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).employment_id).toBe("e1");
  });

  it("fetchAttendanceAdjustments: paginated", async () => {
    const mock = { data: [{ id: "adj-1" }], total: 1 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchAttendanceAdjustments({ status: "pending", page: 1 });
    expect(result).toEqual(mock);
  });

  it("approveAttendanceAdjustment: PUT approve", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await approveAttendanceAdjustment("adj-1", { status: "approved" });
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/attendance/adjustments/adj-1/approve");
    expect((call[1] as RequestInit).method).toBe("PUT");
  });

  it("fetchRawAttendanceLogs: GET", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: [], total: 0, page: 1, limit: 20 }));
    const result = await fetchRawAttendanceLogs({});
    expect(result.total).toBe(0);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/attendance/raw-logs");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).not.toContain("/workspaces/");
  });

  it("uploadAttendance: POST /employees/upload branch", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ job_id: "j-1" }));
    const file = new File(["x"], "a.xlsx");
    const result = await uploadAttendance(file, "/employees/upload");
    expect(result).toEqual({ job_id: "j-1" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/employees/upload");
  });

  it("uploadAttendance: POST /attendance/upload branch", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ job_id: "j-2" }));
    const file = new File(["x"], "a.xlsx");
    const result = await uploadAttendance(file, "/others/upload");
    expect(result).toEqual({ job_id: "j-2" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/attendance/upload");
  });

  it("fetchLatestImportJob: GET flat latest route", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "j-1", status: "completed" }));
    const result = await fetchLatestImportJob("log");
    expect(result).toEqual({ id: "j-1", status: "completed" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/attendance/import-jobs/latest");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).not.toContain("/workspaces/");
  });

  it("fetchImportJobStatus: GET flat job route", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "j-1", status: "processing" }));
    const result = await fetchImportJobStatus("j-1");
    expect(result).toEqual({ id: "j-1", status: "processing" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/attendance/import-jobs/j-1");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).not.toContain("/workspaces/");
  });

  it("reprocessAttendance: POST with date range", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await reprocessAttendance("e1", "2026-01-01", "2026-01-31");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/attendance/reprocess/e1");
  });

  it("reprocessAllAttendance: POST all in range", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ message: "ok", job_id: "j-1" }));
    const result = await reprocessAllAttendance("2026-01-01", "2026-01-31");
    expect(result).toEqual({ message: "ok", job_id: "j-1" });
  });

  it("checkReprocessStatus: GET status", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ job_id: "j-1", status: "completed" }));
    const result = await checkReprocessStatus("j-1");
    expect(result).toEqual({ job_id: "j-1", status: "completed" });
  });

  it("approveOt: POST approve OT", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await approveOt("e1", { date: "2026-01-01", approved_minutes: 60, status: "approved" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/attendance/approve-ot/e1");
  });

  it("fetchAttendanceSummaries: returns array when array", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes([{ id: "s-1" }]));
    const result = await fetchAttendanceSummaries("2026-01-01", "2026-01-31");
    expect(result).toEqual([{ id: "s-1" }]);
  });

  it("fetchAttendanceSummaries: unwraps .data when object", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: [{ id: "s-1" }] }));
    const result = await fetchAttendanceSummaries("2026-01-01", "2026-01-31");
    expect(result).toEqual([{ id: "s-1" }]);
  });

  it("fetchAttendanceSummaries: empty fallback when object without data", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ meta: "x" }));
    const result = await fetchAttendanceSummaries("2026-01-01", "2026-01-31");
    expect(result).toEqual([]);
  });

  it("fetchAttendanceSummariesPaginated: GET paginated", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: [{ id: "s-1" }], total: 1 }));
    const result = await fetchAttendanceSummariesPaginated("2026-01-01", "2026-01-31", {
      page: 1,
      limit: 20,
    });
    expect(result.total).toBe(1);
  });

  it("enterPaperOt: POST returns summary", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "s-1" }));
    const result = await enterPaperOt("e1", { date: "2026-01-01", claimed_minutes: 60, reason: "x" });
    expect(result).toEqual({ id: "s-1" });
  });

  it("bulkUploadPaperOt: POST FormData returns summary", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ total_rows: 1, success_count: 1, failed_count: 0, errors: [] }));
    const file = new File(["x"], "ot.xlsx");
    const result = await bulkUploadPaperOt(file);
    expect(result.total_rows).toBe(1);
  });

  it("fetchStagingLogs: GET flat paginated route", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: [], total: 0, page: 1, limit: 50 }));
    const result = await fetchStagingLogs(1, 50);
    expect(result.total).toBe(0);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/attendance/staging-logs");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).not.toContain("/workspaces/");
  });

  it("resolveStagingLog: POST flat resolve route", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await resolveStagingLog("sl-1", "e1");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/attendance/staging-logs/sl-1/resolve");
    expect(call[0]).not.toContain("/workspaces/");
    expect(JSON.parse((call[1] as RequestInit).body as string).employee_id).toBe("e1");
  });

  it("dismissStagingLog: DELETE flat route", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await dismissStagingLog("sl-1");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/attendance/staging-logs/sl-1");
    expect(call[0]).not.toContain("/workspaces/");
    expect((call[1] as RequestInit).method).toBe("DELETE");
  });

  it("createReportExportJob: POST returns export response", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ job_id: "j-1" }));
    const result = await createReportExportJob({ format: "xlsx" } as never);
    expect(result).toEqual({ job_id: "j-1" });
  });

  it("fetchExportJobStatus: GET status", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ job_id: "j-1", status: "completed" }));
    const result = await fetchExportJobStatus("j-1");
    expect(result).toEqual({ job_id: "j-1", status: "completed" });
  });

  it("cancelExportJob: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await cancelExportJob("j-1");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/reports/jobs/j-1");
    expect((call[1] as RequestInit).method).toBe("DELETE");
  });

  it("downloadExportReport: GET returns blob", async () => {
    const blob = new Blob(["xlsx"]);
    vi.mocked(apiFetch).mockResolvedValueOnce({
      ok: true,
      status: 200,
      blob: () => Promise.resolve(blob),
    } as unknown as Response);
    const result = await downloadExportReport("j-1");
    expect(result).toBe(blob);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/reports/jobs/j-1/download");
  });

  it("downloadExportReport: throws on non-ok", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce({
      ok: false,
      status: 500,
      blob: () => Promise.resolve(new Blob()),
    } as unknown as Response);
    await expect(downloadExportReport("j-1")).rejects.toThrow("[500]");
  });

  it("fetchMonitorOverview: GET global", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ active_jobs: 3 }));
    const result = await fetchMonitorOverview();
    expect(result).toEqual({ active_jobs: 3 });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/api/v1/monitor/overview");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).not.toContain("/api/monitor/");
  });

  it("fetchMonitorAlerts: uses versioned path and query params", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ alerts: [] }));
    await fetchMonitorAlerts({ cursor: "cursor-1", limit: 20, severity: "high", status: "active" });
    const [url, options] = vi.mocked(apiFetch).mock.calls[0];
    expect(url).toContain("/api/v1/monitor/alerts?");
    expect(url).toContain("cursor=cursor-1");
    expect(url).toContain("limit=20");
    expect(url).toContain("severity=high");
    expect(url).toContain("status=active");
    expect((options as RequestInit).method).toBe("GET");
  });

  it("fetchMonitorAlert: uses versioned detail path", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "alert-1" }));
    await fetchMonitorAlert("alert-1");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/api/v1/monitor/alerts/alert-1");
  });

  it("monitor alert mutations: use versioned paths and POST bodies", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "alert-1" }));
    await acknowledgeMonitorAlert("alert-1", { reason: "ack" });
    let [url, options] = vi.mocked(apiFetch).mock.calls[0];
    expect(url).toContain("/api/v1/monitor/alerts/alert-1/acknowledge");
    expect((options as RequestInit).method).toBe("POST");
    expect(JSON.parse((options as RequestInit).body as string)).toEqual({ reason: "ack" });

    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "alert-1" }));
    await resolveMonitorAlert("alert-1", { reason: "resolved" });
    [url, options] = vi.mocked(apiFetch).mock.calls[1];
    expect(url).toContain("/api/v1/monitor/alerts/alert-1/resolve");
    expect((options as RequestInit).method).toBe("POST");
    expect(JSON.parse((options as RequestInit).body as string)).toEqual({ reason: "resolved" });
  });

  it("fetchMonitorJobs: marks all sources unavailable when response is null", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(null));
    const result = await fetchMonitorJobs({ limit: 10 });
    expect(result.jobs).toEqual([]);
    expect(result.next_cursor).toBeNull();
    expect(result.durable_jobs.status).toBe("unavailable");
    expect(result.durable_jobs.jobs).toBeNull();
    expect(result.quality.data_quality).toBe("unavailable");
  });

  it("fetchMonitorJobs: GET returns jobs", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ jobs: [{ id: "j-1" }], next_cursor: 5 }));
    const result = await fetchMonitorJobs({ limit: 10 });
    expect(result.jobs).toHaveLength(1);
    expect(result.next_cursor).toBe(5);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/api/v1/monitor/jobs?limit=10");
  });

  it("fetchDefaultAttendanceRules: GET rules", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ rounding_policy: "floor", work_unit_minutes: 15 }));
    const result = await fetchDefaultAttendanceRules();
    expect(result).toEqual({ rounding_policy: "floor", work_unit_minutes: 15 });
  });

  it("fetchAttendancePolicyVersions: GET list", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes([{ id: "v1" }]));
    const result = await fetchAttendancePolicyVersions();
    expect(result).toEqual([{ id: "v1" }]);
  });

  it("fetchPolicyAssignments: GET paginated list", async () => {
    const payload = { data: [{ id: "pa-1" }], total: 1, page: 1, limit: 50 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(payload));
    const result = await fetchPolicyAssignments();
    expect(result).toEqual(payload);
  });

  it("upsertPolicyAssignment: POST returns assignment", async () => {
    const mock = { id: "pa-new", target_type: "USER" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await upsertPolicyAssignment({
      early_arrival_strategy: "allow",
      rounding_policy: "floor",
      target_id: "e1",
      target_type: "USER",
      work_unit_minutes: 15,
    });
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).target_type).toBe("USER");
  });

  it("deletePolicyAssignment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deletePolicyAssignment("pa-1");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/policies/assignments/pa-1");
    expect((call[1] as RequestInit).method).toBe("DELETE");
  });

  it("updateDefaultAttendanceRules: PATCH returns version", async () => {
    const mock = { id: "v2" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updateDefaultAttendanceRules({ rounding_policy: "ceil" });
    expect(result).toEqual(mock);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect((call[1] as RequestInit).method).toBe("PATCH");
  });

  it("fetchBehalfAssignments: GET list", async () => {
    const mock = [{ id: "ba-1" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchBehalfAssignments();
    expect(result).toEqual([{ id: "ba-1" }]);
  });

  it("createBehalfAssignment: POST returns record", async () => {
    const mock = { id: "ba-new" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "ba-new" }));
    const result = await createBehalfAssignment({ assignee_user_id: "u1" });
    expect(result).toEqual({ id: "ba-new" });
  });

  it("deleteBehalfAssignment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteBehalfAssignment("ba-1");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect((call[1] as RequestInit).method).toBe("DELETE");
  });

  it("claimOt: POST claim", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await claimOt("e1", { date: "2026-01-01", claimed_minutes: 60, reason: "x" });
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/attendance/claim-ot/e1");
    expect(JSON.parse((call[1] as RequestInit).body as string).claimed_minutes).toBe(60);
  });

  it("fetchAdjustmentBehalfAssignments: GET", async () => {
    const mock = [{ id: "ba-1" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchAdjustmentBehalfAssignments();
    expect(result).toEqual([{ id: "ba-1" }]);
  });

  it("createAdjustmentBehalfAssignment: POST", async () => {
    const mock = { id: "ba-new" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "ba-new" }));
    const result = await createAdjustmentBehalfAssignment({ assignee_user_id: "u1" });
    expect(result).toEqual({ id: "ba-new" });
  });

  it("deleteAdjustmentBehalfAssignment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteAdjustmentBehalfAssignment("ba-1");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect((call[1] as RequestInit).method).toBe("DELETE");
  });

  // --- adminConfirmAttendance (lines 748-780) -----------------------------
  // Two branches: with existingRequestId (single approve) and without (creates
  // adjustments via submitAttendanceAdjustment, then approves). The no-request
  // branch computes ISO datetimes from row fields — we feed a row whose
  // actual_punch_in already looks ISO so toDatetime returns it unchanged.
  it("adminConfirmAttendance: with existingRequestId approves once", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    const row = {
      employment_id: "e1",
      date: "2026-01-01",
      actual_punch_in: "2026-01-01T09:00:00Z",
      actual_punch_out: "2026-01-01T17:00:00Z",
    } as never;
    await adminConfirmAttendance(row, "adj-existing");
    // Only one approve call — no submitAttendanceAdjustment calls.
    expect(apiFetch).toHaveBeenCalledTimes(1);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/attendance/adjustments/adj-existing/approve");
    expect((call[1] as RequestInit).method).toBe("PUT");
  });

  it("adminConfirmAttendance: without requestId creates + approves (punchOut present)", async () => {
    // submitAttendanceAdjustment (punch in) -> submitAttendanceAdjustment (punch out)
    // -> approve (punch out) -> approve (punch in)
    vi.mocked(apiFetch)
      .mockResolvedValueOnce(okRes({ id: "adj-in" })) // submit punch in
      .mockResolvedValueOnce(okRes({ id: "adj-out" })) // submit punch out
      .mockResolvedValueOnce(okRes(undefined)) // approve punch out
      .mockResolvedValueOnce(okRes(undefined)); // approve punch in
    const row = {
      employment_id: "e1",
      date: "2026-01-01",
      actual_punch_in: "2026-01-01T09:00:00Z",
      actual_punch_out: "2026-01-01T17:00:00Z",
    } as never;
    await adminConfirmAttendance(row, undefined, "Admin confirmation");
    expect(apiFetch).toHaveBeenCalledTimes(4);
  });

  it("adminConfirmAttendance: without requestId, no punchOut -> single adjustment", async () => {
    vi.mocked(apiFetch)
      .mockResolvedValueOnce(okRes({ id: "adj-in" })) // submit punch in
      .mockResolvedValueOnce(okRes(undefined)); // approve punch in
    const row = {
      employment_id: "e1",
      date: "2026-01-01",
      actual_punch_in: "2026-01-01T09:00:00Z",
      actual_punch_out: null,
    } as never;
    await adminConfirmAttendance(row);
    expect(apiFetch).toHaveBeenCalledTimes(2);
  });
});
