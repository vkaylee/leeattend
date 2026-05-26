import { render, screen, fireEvent, act } from "@testing-library/react";

import { describe, expect, it, vi, beforeEach } from "vitest";

import { PatternModals } from "../modals";

// Mock child components
vi.mock("@/components/ui/modal", () => ({
  default: ({ isOpen, onClose, children, title }: any) => (
    isOpen ? (
      <div data-testid="modal">
        <div className="text-lg font-semibold" data-testid="modal-title">{title}</div>
        {children}
        <button data-testid="close-modal" onClick={onClose}>Close</button>
      </div>
    ) : null
  ),
}));

vi.mock("@/components/ui/combobox", () => ({
  Combobox: ({ onChange, options }: any) => (
    <select data-testid="combobox" onChange={(e) => onChange(e.target.value)}>
      <option value="">Select...</option>
      {options.map((o: any) => (
        <option key={o.value} value={o.value}>{o.label}</option>
      ))}
    </select>
  ),
}));

vi.mock("@/components/common/employee-search", () => ({
  EmployeeSearch: ({ onChange }: any) => (
    <input data-testid="employee-search" onChange={() => onChange(["emp-1"])} />
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("PatternModals", () => {
  const mockHook = {
    addShiftToDay: vi.fn(),
    handleCreate: vi.fn(),
    isCreateModalOpen: false,
    newPatternDesc: "",
    newPatternName: "",
    prefillStandard: vi.fn(),
    removeShiftFromDay: vi.fn(),
    setIsCreateModalOpen: vi.fn(),
    setNewPatternDesc: vi.fn(),
    setNewPatternName: vi.fn(),
    shifts: [{ id: "shift-1", name: "Morning Shift" }],
    weeklyGrid: [[], [], [], [], [], [], []],
    patternType: "weekly" as const,
    setPatternType: vi.fn(),
    cycleLengthDays: 7,
    setCycleLengthDays: vi.fn(),
    assignStartDate: "2026-05-01",
    handleBulkAssign: vi.fn(),
    isBulkModalOpen: false,
    patterns: [{ id: "pattern-1", name: "Standard", pattern_type: "weekly", cycle_length_days: 7 }],
    selectedEmployeeIds: [],
    selectedPatternId: "",
    setAssignStartDate: vi.fn(),
    setIsBulkModalOpen: vi.fn(),
    setSelectedEmployeeIds: vi.fn(),
    setSelectedPatternId: vi.fn(),
    wsId: "ws-1",
    assignAnchorDate: "",
    setAssignAnchorDate: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders CreatePatternModal when open", () => {
    render(<PatternModals hook={{ ...mockHook, isCreateModalOpen: true } as any} />);
    expect(screen.getByText("patterns.modal.create_title")).toBeInTheDocument();
    
    // Test pattern type selector
    act(() => {
      fireEvent.click(screen.getByText("patterns.form.pattern_type_rotation"));
    });
    expect(mockHook.setPatternType).toHaveBeenCalledWith("rotation");

    // Test form inputs
    const nameInput = screen.getByPlaceholderText("patterns.form.name_placeholder");
    fireEvent.change(nameInput, { target: { value: "New Pattern" } });
    expect(mockHook.setNewPatternName).toHaveBeenCalledWith("New Pattern");

    // Test prefill standard button
    act(() => {
      fireEvent.click(screen.getByText("PATTERNS.FORM.PREFILL_STANDARD"));
    });
    expect(mockHook.prefillStandard).toHaveBeenCalled();

    // Test submit button
    act(() => {
      fireEvent.click(screen.getByText("PATTERNS.MODAL.SUBMIT_BTN"));
    });
    expect(mockHook.handleCreate).toHaveBeenCalled();
  });

  it("renders CreatePatternModal as rotation type", () => {
    render(<PatternModals hook={{ ...mockHook, isCreateModalOpen: true, patternType: "rotation", cycleLengthDays: 5 } as any} />);
    
    // Cycle length input should be present
    const cycleInput = screen.getByDisplayValue("5");
    expect(cycleInput).toBeInTheDocument();

    fireEvent.change(cycleInput, { target: { value: "6" } });
    expect(mockHook.setCycleLengthDays).toHaveBeenCalledWith(6);
  });

  it("handles day row interactions in CreatePatternModal", () => {
    const hookWithShifts = {
      ...mockHook,
      isCreateModalOpen: true,
      weeklyGrid: [["shift-1"], [], [], [], [], [], []],
    };
    render(<PatternModals hook={hookWithShifts as any} />);
    
    // Remove shift
    const removeBtns = screen.getAllByRole("button", { name: /Remove Morning Shift/i });
    expect(removeBtns.length).toBe(1);
    fireEvent.click(removeBtns[0]);
    expect(mockHook.removeShiftFromDay).toHaveBeenCalledWith(0, "shift-1");

    // Add shift
    const comboboxes = screen.getAllByTestId("combobox");
    expect(comboboxes.length).toBeGreaterThan(0);
    fireEvent.change(comboboxes[1], { target: { value: "shift-1" } });
    expect(mockHook.addShiftToDay).toHaveBeenCalledWith(1, "shift-1");
  });

  it("renders BulkAssignModal when open", () => {
    render(<PatternModals hook={{ ...mockHook, isBulkModalOpen: true, selectedPatternId: "pattern-1", selectedEmployeeIds: ["emp-1"] } as any} />);
    expect(screen.getByText("patterns.bulk_assign_btn")).toBeInTheDocument();

    // Select pattern
    fireEvent.click(screen.getByText("Standard"));
    expect(mockHook.setSelectedPatternId).toHaveBeenCalledWith("pattern-1");

    // Change start date
    const dateInput = screen.getAllByDisplayValue("2026-05-01")[0];
    fireEvent.change(dateInput, { target: { value: "2026-05-02" } });
    expect(mockHook.setAssignStartDate).toHaveBeenCalledWith("2026-05-02");

    // Change employees
    const empSearch = screen.getByTestId("employee-search");
    fireEvent.change(empSearch, { target: { value: "search" } });
    expect(mockHook.setSelectedEmployeeIds).toHaveBeenCalledWith(["emp-1"]);

    // Submit assign
    act(() => {
      fireEvent.click(screen.getByText("PATTERNS.MODAL.ASSIGN_BTN"));
    });
    expect(mockHook.handleBulkAssign).toHaveBeenCalled();
  });

  it("renders rotation specific fields in BulkAssignModal", () => {
    const rotationPatternHook = {
      ...mockHook,
      isBulkModalOpen: true,
      selectedPatternId: "pattern-2",
      patterns: [{ id: "pattern-2", name: "Rotation 4x4", pattern_type: "rotation" as const, cycle_length_days: 8 }],
      assignAnchorDate: "2026-05-01",
    };
    render(<PatternModals hook={rotationPatternHook as any} />);
    
    // Anchor date should be visible
    expect(screen.getByText("PATTERNS.FORM.ANCHOR_DATE_LABEL")).toBeInTheDocument();
    const anchorInput = screen.getAllByDisplayValue("2026-05-01")[1];
    fireEvent.change(anchorInput, { target: { value: "2026-05-03" } });
    expect(mockHook.setAssignAnchorDate).toHaveBeenCalledWith("2026-05-03");
  });
});
