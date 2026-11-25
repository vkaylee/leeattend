import { render } from "@testing-library/react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import Button from "@/components/ui/button";

import { ShiftsContent } from "../content";
import { ShiftsModals } from "../modals";

vi.mock("@/components/shifts/schedule-lifecycle-panel", () => ({ ScheduleLifecyclePanel: () => <div data-testid="schedule-lifecycle-panel" /> }));
vi.mock("@/components/shifts/monthly-roster-grid", () => ({ MonthlyRosterGrid: () => <div data-testid="monthly-roster" /> }));
vi.mock("@/components/shifts/weekly-roster-grid", () => ({ WeeklyRosterGrid: (props: any) => {
  return (
    <div data-testid="weekly-roster">
      <Button data-testid="btn-clear" onClick={props.onClearSelection} variant="destructive">Clear</Button>
      <Button data-testid="btn-delete" onClick={() => props.onDeleteRoster("2026-01-01")} variant="destructive">Delete</Button>
      <Button data-testid="btn-open-picker" onClick={() => props.onOpenShiftPicker(["2026-01-01"])} variant="ghost">Picker</Button>
      <Button data-testid="btn-toggle" onClick={() => props.onToggleDaySelection("2026-01-01")} variant="ghost">Toggle</Button>
      <Button data-testid="btn-toggle-existing" onClick={() => props.onToggleDaySelection("2026-01-02")} variant="ghost">Toggle Ex</Button>
    </div>
  );
}}));
vi.mock("@/components/shifts/pattern-assignment-form", () => ({ PatternAssignmentForm: () => <div data-testid="pattern-form" /> }));
vi.mock("@/components/shifts/shift-list", () => ({ ShiftList: () => <div data-testid="shift-list" /> }));
vi.mock("@/components/shifts/priority-legend", () => ({ PriorityLegend: () => <div data-testid="priority-legend" /> }));
vi.mock("@/components/shifts/assignment-history", () => ({ AssignmentHistory: () => <div data-testid="assignment-history" /> }));
vi.mock("@/components/shifts/compliance-warning-modal", () => ({
  ComplianceWarningModal: (props: any) => props.isOpen ? (
    <Button data-testid="compliance-modal" onClick={() => props.onConfirm("Approved override")} variant="ghost">
      Confirm override
    </Button>
  ) : null,
}));

vi.mock("@/components/shifts/shift-picker-modal", () => ({ ShiftPickerModal: (props: any) => <Button data-testid="picker-modal" onClick={props.onClose} variant="ghost">Close</Button> }));
vi.mock("@/components/ui/modal", () => ({
  default: ({ children, isOpen }: any) => isOpen ? <div data-testid="ui-modal">{children}</div> : null
}));

describe("Shifts Components Unit Tests", () => {
  const mockHook = {
    activeTab: "weekly-roster",
    viewMode: "month",
    assignPatternMutation: { isPending: false },
    deleteRosterMutation: { mutate: vi.fn() },
    updateShiftMutation: { isPending: false },
    createShiftMutation: { isPending: false },
    currentDate: new Date(),
    employeeSchedule: [],
    endDate: null,
    handleAssign: vi.fn(),
    handleAssignAllWeek: vi.fn(),
    handleCopyPreviousWeek: vi.fn(),
    handleNavigateWeek: vi.fn(),
    history: [],
    isForever: true,
    openEditModal: vi.fn(),
    patterns: [],
    selectedDays: ["2026-01-02"],
    selectedEmployee: { id: "emp-1" },
    selectedPatternId: "pat-1",
    setIsForever: vi.fn(),
    setIsShiftPickerOpen: vi.fn(),
    isPatternModalOpen: false,
    setIsPatternModalOpen: vi.fn(),
    setSelectedDays: vi.fn(),
    setSelectedPatternId: vi.fn(),
    setStartDate: vi.fn(),
    setTargetDates: vi.fn(),
    setEndDate: vi.fn(),
    shifts: [],
    startDate: new Date(),
    weekDays: [new Date()],
    isCreateModalOpen: true,
    breakPunchMode: "none",
    isFixedBreak: false,
    newBreakEnd: "",
    newBreakStart: "",
    newShiftBreak: "0",
    newShiftEnd: "",
    newShiftName: "",
    newShiftStart: "",
    newShiftSymbol: "",
    setIsAutoDeduct: vi.fn(),
    setIsCreateModalOpen: vi.fn(),
    setIsFixedBreak: vi.fn(),
    setNewBreakEnd: vi.fn(),
    setNewBreakStart: vi.fn(),
    setNewShiftBreak: vi.fn(),
    setNewShiftEnd: vi.fn(),
    setNewShiftName: vi.fn(),
    setNewShiftStart: vi.fn(),
    setNewShiftSymbol: vi.fn(),
    handleCreateShift: vi.fn(),
    handleAssignBulk: vi.fn(),
    isShiftPickerOpen: true,
    targetDates: [],
    correctionEffectiveDate: null,
    editingShift: null,
    handleUpdateShift: vi.fn(),
    isCorrection: false,
    setCorrectionEffectiveDate: vi.fn(),
    setEditingShift: vi.fn(),
    setIsCorrection: vi.fn(),
    setNewShiftGrace: vi.fn(),
    complianceModalOpen: false,
    complianceViolations: [],
    isComplianceResolving: false,
    onComplianceOverride: vi.fn(),
    setComplianceModalOpen: vi.fn(),
  };

  const mockMonthlyRoster = {
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
    filteredSchedules: [],
    isDragging: false,
    handleCellPointerDown: vi.fn(),
    handleCellPointerEnter: vi.fn(),
    handleApplyShift: vi.fn(),
    handleClearSelectedOverrides: vi.fn(),
    handleCopyMonth: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders ShiftsContent MonthlyRosterGrid", () => {
    const { getByTestId } = render(<ShiftsContent hook={mockHook as any} monthlyRoster={mockMonthlyRoster as any} />);
    expect(getByTestId("monthly-roster")).toBeDefined();
  });

  it("renders null for Weekly view without selected employee", () => {
    const hook = { ...mockHook, activeTab: "weekly-roster", viewMode: "week", selectedEmployee: null };
    const { container } = render(<ShiftsContent hook={hook as any} monthlyRoster={mockMonthlyRoster as any} />);
    expect(container.firstChild).toBeNull();
  });

  it("renders WeeklyRosterGrid and handles callbacks", () => {
    const hook = { ...mockHook, activeTab: "weekly-roster", viewMode: "week" };
    const { getByTestId } = render(<ShiftsContent hook={hook as any} monthlyRoster={mockMonthlyRoster as any} />);
    expect(getByTestId("weekly-roster")).toBeDefined();

    getByTestId("btn-clear").click();
    expect(hook.setSelectedDays).toHaveBeenCalledWith([]);

    getByTestId("btn-delete").click();
    expect(hook.deleteRosterMutation.mutate).toHaveBeenCalledWith("2026-01-01");

    getByTestId("btn-open-picker").click();
    expect(hook.setTargetDates).toHaveBeenCalledWith(["2026-01-01"]);

    getByTestId("btn-toggle").click();
    expect(hook.setSelectedDays).toHaveBeenCalledWith(["2026-01-02", "2026-01-01"]);

    getByTestId("btn-toggle-existing").click();
    expect(hook.setSelectedDays).toHaveBeenCalledWith([]);
  });

  it("renders PatternAssignmentForm inside ShiftsModals", () => {
    const hook = { ...mockHook, isPatternModalOpen: true };
    const { getByTestId } = render(<ShiftsModals hook={hook as any} />);
    expect(getByTestId("pattern-form")).toBeDefined();
  });

  it("renders ShiftList", () => {
    const hook = { ...mockHook, activeTab: "directory" };
    const { getByTestId } = render(<ShiftsContent hook={hook as any} monthlyRoster={mockMonthlyRoster as any} />);
    expect(getByTestId("shift-list")).toBeDefined();
  });

  it("renders ShiftsModals and handles close events", () => {
    const { getByTestId } = render(<ShiftsModals hook={mockHook as any} />);
    expect(getByTestId("picker-modal")).toBeDefined();

    getByTestId("picker-modal").click();
    expect(mockHook.setIsShiftPickerOpen).toHaveBeenCalledWith(false);
    expect(mockHook.setTargetDates).toHaveBeenCalledWith([]);
  });

  it("renders compliance override feedback for monthly roster conflicts", () => {
    const hook = {
      ...mockHook,
      complianceModalOpen: true,
      complianceViolations: [{ violation_type: "rest_period", message: "Insufficient rest" }],
    };
    const { getByTestId } = render(<ShiftsModals hook={hook as any} />);

    getByTestId("compliance-modal").click();

    expect(hook.onComplianceOverride).toHaveBeenCalledWith("Approved override");
  });
});
