import { render, screen } from "@testing-library/react";
import { useState } from "react";

import userEvent from "@testing-library/user-event";
import { enUS } from "date-fns/locale";
import { describe, it, expect, vi } from "vitest";

import { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { MonthlyRosterGrid } from "../monthly-roster-grid";
import { RosterToolbar } from "../roster-toolbar";
import { RosterMobileList } from "../roster/roster-mobile-list";

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
    isCorrection: false,
    isCreateModalOpen: false,
    breakCreditMode: "scheduled",
      breakPunchMode: "none",
      isAutoDeduct: true,
    isFixedBreak: false,
    isForever: true,
    isShiftPickerOpen: false,
    newBreakEnd: "",
    newBreakStart: "",
    newShiftBreak: 0,
    newShiftEnd: "",
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
    weekDays: [],// Compliance modal state
    complianceModalOpen: false,
    complianceViolations: [],
    isComplianceResolving: false,
    onComplianceOverride: vi.fn(),
    setComplianceModalOpen: vi.fn(),
    ...overrides,
  } as unknown as ReturnType<typeof useShifts>;
};

const createMockMonthlyRoster = (overrides = {}) => {
  return {
    selectedCells: new Set<string>(),
    setSelectedCells: vi.fn(),
    isShiftPickerOpen: false,
    setIsShiftPickerOpen: vi.fn(),
    isCopying: false,
    isMutating: false,
    selectedEmployeeIds: [],
    setSelectedEmployeeIds: vi.fn(),
    setSelectedEmployees: vi.fn(),
    selectedEmployeesMeta: [],
    filteredSchedules: mockMonthSchedules,
    isDragging: false,
    handleCellClick: vi.fn(),
    handleCellPointerDown: vi.fn(),
    handleCellPointerEnter: vi.fn(),
    handleApplyShift: vi.fn(),
    handleClearSelectedOverrides: vi.fn(),
    handleCopyMonth: vi.fn(),
    ...overrides,
  };
};

const StatefulGridWrapper = ({ overrides = {} }: { overrides?: any }) => {
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [selectedEmployees, setSelectedEmployees] = useState<any[]>([]);
  const hook = createMockHook({
    selectedEmployeeIds,
    selectedEmployees,
    setSelectedEmployeeIds: (ids: any) => {
      setSelectedEmployeeIds(ids);
      const meta = mockEmployees.filter((e) => ids.includes(e.id));
      setSelectedEmployees(meta);
    },
    setSelectedEmployees,
    ...overrides,
  });
  const monthlyRoster = createMockMonthlyRoster({
    selectedEmployeeIds,
    selectedEmployeesMeta: selectedEmployees,
    setSelectedEmployeeIds: (ids: any) => {
      setSelectedEmployeeIds(ids);
      const meta = mockEmployees.filter((e) => ids.includes(e.id));
      setSelectedEmployees(meta);
    },
    setSelectedEmployees,
    filteredSchedules: mockMonthSchedules.filter((s) => selectedEmployeeIds.includes(s.employee_id)),
  });
  return (
    <div className="space-y-4">
      <RosterToolbar
        hook={hook}
        isCopying={monthlyRoster.isCopying}
        isMutating={monthlyRoster.isMutating}
        onApplyShift={monthlyRoster.handleApplyShift}
        onAssignSelected={() => monthlyRoster.setIsShiftPickerOpen(true)}
        onCancelSelection={() => monthlyRoster.setSelectedCells(new Set())}
        onClearOverrides={monthlyRoster.handleClearSelectedOverrides}
        selectedCells={monthlyRoster.selectedCells}
        selectedEmployeesMeta={monthlyRoster.selectedEmployeesMeta}
        viewMode="month"
      />
      <MonthlyRosterGrid hook={hook} monthlyRoster={monthlyRoster as any} />
    </div>
  );
};

describe("RosterMobileList", () => {
  it("renders one employee card with every month day and preserves cell activation", async () => {
    const user = userEvent.setup();
    const onCellClick = vi.fn();
    const onCellKeyDown = vi.fn();

    render(
      <RosterMobileList
        currentLocale={enUS}
        monthDays={mockMonthDays}
        monthSchedules={mockMonthSchedules as any}
        onCellClick={onCellClick}
        onCellKeyDown={onCellKeyDown}
        selectedCells={new Set(["emp-1:2026-05-01"])}
        shifts={[]}
      />,
    );

    expect(screen.getByTestId("monthly-roster-mobile")).toBeInTheDocument();
    expect(screen.getAllByTestId(/monthly-roster-mobile-employee-/)).toHaveLength(3);
    expect(screen.getAllByTestId(/monthly-roster-mobile-day-/)).toHaveLength(9);
    expect(screen.getByTestId("monthly-roster-mobile-day-emp-1-2026-05-01")).toHaveAttribute(
      "aria-pressed",
      "true",
    );

    await user.click(screen.getByTestId("monthly-roster-mobile-day-emp-2-2026-05-01"));
    expect(onCellClick).toHaveBeenCalledWith("emp-2", "2026-05-01");
  });

  it("forwards keyboard activation for accessible day entries", () => {
    const onCellClick = vi.fn();
    const onCellKeyDown = vi.fn();

    render(
      <RosterMobileList
        currentLocale={enUS}
        monthDays={mockMonthDays.slice(0, 1)}
        monthSchedules={mockMonthSchedules.slice(0, 1) as any}
        onCellClick={onCellClick}
        onCellKeyDown={onCellKeyDown}
        selectedCells={new Set()}
        shifts={[]}
      />,
    );

    const day = screen.getByTestId("monthly-roster-mobile-day-emp-1-2026-05-01");
    day.focus();
    day.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));

    expect(onCellKeyDown).toHaveBeenCalledWith(
      "emp-1",
      "2026-05-01",
      expect.objectContaining({ key: "Enter" }),
    );
  });
});

describe("MonthlyRosterGrid Filtering", () => {
  it("renders empty state by default when no employees are selected", () => {
    const hook = createMockHook();
    const monthlyRoster = createMockMonthlyRoster({
      filteredSchedules: [],
    });
    render(<MonthlyRosterGrid hook={hook} monthlyRoster={monthlyRoster as any} />);

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
    const monthlyRoster1 = createMockMonthlyRoster({ filteredSchedules: [] });
    const { container: container1 } = render(<MonthlyRosterGrid hook={hook1} monthlyRoster={monthlyRoster1 as any} />);
    expect(container1.querySelectorAll('.animate-pulse').length).toBeGreaterThan(0);

    const hook2 = createMockHook({ isEmployeesLoading: true });
    const monthlyRoster2 = createMockMonthlyRoster({ filteredSchedules: [] });
    const { container: container2 } = render(<MonthlyRosterGrid hook={hook2} monthlyRoster={monthlyRoster2 as any} />);
    expect(container2.querySelectorAll('.animate-pulse').length).toBeGreaterThan(0);
  });

  it("displays error state with retry button when queries fail", async () => {
    const user = userEvent.setup();
    const hook = createMockHook({ isMonthError: true });
    const monthlyRoster = createMockMonthlyRoster({ filteredSchedules: [] });
    render(<MonthlyRosterGrid hook={hook} monthlyRoster={monthlyRoster as any} />);

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

  it("clears the whole selection with the clear-all button", async () => {
    const user = userEvent.setup();
    render(<StatefulGridWrapper />);

    // Select two employees.
    const searchInput = screen.getByTestId("mock-employee-search");
    await user.type(searchInput, "emp-1,emp-2");

    expect(screen.getAllByText("Alice Security").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Bob Patrol").length).toBeGreaterThanOrEqual(1);

    // One click on "Clear all" wipes the whole selection.
    await user.click(
      screen.getByRole("button", { name: /shifts.monthly.clear_all_employees/ }),
    );

    // Grid reverts to empty state and the checklist is gone.
    expect(screen.getByText("shifts.empty_state.title")).toBeInTheDocument();
    expect(screen.queryByRole("checkbox", { name: "Alice Security" })).not.toBeInTheDocument();
  });

  it("restores the cleared selection when undo is clicked", async () => {
    const user = userEvent.setup();
    render(<StatefulGridWrapper />);

    const searchInput = screen.getByTestId("mock-employee-search");
    await user.type(searchInput, "emp-1,emp-2");

    await user.click(
      screen.getByRole("button", { name: /shifts.monthly.clear_all_employees/ }),
    );

    // The undo pill reports how many were cleared.
    expect(screen.getByText("shifts.monthly.cleared_selection_2")).toBeInTheDocument();

    // Undo brings both employees back.
    await user.click(screen.getByRole("button", { name: /shifts.monthly.undo/ }));

    expect(screen.getByRole("checkbox", { name: "Alice Security" })).toBeChecked();
    expect(screen.getByRole("checkbox", { name: "Bob Patrol" })).toBeChecked();
    // Pill disappears after restoring.
    expect(screen.queryByText("shifts.monthly.cleared_selection_2")).not.toBeInTheDocument();
  });

  it("does not show an undo pill when clearing an already-empty selection", async () => {
    const _user = userEvent.setup();
    render(<StatefulGridWrapper />);

    // No employees selected → checklist and clear-all button are absent.
    expect(
      screen.queryByRole("button", { name: /shifts.monthly.clear_all_employees/ }),
    ).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /shifts.monthly.undo/ })).not.toBeInTheDocument();

    // Sanity: still on empty state.
    expect(screen.getByText("shifts.empty_state.title")).toBeInTheDocument();
  });
});
