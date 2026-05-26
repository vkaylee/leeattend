import { render } from "@testing-library/react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import { ShiftsContent } from "../content";
import { ShiftsModals } from "../modals";
import { ShiftsTabs } from "../tabs";
import { ShiftsHeader } from "../header";

vi.mock("@/components/shifts/monthly-roster-grid", () => ({ MonthlyRosterGrid: () => <div data-testid="monthly-roster" /> }));
vi.mock("@/components/shifts/weekly-roster-grid", () => ({ WeeklyRosterGrid: (props: any) => {
  return (
    <div data-testid="weekly-roster">
      <button data-testid="btn-assign-all" onClick={props.onAssignAllWeek} />
      <button data-testid="btn-clear" onClick={props.onClearSelection} />
      <button data-testid="btn-clear-week" onClick={props.onClearWeek} />
      <button data-testid="btn-copy" onClick={props.onCopyPreviousWeek} />
      <button data-testid="btn-delete" onClick={() => props.onDeleteRoster("2026-01-01")} />
      <button data-testid="btn-nav" onClick={() => props.onNavigateWeek("next")} />
      <button data-testid="btn-open-picker" onClick={() => props.onOpenShiftPicker(["2026-01-01"])} />
      <button data-testid="btn-toggle" onClick={() => props.onToggleDaySelection("2026-01-01")} />
      <button data-testid="btn-toggle-existing" onClick={() => props.onToggleDaySelection("2026-01-02")} />
    </div>
  );
}}));
vi.mock("@/components/shifts/pattern-assignment-form", () => ({ PatternAssignmentForm: () => <div data-testid="pattern-form" /> }));
vi.mock("@/components/shifts/shift-list", () => ({ ShiftList: () => <div data-testid="shift-list" /> }));
vi.mock("@/components/shifts/priority-legend", () => ({ PriorityLegend: () => <div data-testid="priority-legend" /> }));
vi.mock("@/components/shifts/assignment-history", () => ({ AssignmentHistory: () => <div data-testid="assignment-history" /> }));

vi.mock("@/components/shifts/create-shift-modal", () => ({ CreateShiftModal: (props: any) => <div data-testid="create-modal" onClick={props.onClose} /> }));
vi.mock("@/components/shifts/shift-picker-modal", () => ({ ShiftPickerModal: (props: any) => <div data-testid="picker-modal" onClick={props.onClose} /> }));
vi.mock("@/components/shifts/edit-shift-modal", () => ({ EditShiftModal: (props: any) => <div data-testid="edit-modal" onClick={props.onClose} /> }));

describe("Shifts Components Unit Tests", () => {
  const mockHook: any = {
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
    setSelectedDays: vi.fn(),
    setSelectedPatternId: vi.fn(),
    setStartDate: vi.fn(),
    setTargetDates: vi.fn(),
    setEndDate: vi.fn(),
    shifts: [],
    startDate: new Date(),
    weekDays: [new Date()],
    isAutoDeduct: false,
    isCreateModalOpen: true,
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
    newShiftGrace: "0",
    setCorrectionEffectiveDate: vi.fn(),
    setEditingShift: vi.fn(),
    setIsCorrection: vi.fn(),
    setNewShiftGrace: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders ShiftsContent MonthlyRosterGrid", () => {
    const { getByTestId } = render(<ShiftsContent hook={mockHook} />);
    expect(getByTestId("monthly-roster")).toBeDefined();
  });

  it("renders null for Weekly view without selected employee", () => {
    const hook = { ...mockHook, activeTab: "weekly-roster", viewMode: "week", selectedEmployee: null };
    const { container } = render(<ShiftsContent hook={hook} />);
    expect(container.firstChild).toBeNull();
  });

  it("renders WeeklyRosterGrid and handles callbacks", () => {
    const hook = { ...mockHook, activeTab: "weekly-roster", viewMode: "week" };
    const { getByTestId } = render(<ShiftsContent hook={hook} />);
    expect(getByTestId("weekly-roster")).toBeDefined();

    getByTestId("btn-assign-all").click();
    expect(hook.setTargetDates).toHaveBeenCalled();
    expect(hook.setIsShiftPickerOpen).toHaveBeenCalledWith(true);

    getByTestId("btn-clear").click();
    expect(hook.setSelectedDays).toHaveBeenCalledWith([]);

    getByTestId("btn-clear-week").click();
    expect(hook.handleAssignAllWeek).toHaveBeenCalledWith(null);

    getByTestId("btn-copy").click();
    expect(hook.handleCopyPreviousWeek).toHaveBeenCalled();

    getByTestId("btn-delete").click();
    expect(hook.deleteRosterMutation.mutate).toHaveBeenCalledWith("2026-01-01");

    getByTestId("btn-nav").click();
    expect(hook.handleNavigateWeek).toHaveBeenCalledWith("next");

    getByTestId("btn-open-picker").click();
    expect(hook.setTargetDates).toHaveBeenCalledWith(["2026-01-01"]);

    getByTestId("btn-toggle").click();
    expect(hook.setSelectedDays).toHaveBeenCalledWith(["2026-01-02", "2026-01-01"]);

    getByTestId("btn-toggle-existing").click();
    expect(hook.setSelectedDays).toHaveBeenCalledWith([]);
  });

  it("renders PatternAssignmentForm", () => {
    const hook = { ...mockHook, activeTab: "assignments" };
    const { getByTestId } = render(<ShiftsContent hook={hook} />);
    expect(getByTestId("pattern-form")).toBeDefined();
  });

  it("renders ShiftList", () => {
    const hook = { ...mockHook, activeTab: "directory" };
    const { getByTestId } = render(<ShiftsContent hook={hook} />);
    expect(getByTestId("shift-list")).toBeDefined();
  });

  it("renders ShiftsModals and handles close events", () => {
    const { getByTestId } = render(<ShiftsModals hook={mockHook} />);
    expect(getByTestId("create-modal")).toBeDefined();
    expect(getByTestId("picker-modal")).toBeDefined();
    expect(getByTestId("edit-modal")).toBeDefined();

    getByTestId("create-modal").click();
    expect(mockHook.setIsCreateModalOpen).toHaveBeenCalledWith(false);

    getByTestId("picker-modal").click();
    expect(mockHook.setIsShiftPickerOpen).toHaveBeenCalledWith(false);
    expect(mockHook.setTargetDates).toHaveBeenCalledWith([]);

    getByTestId("edit-modal").click();
    expect(mockHook.setEditingShift).toHaveBeenCalledWith(null);
  });
});
