import { render, screen } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";

import { Shift } from "../../../lib/api";
import { ShiftList } from "../shift-list";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "common.total") return "TOTAL";
      return key;
    },
  }),
}));

// Mock hook useDateTimeFormatter
vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatTime: (time: string) => time ? time.substring(0, 5) : "",
  }),
}));

const mockShifts: Shift[] = [
  {
    id: "shift-1",
    name: "Morning Shift",
    start_time: "08:00:00",
    end_time: "17:00:00",
    break_start: "12:00:00",
    break_end: "13:00:00",
    break_duration_minutes: 60,
    is_break_auto_deduct: true,
  } as unknown as Shift,
  {
    id: "shift-2",
    name: "Night Shift",
    start_time: "22:00:00",
    end_time: "06:00:00",
    break_start: null,
    break_end: null,
    break_duration_minutes: 30,
    is_break_auto_deduct: false,
  } as unknown as Shift,
];

describe("ShiftList Component", () => {
  const onEdit = vi.fn();
  const onDelete = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders shifts list with correct layout", () => {
    render(<ShiftList onDelete={onDelete} onEdit={onEdit} shifts={mockShifts} />);

    expect(screen.getByText("shifts.directory_title")).toBeInTheDocument();
    expect(screen.getByText("TOTAL: 2")).toBeInTheDocument();

    // Check Morning Shift Row
    expect(screen.getByText("Morning Shift")).toBeInTheDocument();
    expect(screen.getByText("08:00")).toBeInTheDocument();
    expect(screen.getByText("17:00")).toBeInTheDocument();
    expect(screen.getByText("12:00")).toBeInTheDocument();
    expect(screen.getByText("13:00")).toBeInTheDocument();
    expect(screen.getByText("shifts.auto_deduct")).toBeInTheDocument();

    // Check Night Shift Row (with break duration minutes only)
    expect(screen.getByText("Night Shift")).toBeInTheDocument();
    expect(screen.getByText("22:00")).toBeInTheDocument();
    expect(screen.getByText("06:00")).toBeInTheDocument();
    expect(screen.getByText("30 common.minutes")).toBeInTheDocument();
  });

  it("calls onEdit when edit button is clicked", async () => {
    const user = userEvent.setup();
    render(<ShiftList onDelete={onDelete} onEdit={onEdit} shifts={mockShifts} />);

    const editButtons = screen.getAllByTitle("common.edit");
    await user.click(editButtons[0]);

    expect(onEdit).toHaveBeenCalledWith(mockShifts[0]);
  });

  it("calls onDelete when delete button is clicked", async () => {
    const user = userEvent.setup();
    render(<ShiftList onDelete={onDelete} onEdit={onEdit} shifts={mockShifts} />);

    const deleteButtons = screen.getAllByTitle("common.delete");
    await user.click(deleteButtons[0]);

    expect(onDelete).toHaveBeenCalledWith(mockShifts[0]);
  });

  it("renders empty state message when shifts is empty", () => {
    render(<ShiftList onDelete={onDelete} onEdit={onEdit} shifts={[]} />);

    expect(screen.getByText("shifts.messages.no_shifts")).toBeInTheDocument();
    expect(screen.getByText("shifts.directory_title is empty")).toBeInTheDocument();
  });
});
