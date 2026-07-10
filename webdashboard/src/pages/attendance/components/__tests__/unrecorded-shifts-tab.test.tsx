import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../../../lib/api";
import type { DailyAttendanceReport } from "../../../../lib/api/types";
import { UnrecordedShiftsTab } from "../unrecorded-shifts-tab";

// Hoisted mock fn so the vi.mock factory (hoisted above imports) can close
// over it. Populated in beforeEach.
const { mockExportNoShowToExcel } = vi.hoisted(() => ({
  mockExportNoShowToExcel: vi.fn(),
}));

// ---------------------------------------------------------------------------
// IntersectionObserver mock
//
// Captures the callback passed to `new IntersectionObserver(cb)` so tests can
// simulate the sentinel scrolling into view — the trigger that fires
// fetchNextPage in the infinite-scroll hook.
// ---------------------------------------------------------------------------
type LooseIntersectionObserverCallback = (
  entries: IntersectionObserverEntry[],
  observer: IntersectionObserver,
) => void;

let intersectionCallback: LooseIntersectionObserverCallback | null = null;
class MockIntersectionObserver {
  constructor(cb: LooseIntersectionObserverCallback) {
    intersectionCallback = cb;
  }
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  static fire(isIntersecting: boolean) {
    intersectionCallback?.([{ isIntersecting } as IntersectionObserverEntry], null as unknown as IntersectionObserver);
  }
}
window.IntersectionObserver =
  MockIntersectionObserver as unknown as typeof IntersectionObserver;

// ---------------------------------------------------------------------------
// Module mocks
// ---------------------------------------------------------------------------
vi.mock("../../../../lib/api", () => ({
  fetchAttendanceReport: vi.fn(),
  fetchEmployees: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

// The Excel module dynamically imports exceljs + file-saver; mock it so the
// component test can assert the export wiring without pulling in those deps.
vi.mock("../../../../lib/excel-no-show", () => ({
  exportNoShowToExcel: (...args: unknown[]) => mockExportNoShowToExcel(...args),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
  }),
}));

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------
function makeRow(
  overrides: Partial<DailyAttendanceReport> = {},
): DailyAttendanceReport {
  return {
    actual_punch_in: null,
    actual_punch_out: null,
    break_duration_minutes: null,
    break_end: null,
    break_start: null,
    computed_ot_hours: 0,
    computed_std_hours: 0,
    date: "2026-07-10",
    employee_id: "emp-1",
    employee_name: "Employee One",
    employment_id: "employment-1",
    is_break_auto_deduct: false,
    is_holiday: false,
    ot_work_units: 0,
    scheduled_end: "17:00:00",
    scheduled_start: "08:00:00",
    shift_name: "Morning Shift",
    status: "absent",
    std_work_units: 0,
    unit_coefficient_hours: 1,
    unit_type: "DAY",
    ...overrides,
  };
}

/** Build `count` employees so the employeeMap can resolve department names. */
function makeEmployees(count: number) {
  return Array.from({ length: count }, (_, i) => ({
    id: `emp-${i + 1}`,
    full_name: `Employee ${i + 1}`,
    employee_id_number: `EMP${String(i + 1).padStart(3, "0")}`,
    department_name: i % 2 === 0 ? "Engineering" : "HR",
  }));
}

describe("UnrecordedShiftsTab", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    intersectionCallback = null;
    mockExportNoShowToExcel.mockReset();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });

    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: makeEmployees(60),
      total: 60,
      page: 1,
      limit: 500,
    } as never);
  });

  const renderTab = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <UnrecordedShiftsTab />
      </QueryClientProvider>,
    );

  // ── Core rendering ──────────────────────────────────────────────────────

  it("renders absent rows enriched with department and employee code", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [
        makeRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Alice" }),
        makeRow({ employment_id: "employment-2", employee_id: "emp-2", employee_name: "Bob" }),
      ],
      total: 2,
      page: 1,
      limit: 30,
    } as never);

    renderTab();

    expect(await screen.findByText("Alice")).toBeInTheDocument();
    expect(screen.getByText("Bob")).toBeInTheDocument();
    // Department resolved from employeeMap (emp-1 → Engineering). The name
    // appears in both the table cell and the department-filter dropdown, so
    // assert on the collection rather than a single element.
    expect(screen.getAllByText("Engineering").length).toBeGreaterThanOrEqual(1);
    // Employee code badge resolved from employeeMap
    expect(screen.getByText("EMP001")).toBeInTheDocument();
  });

  it("renders the Shift Type column with the shift_name value", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [
        makeRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Alice", shift_name: "Morning Shift" }),
        makeRow({ employment_id: "employment-2", employee_id: "emp-2", employee_name: "Bob", shift_name: null }),
      ],
      total: 2,
      page: 1,
      limit: 30,
    } as never);

    renderTab();

    // Header
    expect(await screen.findByText("unrecorded_shifts.columns.shift_type")).toBeInTheDocument();
    // Row with a shift name shows it
    expect(screen.getByText("Morning Shift")).toBeInTheDocument();
    // Row with null shift name renders an em-dash
    const dashes = screen.getAllByText("—");
    expect(dashes.length).toBeGreaterThanOrEqual(1);
  });

  it("filters out rows whose status is not 'absent'", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [
        makeRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Absent Alice", status: "absent" }),
        makeRow({ employment_id: "employment-2", employee_id: "emp-2", employee_name: "Present Bob", status: "present" }),
        makeRow({ employment_id: "employment-3", employee_id: "emp-3", employee_name: "Late Carol", status: "late" }),
        makeRow({ employment_id: "employment-4", employee_id: "emp-4", employee_name: "Off Dave", status: "off" }),
      ],
      total: 4,
      page: 1,
      limit: 30,
    } as never);

    renderTab();

    expect(await screen.findByText("Absent Alice")).toBeInTheDocument();
    expect(screen.queryByText("Present Bob")).not.toBeInTheDocument();
    expect(screen.queryByText("Late Carol")).not.toBeInTheDocument();
    expect(screen.queryByText("Off Dave")).not.toBeInTheDocument();
  });

  // ── Loading / empty states ──────────────────────────────────────────────

  it("shows skeleton rows while the first page is loading", () => {
    // Never-resolving promise keeps the query in the loading state.
    vi.mocked(api.fetchAttendanceReport).mockImplementation(
      () => new Promise(() => { /* pending */ }),
    );

    renderTab();

    expect(document.querySelectorAll(".animate-pulse").length).toBeGreaterThan(0);
  });

  it("shows the empty state when no absent rows exist", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [
        makeRow({ employment_id: "employment-1", employee_id: "emp-1", status: "present" }),
      ],
      total: 1,
      page: 1,
      limit: 30,
    } as never);

    renderTab();

    expect(await screen.findByText("unrecorded_shifts.empty")).toBeInTheDocument();
  });

  it("shows the empty state when the report returns no rows at all", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [],
      total: 0,
      page: 1,
      limit: 30,
    } as never);

    renderTab();

    expect(await screen.findByText("unrecorded_shifts.empty")).toBeInTheDocument();
  });

  // ── Pagination configuration ────────────────────────────────────────────

  it("requests the first page with EMPLOYEE_PAGE_SIZE (30) as the limit", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [makeRow()],
      total: 1,
      page: 1,
      limit: 30,
    } as never);

    renderTab();

    await waitFor(() => {
      expect(api.fetchAttendanceReport).toHaveBeenCalledTimes(1);
    });
    expect(api.fetchAttendanceReport).toHaveBeenCalledWith(
      expect.objectContaining({ page: 1, limit: 30, type: "daily" }),
    );
  });

  // ── Infinite scroll: multi-page fetch ───────────────────────────────────

  it("fetches the next page via the sentinel when scrolled into view", async () => {
    // Page 1 returns 30 absent rows; total is 60 → hasNextPage is true.
    const page1Rows = Array.from({ length: 30 }, (_, i) =>
      makeRow({
        employment_id: `employment-${i + 1}`,
        employee_id: `emp-${i + 1}`,
        employee_name: `Employee ${i + 1}`,
      }),
    );
    const page2Rows = Array.from({ length: 30 }, (_, i) =>
      makeRow({
        employment_id: `employment-${i + 31}`,
        employee_id: `emp-${i + 31}`,
        employee_name: `Employee ${i + 31}`,
      }),
    );

    vi.mocked(api.fetchAttendanceReport).mockImplementation(
      (args) =>
        Promise.resolve({
          data: args.page === 1 ? page1Rows : page2Rows,
          total: 60,
          page: args.page as number,
          limit: 30,
        }) as never,
    );

    renderTab();

    // First page renders.
    expect(await screen.findByText("Employee 1")).toBeInTheDocument();
    expect(api.fetchAttendanceReport).toHaveBeenCalledTimes(1);

    // The sentinel observer must have been created (sentinel <tr ref> mounted).
    await waitFor(() => {
      expect(intersectionCallback).not.toBeNull();
    });

    // Simulate the sentinel scrolling into view → triggers fetchNextPage.
    MockIntersectionObserver.fire(true);

    // Second page is fetched and its rows are appended.
    await waitFor(() => {
      expect(api.fetchAttendanceReport).toHaveBeenCalledTimes(2);
    });
    expect(api.fetchAttendanceReport).toHaveBeenLastCalledWith(
      expect.objectContaining({ page: 2, limit: 30 }),
    );
    expect(await screen.findByText("Employee 31")).toBeInTheDocument();
  });

  it("does not fetch a second page when all rows fit in the first page", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [
        makeRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Only One" }),
      ],
      total: 1,
      page: 1,
      limit: 30,
    } as never);

    renderTab();

    expect(await screen.findByText("Only One")).toBeInTheDocument();

    // Give the observer a chance to fire; it must NOT trigger another fetch.
    await waitFor(() => {
      expect(api.fetchAttendanceReport).toHaveBeenCalledTimes(1);
    });
    expect(api.fetchAttendanceReport).not.toHaveBeenCalledWith(
      expect.objectContaining({ page: 2 }),
    );
  });

  // ── Department filter ───────────────────────────────────────────────────

  it("filters rendered rows by the selected department", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [
        // emp-1 → Engineering (even index), emp-2 → HR (odd index)
        makeRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Eng Worker" }),
        makeRow({ employment_id: "employment-2", employee_id: "emp-2", employee_name: "HR Worker" }),
      ],
      total: 2,
      page: 1,
      limit: 30,
    } as never);

    renderTab();

    expect(await screen.findByText("Eng Worker")).toBeInTheDocument();
    expect(screen.getByText("HR Worker")).toBeInTheDocument();

    // Select "HR" from the department filter dropdown.
    const deptSelect = document.getElementById("unrecorded-dept-filter") as HTMLSelectElement;
    deptSelect.value = "HR";
    deptSelect.dispatchEvent(new Event("change", { bubbles: true }));

    // Re-render: only HR remains.
    await waitFor(() => {
      expect(screen.queryByText("Eng Worker")).not.toBeInTheDocument();
    });
    expect(screen.getByText("HR Worker")).toBeInTheDocument();
  });

  // ── Export ───────────────────────────────────────────────────────────────

  it("exports the filtered rows to Excel when the export button is clicked", async () => {
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [
        makeRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Alice" }),
        makeRow({ employment_id: "employment-2", employee_id: "emp-2", employee_name: "Bob" }),
      ],
      total: 2,
      page: 1,
      limit: 30,
    } as never);

    renderTab();
    expect(await screen.findByText("Alice")).toBeInTheDocument();

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    expect(exportBtn).not.toBeDisabled();
    exportBtn.click();

    await waitFor(() => {
      expect(mockExportNoShowToExcel).toHaveBeenCalledTimes(1);
    });

    const [rows, options] = mockExportNoShowToExcel.mock.calls[0] as [
      Array<{ employee_name: string }>,
      { date: string },
    ];
    expect(rows).toHaveLength(2);
    expect(rows[0].employee_name).toBe("Alice");
    expect(rows[1].employee_name).toBe("Bob");
    // The report date is forwarded so the filename is dated.
    expect(options.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });

  it("disables the export button while loading or when there are no rows", async () => {
    // No absent rows → button disabled even after data loads.
    vi.mocked(api.fetchAttendanceReport).mockResolvedValue({
      data: [makeRow({ status: "present" })],
      total: 1,
      page: 1,
      limit: 30,
    } as never);

    renderTab();
    await waitFor(() => {
      expect(screen.queryByText("Employee One")).not.toBeInTheDocument();
    });

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    expect(exportBtn).toBeDisabled();
  });
});
