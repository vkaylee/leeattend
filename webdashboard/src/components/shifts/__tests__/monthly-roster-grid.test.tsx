import { render, screen } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";

import { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { MonthlyRosterGrid } from "../monthly-roster-grid";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, any>) => {
      if (params?.count !== undefined) {
        return `${key}_${params.count}`;
      }
      return key;
    },
    i18n: { language: "en" },
  }),
}));

const mockEmployees = [
  { id: "emp-1", full_name: "Alice Security", department_name: "Security Guard" },
  { id: "emp-2", full_name: "Bob Patrol", department_name: "Patrol Officer" },
  { id: "emp-3", full_name: "Charlie Freelance" },
];

const mockDepartments = [
  { id: "dept-1", name: "Security Guard" },
  { id: "dept-2", name: "Patrol Officer" },
];

const mockMonthDays = [
  new Date("2026-05-01T00:00:00"),
  new Date("2026-05-02T00:00:00"),
  new Date("2026-05-03T00:00:00"),
];

const mockMonthSchedules = [
  {
    employee_id: "emp-1",
    full_name: "Alice Security",
    schedules: [
      {
        date: "2026-05-01",
        shift_id: "shift-1",
        shift_name: "Morning Shift",
        start_time: "08:00:00",
        end_time: "16:00:00",
        source: "pattern",
        traces: [],
      },
    ],
  },
  {
    employee_id: "emp-2",
    full_name: "Bob Patrol",
    schedules: [
      {
        date: "2026-05-01",
        shift_id: "shift-2",
        shift_name: "Night Shift",
        start_time: "22:00:00",
        end_time: "06:00:00",
        source: "roster",
        traces: [],
      },
    ],
  },
  {
    employee_id: "emp-3",
    full_name: "Charlie Freelance",
    schedules: [
      {
        date: "2026-05-01",
        shift_id: null,
        shift_name: null,
        start_time: null,
        end_time: null,
        source: "none",
        traces: [],
      },
    ],
  },
];

const createMockHook = (overrides = {}) => {
  return {
    monthDays: mockMonthDays,
    currentMonth: new Date("2026-05-01T00:00:00"),
    handleNavigateMonth: vi.fn(),
    monthSchedules: mockMonthSchedules,
    isMonthLoading: false,
    shifts: [
      { id: "shift-1", name: "Morning Shift", start_time: "08:00", end_time: "16:00" },
      { id: "shift-2", name: "Night Shift", start_time: "22:00", end_time: "06:00" },
    ],
    handleMonthBulkAssign: vi.fn(),
    handleMonthDeleteRoster: vi.fn(),
    handleCopyPreviousMonth: vi.fn(),
    employees: mockEmployees,
    isMonthError: false,
    monthError: null,
    refetchMonthSchedules: vi.fn().mockResolvedValue(undefined),
    isEmployeesLoading: false,
    isEmployeesError: false,
    employeesError: null,
    refetchEmployees: vi.fn().mockResolvedValue(undefined),
    departments: mockDepartments,
    activeTab: "weekly-roster",
    assignPatternMutation: {} as unknown as ReturnType<typeof useShifts>["assignPatternMutation"],
    bulkRosterMutation: {} as unknown as ReturnType<typeof useShifts>["bulkRosterMutation"],
    correctionEffectiveDate: "",
    createShiftMutation: {} as unknown as ReturnType<typeof useShifts>["createShiftMutation"],
    currentDate: new Date(),
    deleteRosterMutation: {} as unknown as ReturnType<typeof useShifts>["deleteRosterMutation"],
    editingShift: null,
    endDate: "",
    employeeSchedule: undefined,
    handleAssign: vi.fn(),
    handleAssignAllWeek: vi.fn(),
    handleAssignBulk: vi.fn(),
    handleCopyPreviousWeek: vi.fn(),
    handleCreateShift: vi.fn(),
    handleNavigateWeek: vi.fn(),
    handleUpdateShift: vi.fn(),
    history: [],
    isAutoDeduct: false,
    isCorrection: false,
    isCreateModalOpen: false,
    isFixedBreak: false,
    isForever: true,
    isShiftPickerOpen: false,
    newBreakEnd: "",
    newBreakStart: "",
    newShiftBreak: 0,
    newShiftEnd: "",
    newShiftGrace: 0,
    newShiftName: "",
    newShiftStart: "",
    newShiftSymbol: "",
    openEditModal: vi.fn(),
    patterns: [],
    selectedDays: [],
    selectedEmployee: null,
    selectedPatternId: "",
    setActiveTab: vi.fn(),
    setCorrectionEffectiveDate: vi.fn(),
    setCurrentDate: vi.fn(),
    setEditingShift: vi.fn(),
    setEndDate: vi.fn(),
    setIsAutoDeduct: vi.fn(),
    setIsCorrection: vi.fn(),
    setIsCreateModalOpen: vi.fn(),
    setIsFixedBreak: vi.fn(),
    setIsForever: vi.fn(),
    setIsShiftPickerOpen: vi.fn(),
    setNewBreakEnd: vi.fn(),
    setNewBreakStart: vi.fn(),
    setNewShiftBreak: vi.fn(),
    setNewShiftEnd: vi.fn(),
    setNewShiftGrace: vi.fn(),
    setNewShiftName: vi.fn(),
    setNewShiftStart: vi.fn(),
    setNewShiftSymbol: vi.fn(),
    setSelectedDays: vi.fn(),
    setSelectedEmployee: vi.fn(),
    setSelectedPatternId: vi.fn(),
    setStartDate: vi.fn(),
    setTargetDates: vi.fn(),
    setViewMode: vi.fn(),
    startDate: "",
    targetDates: [],
    updateShiftMutation: {} as unknown as ReturnType<typeof useShifts>["updateShiftMutation"],
    viewMode: "month",
    weekDays: [],
    wsId: "test-ws-id",
    ...overrides,
  } as unknown as ReturnType<typeof useShifts>;
};

describe("MonthlyRosterGrid Filtering", () => {
  it("renders all employees by default", () => {
    const hook = createMockHook();
    render(<MonthlyRosterGrid hook={hook} />);

    expect(screen.getByText("Alice Security")).toBeInTheDocument();
    expect(screen.getByText("Bob Patrol")).toBeInTheDocument();
    expect(screen.getByText("Charlie Freelance")).toBeInTheDocument();
  });

  it("filters employees based on search input", async () => {
    const user = userEvent.setup();
    const hook = createMockHook();
    render(<MonthlyRosterGrid hook={hook} />);

    const searchInput = screen.getByPlaceholderText("shifts.monthly.search_placeholder");
    await user.type(searchInput, "Alice");

    expect(screen.getByText("Alice Security")).toBeInTheDocument();
    expect(screen.queryByText("Bob Patrol")).not.toBeInTheDocument();
    expect(screen.queryByText("Charlie Freelance")).not.toBeInTheDocument();
  });

  it("filters employees based on department selection", async () => {
    const user = userEvent.setup();
    const hook = createMockHook();
    render(<MonthlyRosterGrid hook={hook} />);

    // Click on department filter button to open dropdown
    const deptFilterButton = screen.getByRole("button", { name: /shifts\.monthly\.all_departments/i });
    await user.click(deptFilterButton);

    // Verify departments are in dropdown
    const deptOption = screen.getByRole("menuitemradio", { name: "Security Guard" });
    await user.click(deptOption);

    // Grid should only show Alice (Security Guard)
    expect(screen.getByText("Alice Security")).toBeInTheDocument();
    expect(screen.queryByText("Bob Patrol")).not.toBeInTheDocument();
    expect(screen.queryByText("Charlie Freelance")).not.toBeInTheDocument();
  });

  it("filters employees with No Department selected", async () => {
    const user = userEvent.setup();
    const hook = createMockHook();
    render(<MonthlyRosterGrid hook={hook} />);

    // Click on department filter button to open dropdown
    const deptFilterButton = screen.getByRole("button", { name: /shifts\.monthly\.all_departments/i });
    await user.click(deptFilterButton);

    // Select 'No Department'
    const noDeptOption = screen.getByRole("menuitemradio", { name: "shifts.monthly.no_department" });
    await user.click(noDeptOption);

    // Grid should only show Charlie Freelance (no department)
    expect(screen.getByText("Charlie Freelance")).toBeInTheDocument();
    expect(screen.queryByText("Alice Security")).not.toBeInTheDocument();
    expect(screen.queryByText("Bob Patrol")).not.toBeInTheDocument();
  });

  it("displays search empty state when no results match", async () => {
    const user = userEvent.setup();
    const hook = createMockHook();
    render(<MonthlyRosterGrid hook={hook} />);

    const searchInput = screen.getByPlaceholderText("shifts.monthly.search_placeholder");
    await user.type(searchInput, "xyz");

    // Verify that empty state is shown
    expect(screen.getByText("common.combobox.no_results")).toBeInTheDocument();
    expect(screen.getByText("shifts.monthly.no_results_desc")).toBeInTheDocument();

    // Verify grid contents are not shown
    expect(screen.queryByText("Alice Security")).not.toBeInTheDocument();
  });

  it("displays loading state when isMonthLoading or isEmployeesLoading is true", () => {
    const hook1 = createMockHook({ isMonthLoading: true });
    const { rerender } = render(<MonthlyRosterGrid hook={hook1} />);
    expect(screen.getByText("common.loading...")).toBeInTheDocument();

    const hook2 = createMockHook({ isEmployeesLoading: true });
    rerender(<MonthlyRosterGrid hook={hook2} />);
    expect(screen.getByText("common.loading...")).toBeInTheDocument();
  });

  it("displays error state with retry button when queries fail", async () => {
    const user = userEvent.setup();
    const hook = createMockHook({ isMonthError: true });
    render(<MonthlyRosterGrid hook={hook} />);

    expect(screen.getByText("common.error")).toBeInTheDocument();
    expect(screen.getByText("shifts.monthly.failed_to_load")).toBeInTheDocument();

    const retryButton = screen.getByRole("button", { name: "common.retry" });
    await user.click(retryButton);

    expect(hook.refetchMonthSchedules).toHaveBeenCalled();
    expect(hook.refetchEmployees).toHaveBeenCalled();
  });
});
