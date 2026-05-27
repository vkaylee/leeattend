import { render, screen, fireEvent } from "@testing-library/react";

import { describe, expect, it, vi, beforeEach } from "vitest";

import { WorkingPattern, WorkingPatternDetail, Shift } from "@/lib/api/types";

import { PatternCard } from "../pattern-card";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: any) => opts?.index ? `${key}_${opts.index}` : key,
  }),
}));

describe("PatternCard", () => {
  const mockPattern = {
    id: "pattern-1",
    name: "Weekly Standard",
    description: "Standard 9-5",
    is_active: true,
    pattern_type: "weekly",
  } as unknown as WorkingPattern;

  const mockRotationPattern = {
    id: "pattern-2",
    name: "Rotation 4x4",
    description: "4 days on, 4 days off",
    is_active: false,
    pattern_type: "rotation",
    cycle_length_days: 8,
  } as unknown as WorkingPattern;

  const mockDetails = [
    { schedule_day: 0, shift_id: "shift-1" },
    { schedule_day: 1, shift_id: "shift-1" },
  ] as unknown as WorkingPatternDetail[];

  const mockShifts = [
    { id: "shift-1", name: "Morning Shift" }
  ] as unknown as Shift[];

  const mockOnDelete = vi.fn();
  const mockOnEdit = vi.fn();
  const mockOnToggleExpand = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders weekly pattern basic info", () => {
    render(
      <PatternCard
        expandedPatternId={null}
        onDelete={mockOnDelete}
        onEdit={mockOnEdit}
        onToggleExpand={mockOnToggleExpand}
        pattern={mockPattern}
      />
    );

    expect(screen.getByText("Weekly Standard")).toBeInTheDocument();
    expect(screen.getByText("Standard 9-5")).toBeInTheDocument();
    expect(screen.getByText("status_active_caps")).toBeInTheDocument();
    expect(screen.getByText("patterns.form.pattern_type_weekly")).toBeInTheDocument();
  });

  it("renders rotation pattern basic info", () => {
    render(
      <PatternCard
        expandedPatternId={null}
        onDelete={mockOnDelete}
        onEdit={mockOnEdit}
        onToggleExpand={mockOnToggleExpand}
        pattern={mockRotationPattern}
      />
    );

    expect(screen.getByText("Rotation 4x4")).toBeInTheDocument();
    expect(screen.getByText("status_off")).toBeInTheDocument();
    expect(screen.getByText("patterns.form.pattern_type_rotation (8d)")).toBeInTheDocument();
  });

  it("handles expand toggle", () => {
    render(
      <PatternCard
        expandedPatternId={null}
        onDelete={mockOnDelete}
        onEdit={mockOnEdit}
        onToggleExpand={mockOnToggleExpand}
        pattern={mockPattern}
      />
    );

    const toggleBtn = screen.getByRole("button", { name: /patterns.view_details/i });
    fireEvent.click(toggleBtn);
    expect(mockOnToggleExpand).toHaveBeenCalledWith("pattern-1");
  });

  it("renders expanded weekly details", () => {
    render(
      <PatternCard
        expandedPatternId={"pattern-1"}
        onDelete={mockOnDelete}
        onEdit={mockOnEdit}
        onToggleExpand={mockOnToggleExpand}
        pattern={mockPattern}
        patternDetails={mockDetails}
        shifts={mockShifts}
      />
    );

    // Should render days
    expect(screen.getByText("days.sunday_short")).toBeInTheDocument();
    const morningShifts = screen.getAllByText("Morning Shift");
    expect(morningShifts.length).toBeGreaterThan(0);
    // Days without shifts should show "status_off"
    const statusOff = screen.getAllByText("status_off");
    expect(statusOff.length).toBeGreaterThan(0);
  });

  it("renders expanded rotation details", () => {
    render(
      <PatternCard
        expandedPatternId={"pattern-2"}
        onDelete={mockOnDelete}
        onEdit={mockOnEdit}
        onToggleExpand={mockOnToggleExpand}
        pattern={mockRotationPattern}
        patternDetails={mockDetails}
        shifts={mockShifts}
      />
    );

    // Should render cycle days
    expect(screen.getByText("patterns.form.day_index_1")).toBeInTheDocument();
    expect(screen.getByText("patterns.form.day_index_8")).toBeInTheDocument();
  });

  it("handles edit and delete actions", () => {
    render(
      <PatternCard
        expandedPatternId={null}
        onDelete={mockOnDelete}
        onEdit={mockOnEdit}
        onToggleExpand={mockOnToggleExpand}
        pattern={mockPattern}
      />
    );

    const editBtn = screen.getByTitle("common.edit");
    const deleteBtn = screen.getByTitle("common.delete");

    fireEvent.click(editBtn);
    expect(mockOnEdit).toHaveBeenCalledWith(mockPattern);

    fireEvent.click(deleteBtn);
    expect(mockOnDelete).toHaveBeenCalledWith(mockPattern);
  });
});
