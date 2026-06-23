import { useState } from "react";
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

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatDate: (date: any) => date ? String(date) : "---",
    formatTime: (time: any) => time ? String(time).substring(0, 5) : "---",
    formatTime24h: (time: any) => time ? String(time).substring(0, 5) : "",
    formatDateTime: (dt: any) => dt ? String(dt) : "---",
    formatMonthYear: (dt: any) => dt ? String(dt) : "---",
  }),
}));

vi.mock("@/components/common/employee-search", () => ({
  EmployeeSearch: ({
    onChange,
    value,
    onSelectEmployee,
  }: {
    onChange: (val: string[]) => void;
    value?: string | string[];
    onSelectEmployee?: (emp: any) => void;
  }) => {
    return (
      <input
        data-testid="mock-employee-search"
        onChange={(e) => {
          const val = e.target.value;
          const ids = val ? val.split(",") : [];
          if (ids.length > 0 && onSelectEmployee) {
            const lastId = ids[ids.length - 1];
            const localMockEmployees = [
              { id: "emp-1", full_name: "Alice Security", department_name: "Security Guard" },
              { id: "emp-2", full_name: "Bob Patrol", department_name: "Patrol Officer" },
              { id: "emp-3", full_name: "Charlie Freelance" },
            ];
            const existing = localMockEmployees.find((emp) => emp.id === lastId);
            if (existing) {
              onSelectEmployee(existing);
            } else {
              onSelectEmployee({
                id: lastId,
                full_name: `Searched Employee ${lastId.toUpperCase()}`,
              });
            }
          }
          onChange(ids);
        }}
        value={Array.isArray(value) ? value.join(",") : (value ?? "")}
      />
    );
  },
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
    selectedEmployeeIds: [],
    selectedEmployees: [],
    setSelectedEmployeeIds: vi.fn(),
    setSelectedEmployees: vi.fn(),
    startDate: "",
    targetDates: [],
    updateShiftMutation: {} as unknown as ReturnType<typeof useShifts>["updateShiftMutation"],
    viewMode: "month",
    weekDays: [],
    wsId: "test-ws-id",
    ...overrides,
  } as unknown as ReturnType<typeof useShifts>;
};

const StatefulGridWrapper = ({ overrides = {} }: { overrides?: any }) => {
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [selectedEmployees, setSelectedEmployees] = useState<any[]>([]);
  const hook = createMockHook({
    selectedEmployeeIds,
    selectedEmployees,
    setSelectedEmployeeIds,
    setSelectedEmployees,
    ...overrides,
  });
  return <MonthlyRosterGrid hook={hook} />;
};

describe("MonthlyRosterGrid Filtering", () => {
  it("renders empty state by default when no employees are selected", () => {
    const hook = createMockHook();
    render(<MonthlyRosterGrid hook={hook} />);

    expect(screen.getByText("shifts.empty_state.title")).toBeInTheDocument();
    expect(screen.getByText("shifts.empty_state.desc")).toBeInTheDocument();
    expect(screen.queryByText("Alice Security")).not.toBeInTheDocument();
  });

  it("renders grid and employee when selected", async () => {
    const user = userEvent.setup();
    render(<StatefulGridWrapper />);

    // Check empty state initially
    expect(screen.getByText("shifts.empty_state.title")).toBeInTheDocument();

    // Select employee "emp-1" (Alice Security)
    const searchInput = screen.getByTestId("mock-employee-search");
    await user.type(searchInput, "emp-1");

    // Grid and checklist should show Alice Security
    expect(screen.getAllByText("Alice Security").length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByText("Bob Patrol")).not.toBeInTheDocument();
    expect(screen.queryByText("shifts.empty_state.title")).not.toBeInTheDocument();
  });

  it("can deselect employee by unchecking their name in the checklist", async () => {
    const user = userEvent.setup();
    render(<StatefulGridWrapper />);

    // Select employee "emp-1" (Alice Security)
    const searchInput = screen.getByTestId("mock-employee-search");
    await user.type(searchInput, "emp-1");

    // Grid and checklist should show Alice Security
    expect(screen.getAllByText("Alice Security").length).toBeGreaterThanOrEqual(1);

    // Click checkbox to deselect
    const checkbox = screen.getByRole("checkbox", { name: "Alice Security" });
    await user.click(checkbox);

    // Verify empty state is shown again
    expect(screen.getByText("shifts.empty_state.title")).toBeInTheDocument();
  });

  it("displays loading state when isMonthLoading or isEmployeesLoading is true", () => {
    const hook1 = createMockHook({ isMonthLoading: true });
    const { container: container1 } = render(<MonthlyRosterGrid hook={hook1} />);
    expect(container1.querySelectorAll('.animate-pulse').length).toBeGreaterThan(0);

    const hook2 = createMockHook({ isEmployeesLoading: true });
    const { container: container2 } = render(<MonthlyRosterGrid hook={hook2} />);
    expect(container2.querySelectorAll('.animate-pulse').length).toBeGreaterThan(0);
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

  it("successfully syncs and renders selected employees missing from preloaded list", async () => {
    const user = userEvent.setup();
    render(<StatefulGridWrapper />);

    // Initially shows empty state
    expect(screen.getByText("shifts.empty_state.title")).toBeInTheDocument();

    // Select employee "emp-new" (who is not in hook.employees)
    const searchInput = screen.getByTestId("mock-employee-search");
    await user.type(searchInput, "emp-new");

    // The grid should now show "Searched Employee EMP-NEW"
    expect(screen.getAllByText("Searched Employee EMP-NEW").length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByText("shifts.empty_state.title")).not.toBeInTheDocument();

    // The checklist checkbox for the employee should be checked
    const checkbox = screen.getByRole("checkbox", { name: "Searched Employee EMP-NEW" });
    expect(checkbox).toBeChecked();

    // Deselecting should remove them and revert to empty state
    await user.click(checkbox);
    expect(screen.getByText("shifts.empty_state.title")).toBeInTheDocument();
  });
});
