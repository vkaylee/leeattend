import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  useAttendanceSummaries,
  useAttendanceSummariesInfinite,
  useApproveOt,
  useEnterPaperOt,
  useBulkUploadPaperOt,
  useClaimOt,
  attendanceKeys,
} from "../use-attendance";

// ── Mocks ────────────────────────────────────────────────────────────────────

const mockUseWorkspace = vi.hoisted(() => vi.fn());

vi.mock("@/hooks/use-workspace", () => ({
  useWorkspace: mockUseWorkspace,
}));

const api = vi.hoisted(() => ({
  fetchAttendanceSummaries: vi.fn(),
  fetchAttendanceSummariesPaginated: vi.fn(),
  approveOt: vi.fn(),
  enterPaperOt: vi.fn(),
  bulkUploadPaperOt: vi.fn(),
  claimOt: vi.fn(),
  getStandaloneOrganizationId: vi.fn(() => "standalone"),
}));

vi.mock("@/lib/api", () => api);

vi.mock("@/lib/api/helpers", () => ({
  getStandaloneOrganizationId: () => api.getStandaloneOrganizationId(),
}));

const WS_ID = "standalone";
const withWorkspace = { workspace: { id: WS_ID }, isLoading: false, isError: false };
const withoutWorkspace = { workspace: undefined, isLoading: false, isError: false };

// ── Helpers ──────────────────────────────────────────────────────────────────

let queryClient: QueryClient;

function createWrapper() {
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

const MOCK_SUMMARY: {
  date: string;
  employee_id: string;
  employee_name: string;
  scheduled_seconds: number;
  worked_seconds: number;
  overtime_seconds: number;
  late_seconds: number;
  early_departure_seconds: number;
  status: string;
} = {
  date: "2026-08-01",
  employee_id: "emp-1",
  employee_name: "Alice",
  scheduled_seconds: 28800,
  worked_seconds: 31200,
  overtime_seconds: 2400,
  late_seconds: 0,
  early_departure_seconds: 0,
  status: "present",
};

// ── useAttendanceSummaries ───────────────────────────────────────────────────

describe("useAttendanceSummaries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.fetchAttendanceSummaries as Mock).mockResolvedValue([MOCK_SUMMARY]);
  });

  it("fetches summaries for the given date range and workspace", async () => {
    const { result } = renderHook(
      () => useAttendanceSummaries("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchAttendanceSummaries).toHaveBeenCalledWith(
      WS_ID,
      "2026-08-01",
      "2026-08-03",
      undefined,
    );
    expect(result.current.summaries).toHaveLength(1);
    expect(result.current.summaries[0].employee_name).toBe("Alice");
  });

  it("passes employeeId through to the API when provided", async () => {
    const { result } = renderHook(
      () => useAttendanceSummaries("2026-08-01", "2026-08-03", "emp-42"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchAttendanceSummaries).toHaveBeenCalledWith(
      WS_ID,
      "2026-08-01",
      "2026-08-03",
      "emp-42",
    );
  });


  it("exposes isError=true when the API call fails", async () => {
    (api.fetchAttendanceSummaries as Mock).mockRejectedValue(new Error("Server Error"));

    const { result } = renderHook(
      () => useAttendanceSummaries("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.summaries).toEqual([]);
  });

  it("calls refetch to re-trigger the query", async () => {
    (api.fetchAttendanceSummaries as Mock).mockResolvedValue([MOCK_SUMMARY]);

    const { result } = renderHook(
      () => useAttendanceSummaries("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(api.fetchAttendanceSummaries).toHaveBeenCalledTimes(1);

    act(() => {
      result.current.refetch();
    });

    await waitFor(() => expect(api.fetchAttendanceSummaries).toHaveBeenCalledTimes(2));
  });
});

// ── useAttendanceSummariesInfinite ────────────────────────────────────────────

describe("useAttendanceSummariesInfinite", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getStandaloneOrganizationId.mockReturnValue("standalone");
  });

  it("fetches the first page of paginated summaries and flattens results", async () => {
    (api.fetchAttendanceSummariesPaginated as Mock).mockResolvedValue({
      data: [MOCK_SUMMARY],
      total: 1,
      page: 1,
      limit: 20,
    });

    const { result } = renderHook(
      () => useAttendanceSummariesInfinite("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchAttendanceSummariesPaginated).toHaveBeenCalledWith(
      WS_ID,
      "2026-08-01",
      "2026-08-03",
      { page: 1, limit: 20, employeeId: undefined, otStatus: undefined },
    );
    expect(result.current.summaries).toHaveLength(1);
  });

  it("passes employeeId and otStatus through to the API", async () => {
    (api.fetchAttendanceSummariesPaginated as Mock).mockResolvedValue({
      data: [],
      total: 0,
      page: 1,
      limit: 20,
    });

    renderHook(
      () =>
        useAttendanceSummariesInfinite("2026-08-01", "2026-08-03", {
          employeeId: "emp-5",
          otStatus: "pending",
        }),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(api.fetchAttendanceSummariesPaginated).toHaveBeenCalled());

    expect(api.fetchAttendanceSummariesPaginated).toHaveBeenCalledWith(
      "standalone",
      "2026-08-01",
      "2026-08-03",
      { page: 1, limit: 20, employeeId: "emp-5", otStatus: "pending" },
    );
  });

  it("returns hasNextPage=false when total loaded >= total", async () => {
    (api.fetchAttendanceSummariesPaginated as Mock).mockResolvedValue({
      data: [MOCK_SUMMARY],
      total: 1,
      page: 1,
      limit: 20,
    });

    const { result } = renderHook(
      () => useAttendanceSummariesInfinite("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.hasNextPage).toBe(false);
  });

  it("returns hasNextPage=true when there are more pages", async () => {
    (api.fetchAttendanceSummariesPaginated as Mock).mockResolvedValue({
      data: Array(20).fill(MOCK_SUMMARY),
      total: 50,
      page: 1,
      limit: 20,
    });

    const { result } = renderHook(
      () => useAttendanceSummariesInfinite("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.hasNextPage).toBe(true);
  });

  it("fetches the next page when fetchNextPage is called", async () => {
    (api.fetchAttendanceSummariesPaginated as Mock)
      .mockResolvedValueOnce({
        data: Array(20).fill(MOCK_SUMMARY),
        total: 45,
        page: 1,
        limit: 20,
      })
      .mockResolvedValueOnce({
        data: Array(20).fill({ ...MOCK_SUMMARY, employee_id: "emp-2" }),
        total: 45,
        page: 2,
        limit: 20,
      });

    const { result } = renderHook(
      () => useAttendanceSummariesInfinite("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.fetchNextPage();
    });

    await waitFor(() => {
      expect(result.current.summaries).toHaveLength(40);
    });
    expect(api.fetchAttendanceSummariesPaginated).toHaveBeenCalledTimes(2);
  });

  it("uses standalone organization context", async () => {
    const { result } = renderHook(
      () => useAttendanceSummariesInfinite("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(api.fetchAttendanceSummariesPaginated).toHaveBeenCalledWith(
      "standalone",
      "2026-08-01",
      "2026-08-03",
      { employeeId: undefined, limit: 20, otStatus: undefined, page: 1 },
    );
  });

  it("flattens null data from paginated response gracefully", async () => {
    (api.fetchAttendanceSummariesPaginated as Mock).mockResolvedValue({
      data: null,
      total: 0,
      page: 1,
      limit: 20,
    });

    const { result } = renderHook(
      () => useAttendanceSummariesInfinite("2026-08-01", "2026-08-03"),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.summaries).toEqual([]);
  });
});

// ── useApproveOt ─────────────────────────────────────────────────────────────

describe("useApproveOt", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.approveOt as Mock).mockResolvedValue(undefined);
  });

  it("calls approveOt API with the correct payload (approved)", async () => {
    const { result } = renderHook(() => useApproveOt(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.approveOt({
        employeeId: "emp-1",
        date: "2026-08-01",
        approved_minutes: 120,
        manager_note: "Good work",
        status: "approved",
      });
    });

    expect(api.approveOt).toHaveBeenCalledWith("standalone", "emp-1", {
      date: "2026-08-01",
      approved_minutes: 120,
      manager_note: "Good work",
      status: "approved",
    });
  });

  it("calls approveOt API with rejected status", async () => {
    const { result } = renderHook(() => useApproveOt(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.approveOt({
        employeeId: "emp-2",
        date: "2026-08-02",
        approved_minutes: 0,
        status: "rejected",
      });
    });

    expect(api.approveOt).toHaveBeenCalledWith("standalone", "emp-2", {
      date: "2026-08-02",
      approved_minutes: 0,
      manager_note: undefined,
      status: "rejected",
    });
  });

  it("reports isProcessing=true while the mutation is pending", async () => {
    let resolveApprove: (v: unknown) => void;
    const deferred = new Promise((resolve) => { resolveApprove = resolve; });
    (api.approveOt as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useApproveOt(), { wrapper: createWrapper() });

    act(() => {
      result.current.approveOt({
        employeeId: "emp-1",
        date: "2026-08-01",
        approved_minutes: 60,
        status: "approved",
      });
    });

    await waitFor(() => expect(result.current.isProcessing).toBe(true));

    await act(async () => {
      resolveApprove?.(undefined);
    });
    await waitFor(() => expect(result.current.isProcessing).toBe(false));
  });
});

// ── useEnterPaperOt ──────────────────────────────────────────────────────────

describe("useEnterPaperOt", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.enterPaperOt as Mock).mockResolvedValue(MOCK_SUMMARY);
  });

  it("calls enterPaperOt API with the full payload", async () => {
    const { result } = renderHook(() => useEnterPaperOt(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.enterPaperOt({
        employeeId: "emp-1",
        date: "2026-08-01",
        claimed_minutes: 90,
        reason: "Project deadline",
        paper_form_photo_url: "https://example.com/photo.jpg",
      });
    });

    expect(api.enterPaperOt).toHaveBeenCalledWith("standalone", "emp-1", {
      date: "2026-08-01",
      claimed_minutes: 90,
      reason: "Project deadline",
      paper_form_photo_url: "https://example.com/photo.jpg",
    });
  });

  it("returns the created attendance summary on success", async () => {
    const returnSummary = { ...MOCK_SUMMARY, overtime_seconds: 5400 };
    (api.enterPaperOt as Mock).mockResolvedValue(returnSummary);

    const { result } = renderHook(() => useEnterPaperOt(), { wrapper: createWrapper() });

    let data: unknown;
    await act(async () => {
      data = await result.current.enterPaperOt({
        employeeId: "emp-1",
        date: "2026-08-01",
        claimed_minutes: 90,
        reason: "Test",
      });
    });

    expect(data).toEqual(returnSummary);
  });

  it("reports isSubmitting while the mutation is in flight", async () => {
    let resolveEnter: (v: unknown) => void;
    const deferred = new Promise((resolve) => { resolveEnter = resolve; });
    (api.enterPaperOt as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useEnterPaperOt(), { wrapper: createWrapper() });

    act(() => {
      result.current.enterPaperOt({
        employeeId: "emp-1",
        date: "2026-08-01",
        claimed_minutes: 60,
        reason: "Test",
      });
    });

    await waitFor(() => expect(result.current.isSubmitting).toBe(true));

    await act(async () => {
      resolveEnter?.(MOCK_SUMMARY);
    });
    await waitFor(() => expect(result.current.isSubmitting).toBe(false));
  });
});

// ── useBulkUploadPaperOt ─────────────────────────────────────────────────────

describe("useBulkUploadPaperOt", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.bulkUploadPaperOt as Mock).mockResolvedValue({
      total_rows: 5,
      success_count: 4,
      failed_count: 1,
      errors: [{ line_number: 3, employment_id: "emp-3", error_message: "Invalid date" }],
    });
  });

  it("calls bulkUploadPaperOt API with the File object", async () => {
    const file = new File(["csv,content"], "upload.csv", { type: "text/csv" });
    const { result } = renderHook(() => useBulkUploadPaperOt(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.bulkUploadPaperOt(file);
    });

    expect(api.bulkUploadPaperOt).toHaveBeenCalledWith(WS_ID, file);
  });

  it("returns the bulk result payload on success", async () => {
    const bulkResult = {
      total_rows: 2,
      success_count: 2,
      failed_count: 0,
      errors: [],
    };
    (api.bulkUploadPaperOt as Mock).mockResolvedValue(bulkResult);

    const { result } = renderHook(() => useBulkUploadPaperOt(), { wrapper: createWrapper() });
    const file = new File([], "empty.csv");

    let data: unknown;
    await act(async () => {
      data = await result.current.bulkUploadPaperOt(file);
    });

    expect(data).toEqual(bulkResult);
  });

  it("reports isUploading while the mutation is pending", async () => {
    let resolveBulk: (v: unknown) => void;
    const deferred = new Promise((resolve) => { resolveBulk = resolve; });
    (api.bulkUploadPaperOt as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useBulkUploadPaperOt(), { wrapper: createWrapper() });

    act(() => {
      result.current.bulkUploadPaperOt(new File([], "test.csv"));
    });

    await waitFor(() => expect(result.current.isUploading).toBe(true));

    await act(async () => {
      resolveBulk?.({ total_rows: 0, success_count: 0, failed_count: 0, errors: [] });
    });
    await waitFor(() => expect(result.current.isUploading).toBe(false));
  });
});

// ── useClaimOt ───────────────────────────────────────────────────────────────

describe("useClaimOt", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.claimOt as Mock).mockResolvedValue(undefined);
  });

  it("calls claimOt API with employeeId, date, and minutes", async () => {
    const { result } = renderHook(() => useClaimOt(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.claimOt({
        employeeId: "emp-1",
        date: "2026-08-01",
        claimed_minutes: 60,
        reason: "Overtime work",
      });
    });

    expect(api.claimOt).toHaveBeenCalledWith("standalone", "emp-1", {
      date: "2026-08-01",
      claimed_minutes: 60,
      reason: "Overtime work",
    });
  });

  it("invalidates both attendance and attendance-report keys on success", async () => {
    const { result } = renderHook(() => useClaimOt(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.claimOt({
        employeeId: "emp-1",
        date: "2026-08-01",
        claimed_minutes: 45,
      });
    });

    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: attendanceKeys.all });
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["attendance-report"] });
    });
  });

  it("reports isSubmitting during the mutation lifecycle", async () => {
    let resolveClaim: (v: unknown) => void;
    const deferred = new Promise((resolve) => { resolveClaim = resolve; });
    (api.claimOt as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useClaimOt(), { wrapper: createWrapper() });

    act(() => {
      result.current.claimOt({
        employeeId: "emp-1",
        date: "2026-08-01",
        claimed_minutes: 30,
      });
    });

    await waitFor(() => expect(result.current.isSubmitting).toBe(true));

    await act(async () => {
      resolveClaim?.();
    });
    await waitFor(() => expect(result.current.isSubmitting).toBe(false));
  });
});
