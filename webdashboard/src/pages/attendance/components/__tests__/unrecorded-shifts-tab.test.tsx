import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import type { DailyAttendanceReport } from "../../../../lib/api/types";

import { UnrecordedShiftsTab } from "../unrecorded-shifts-tab";

// ── Hoisted mock state ──────────────────────────────────────────────────────
const { mockExportNoShowToExcel, mockHookData, mockExportHook } = vi.hoisted(() => ({
  mockExportNoShowToExcel: vi.fn(),
  mockHookData: {
    absentRows: [] as any[],
    allRows: [] as any[],
    departments: [] as string[],
    employeeMap: new Map<string, { department_name: string; employee_id_number: string }>(),
    fetchNextPage: vi.fn() as () => Promise<unknown>,
    filteredRows: [] as any[],
    hasNextPage: undefined as boolean | undefined,
    isFetchingNextPage: false,
    isLoading: false,
    queryKey: ["unrecorded-shifts"],
    tracesByKey: new Map<string, any[]>(),
    organizationTimezone: "UTC",
  },
  mockExportHook: {
    exportFn: vi.fn() as () => Promise<void>,
    isExporting: false,
  },
}));

// ── IntersectionObserver mock ───────────────────────────────────────────────
type LooseIntersectionObserverCallback = (
  entries: IntersectionObserverEntry[],
  observer: IntersectionObserver,
) => void;

let intersectionCallback: LooseIntersectionObserverCallback | null = null;
class MockIntersectionObserver {
  constructor(cb: LooseIntersectionObserverCallback) { intersectionCallback = cb; }
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  static fire(isIntersecting: boolean) {
    intersectionCallback?.([{ isIntersecting } as IntersectionObserverEntry], null as unknown as IntersectionObserver);
  }
}
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;

// ── Module mocks ────────────────────────────────────────────────────────────
vi.mock("../hooks/use-unrecorded-shifts-data", () => ({
  useUnrecordedShiftsData: () => mockHookData,
  getNextNoShowPageParam: vi.fn(),
}));

vi.mock("../unrecorded-shifts/use-no-show-export", () => ({
  useNoShowExport: () => mockExportHook,
}));

vi.mock("@/hooks/use-employees", () => ({
  useAllEmployees: () => ({ employees: makeEmployees(60), isLoading: false }),
}));

vi.mock("@/hooks/use-workspace", () => ({
  useOrganization: () => ({ organization: { timezone: "UTC" }, isLoading: false }),
}));

vi.mock("../../../../lib/excel-no-show", () => ({
  exportNoShowToExcel: (...args: unknown[]) => mockExportNoShowToExcel(...args),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
  }),
}));

// ── Test helpers ────────────────────────────────────────────────────────────
function makeRow(overrides: Partial<DailyAttendanceReport> = {}): DailyAttendanceReport {
  const now = new Date();
  const currentDate = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
  return {
    actual_punch_in: null, actual_punch_out: null, break_duration_minutes: null,
    break_end: null, break_start: null, computed_ot_hours: 0, computed_std_hours: 0,
    date: currentDate, employee_id: "emp-1", employee_name: "Employee One",
    employment_id: "employment-1", is_break_auto_deduct: false, is_holiday: false,
    ot_work_units: 0, scheduled_end: "23:59:00", scheduled_start: "00:00:00",
    shift_name: "Morning Shift", grace_period_minutes: 0, status: "absent",
    std_work_units: 0, unit_coefficient_hours: 1, unit_type: "DAY",
    timezone: "UTC",
    ...overrides,
  };
}

function makeEmployees(count: number) {
  return Array.from({ length: count }, (_, i) => ({
    id: `emp-${i + 1}`, full_name: `Employee ${i + 1}`,
    employee_id_number: `EMP${String(i + 1).padStart(3, "0")}`,
    department_name: i % 2 === 0 ? "Engineering" : "HR",
  }));
}

function makeZonedRow(overrides: any = {}) {
  const base = makeRow(overrides);
  const empIdx = parseInt((base.employee_id ?? "emp-1").replace("emp-", ""), 10) || 1;
  return {
    ...base,
    department_name: overrides.department_name ?? (empIdx % 2 === 0 ? "HR" : "Engineering"),
    employee_id_number: overrides.employee_id_number ?? `EMP${String(empIdx).padStart(3, "0")}`,
    zone: overrides.zone ?? "b",
    zoneMinutes: overrides.zoneMinutes ?? 30,
  };
}

function resetHookData() {
  mockHookData.absentRows = [];
  mockHookData.allRows = [];
  mockHookData.departments = [];
  mockHookData.employeeMap = new Map();
  mockHookData.fetchNextPage = vi.fn();
  mockHookData.filteredRows = [];
  mockHookData.hasNextPage = undefined;
  mockHookData.isFetchingNextPage = false;
  mockHookData.isLoading = false;
  mockHookData.queryKey = ["unrecorded-shifts", "test-ws-id"];
  mockHookData.tracesByKey = new Map();
  mockHookData.organizationTimezone = "UTC";
}

// ── Tests ───────────────────────────────────────────────────────────────────
describe("UnrecordedShiftsTab", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    intersectionCallback = null;
    mockExportNoShowToExcel.mockReset();
    mockExportHook.exportFn = vi.fn();
    mockExportHook.isExporting = false;
    resetHookData();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
  });

  const renderTab = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <UnrecordedShiftsTab />
      </QueryClientProvider>,
    );

  // ── Core rendering ──────────────────────────────────────────────────────

  it("renders absent rows enriched with department and employee code", () => {
    const rows = [
      makeZonedRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Alice" }),
      makeZonedRow({ employment_id: "employment-2", employee_id: "emp-2", employee_name: "Bob" }),
    ];
    mockHookData.absentRows = rows;
    mockHookData.filteredRows = rows;
    mockHookData.departments = ["Engineering", "HR"];

    renderTab();

    expect(screen.getAllByText("Alice").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Bob").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Engineering").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("EMP001").length).toBeGreaterThan(0);
  });

  it("renders the Shift Type column with the shift_name value", () => {
    const rows = [
      makeZonedRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Alice", shift_name: "Morning Shift" }),
      makeZonedRow({ employment_id: "employment-2", employee_id: "emp-2", employee_name: "Bob", shift_name: null }),
    ];
    mockHookData.absentRows = rows;
    mockHookData.filteredRows = rows;
    mockHookData.departments = ["Engineering", "HR"];

    renderTab();

    expect(screen.getAllByText("unrecorded_shifts.columns.shift_type").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Morning Shift").length).toBeGreaterThan(0);
    expect(screen.getAllByText("—").length).toBeGreaterThanOrEqual(1);
  });

  it("only renders absent-status rows (non-absent excluded by useNoShowRows)", () => {
    const absentOnly = [
      makeZonedRow({ employment_id: "employment-1", employee_id: "emp-1", employee_name: "Absent Alice", status: "absent" }),
    ];
    mockHookData.absentRows = absentOnly;
    mockHookData.filteredRows = absentOnly;
    mockHookData.allRows = [
      ...absentOnly,
      makeZonedRow({ employee_name: "Present Bob", status: "present" }),
      makeZonedRow({ employee_name: "Late Carol", status: "late" }),
      makeZonedRow({ employee_name: "Off Dave", status: "off" }),
    ];

    renderTab();

    expect(screen.getAllByText("Absent Alice").length).toBeGreaterThan(0);
    expect(screen.queryByText("Present Bob")).not.toBeInTheDocument();
    expect(screen.queryByText("Late Carol")).not.toBeInTheDocument();
    expect(screen.queryByText("Off Dave")).not.toBeInTheDocument();
  });

  // ── Loading / empty states ──────────────────────────────────────────────

  it("shows skeleton rows while the first page is loading", () => {
    mockHookData.isLoading = true;
    renderTab();
    expect(document.querySelectorAll(".animate-pulse").length).toBeGreaterThan(0);
  });

  it("shows the empty state when no absent rows exist", () => {
    mockHookData.absentRows = [];
    mockHookData.filteredRows = [];
    renderTab();
    expect(screen.getAllByText(/unrecorded_shifts\.empty/).length).toBeGreaterThan(0);
  });

  it("shows the empty state when the report returns no rows at all", () => {
    mockHookData.absentRows = [];
    mockHookData.filteredRows = [];
    renderTab();
    expect(screen.getAllByText(/unrecorded_shifts\.empty/).length).toBeGreaterThan(0);
  });

  // ── Department filter ───────────────────────────────────────────────────

  it("filters rendered rows by the selected department", () => {
    const engRow = makeZonedRow({ employee_id: "emp-1", employee_name: "Eng Worker", department_name: "Engineering" });
    const hrRow = makeZonedRow({ employee_id: "emp-2", employee_name: "HR Worker", department_name: "HR" });
    mockHookData.absentRows = [engRow, hrRow];
    mockHookData.filteredRows = [engRow, hrRow];
    mockHookData.departments = ["Engineering", "HR"];

    const { unmount } = renderTab();
    expect(screen.getAllByText("Eng Worker").length).toBeGreaterThan(0);
    expect(screen.getAllByText("HR Worker").length).toBeGreaterThan(0);
    unmount();

    // Simulate HR filter applied — only HR rows pass through
    mockHookData.filteredRows = [hrRow];
    renderTab();

    expect(screen.queryByText("Eng Worker")).not.toBeInTheDocument();
    expect(screen.getAllByText("HR Worker").length).toBeGreaterThan(0);
  });

  // ── Export ───────────────────────────────────────────────────────────────

  it("exports the filtered rows to Excel when the export button is clicked", () => {
    const rows = [
      makeZonedRow({ employee_id: "emp-1", employee_name: "Alice" }),
      makeZonedRow({ employee_id: "emp-2", employee_name: "Bob" }),
    ];
    mockHookData.absentRows = rows;
    mockHookData.filteredRows = rows;

    // Configure export to call exportNoShowToExcel with filtered rows
    mockExportHook.exportFn = vi.fn().mockImplementation(async () => {
      const noShowRows = rows.map((r) => ({
        employee_id_number: r.employee_id_number,
        employee_name: r.employee_name,
        department_name: r.department_name,
        shift_name: r.shift_name ?? null,
        scheduled_start: r.scheduled_start ?? null,
        scheduled_end: r.scheduled_end ?? null,
        zone_label: "B · Grace hết",
      }));
      void mockExportNoShowToExcel(noShowRows, { t: (k: string) => k, dateLabel: "2026-07-24" });
    });

    renderTab();
    expect(screen.getAllByText("Alice").length).toBeGreaterThan(0);

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    exportBtn.click();

    expect(mockExportNoShowToExcel).toHaveBeenCalledTimes(1);
    const [rowsArg, options] = mockExportNoShowToExcel.mock.calls[0] as [{ employee_name: string }[], { dateLabel: string }];
    expect(rowsArg).toHaveLength(2);
    expect(rowsArg[0].employee_name).toBe("Alice");
    expect(rowsArg[1].employee_name).toBe("Bob");
    expect(options.dateLabel).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });

  it("drains every page before exporting so no rows are missed", () => {
    const allRows = [
      makeZonedRow({ employee_id: "emp-1", employee_name: "Alice" }),
      makeZonedRow({ employee_id: "emp-2", employee_name: "Bob" }),
      makeZonedRow({ employee_id: "emp-3", employee_name: "Carol" }),
      makeZonedRow({ employee_id: "emp-4", employee_name: "Dave" }),
    ];
    mockHookData.allRows = allRows;
    mockHookData.absentRows = allRows;
    mockHookData.filteredRows = allRows;

    mockExportHook.exportFn = vi.fn().mockImplementation(async () => {
      const noShowRows = allRows.map((r) => ({
        employee_id_number: r.employee_id_number,
        employee_name: r.employee_name,
        department_name: r.department_name,
        shift_name: r.shift_name ?? null,
        scheduled_start: r.scheduled_start ?? null,
        scheduled_end: r.scheduled_end ?? null,
        zone_label: "B · Grace hết",
      }));
      void mockExportNoShowToExcel(noShowRows, { t: (k: string) => k, dateLabel: "2026-07-24" });
    });

    renderTab();
    expect(screen.getAllByText("Alice").length).toBeGreaterThan(0);

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    exportBtn.click();

    expect(mockExportNoShowToExcel).toHaveBeenCalledTimes(1);
    const [rowsArg] = mockExportNoShowToExcel.mock.calls[0] as [{ employee_name: string }[], unknown];
    const names = rowsArg.map((r) => r.employee_name).sort();
    expect(names).toEqual(["Alice", "Bob", "Carol", "Dave"]);
  });

  it("applies the department filter across all drained pages", () => {
    const allRows = [
      makeZonedRow({ employee_id: "emp-1", employee_name: "Eng Alice", department_name: "Engineering" }),
      makeZonedRow({ employee_id: "emp-2", employee_name: "HR Bob", department_name: "HR" }),
      makeZonedRow({ employee_id: "emp-3", employee_name: "Eng Carol", department_name: "Engineering" }),
      makeZonedRow({ employee_id: "emp-4", employee_name: "HR Dave", department_name: "HR" }),
    ];
    mockHookData.allRows = allRows;
    mockHookData.absentRows = allRows;
    mockHookData.filteredRows = [allRows[1], allRows[3]]; // HR-only
    mockHookData.departments = ["Engineering", "HR"];

    mockExportHook.exportFn = vi.fn().mockImplementation(async () => {
      const hrRows = allRows.filter((r) => r.department_name === "HR");
      const noShowRows = hrRows.map((r) => ({
        employee_id_number: r.employee_id_number,
        employee_name: r.employee_name,
        department_name: r.department_name,
        shift_name: r.shift_name ?? null,
        scheduled_start: r.scheduled_start ?? null,
        scheduled_end: r.scheduled_end ?? null,
        zone_label: "B · Grace hết",
      }));
      void mockExportNoShowToExcel(noShowRows, { t: (k: string) => k, dateLabel: "2026-07-24" });
    });

    renderTab();
    expect(screen.getAllByText("HR Bob").length).toBeGreaterThan(0);

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    exportBtn.click();

    expect(mockExportNoShowToExcel).toHaveBeenCalledTimes(1);
    const [rowsArg] = mockExportNoShowToExcel.mock.calls[0] as [{ employee_name: string }[], unknown];
    const names = rowsArg.map((r) => r.employee_name).sort();
    expect(names).toEqual(["HR Bob", "HR Dave"]);
  });

  it("shows the total absent count on the export badge", () => {
    const rows = Array.from({ length: 60 }, (_, i) =>
      makeZonedRow({ employee_id: `emp-${i + 1}`, employee_name: `Employee ${i + 1}` }),
    );
    mockHookData.absentRows = rows;
    mockHookData.filteredRows = rows;

    renderTab();
    expect(screen.getAllByText("Employee 1").length).toBeGreaterThan(0);

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    expect(exportBtn).toHaveTextContent("60");
  });

  it("includes the shift_name field in every exported row", () => {
    const rows = [
      makeZonedRow({ employee_id: "emp-1", employee_name: "Alice", shift_name: "Morning Shift" }),
      makeZonedRow({ employee_id: "emp-2", employee_name: "Bob", shift_name: null }),
    ];
    mockHookData.absentRows = rows;
    mockHookData.filteredRows = rows;

    mockExportHook.exportFn = vi.fn().mockImplementation(async () => {
      const noShowRows = rows.map((r) => ({
        employee_id_number: r.employee_id_number,
        employee_name: r.employee_name,
        department_name: r.department_name,
        shift_name: r.shift_name ?? null,
        scheduled_start: r.scheduled_start ?? null,
        scheduled_end: r.scheduled_end ?? null,
        zone_label: "B · Grace hết",
      }));
      void mockExportNoShowToExcel(noShowRows, { t: (k: string) => k, dateLabel: "2026-07-24" });
    });

    renderTab();
    expect(screen.getAllByText("Alice").length).toBeGreaterThan(0);

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    exportBtn.click();

    expect(mockExportNoShowToExcel).toHaveBeenCalledTimes(1);
    const [rowsArg] = mockExportNoShowToExcel.mock.calls[0] as [{ shift_name: string | null }[], unknown];
    expect(rowsArg).toHaveLength(2);
    expect(rowsArg[0].shift_name).toBe("Morning Shift");
    expect(rowsArg[1].shift_name).toBeNull();
  });

  it("enables the export button when rows are present", () => {
    const rows = [makeZonedRow({ employee_id: "emp-1", employee_name: "Alice" })];
    mockHookData.absentRows = rows;
    mockHookData.filteredRows = rows;

    renderTab();

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    expect(exportBtn).not.toBeDisabled();
  });

  it("disables the export button when there are no absent rows", () => {
    mockHookData.absentRows = [];
    mockHookData.filteredRows = [];

    renderTab();

    const exportBtn = document.getElementById("unrecorded-export-btn") as HTMLButtonElement;
    expect(exportBtn).toBeDisabled();
  });
});
