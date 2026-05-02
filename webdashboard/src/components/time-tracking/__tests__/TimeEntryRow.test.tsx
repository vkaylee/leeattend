import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { TimeEntry } from "@/lib/api/types";

import { TimeEntryRow } from "../TimeEntryRow";

const mockTranslation = {
  "common.break": "Break",
  "common.work": "Work",
  "common.adjust": "Adjust",
  "time_tracking.status.on_break": "On Break",
  "time_tracking.status.working": "Working",
};

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => mockTranslation[key as keyof typeof mockTranslation] || key,
  }),
}));

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatDate: (_date: string) => "2024-01-21",
    formatTime: (_date: string) => "08:00 AM",
  }),
}));

describe("TimeEntryRow", () => {
  const mockOnAdjust = vi.fn();

  const workEntry = {
    id: "1",
    employee_id: "emp-1",
    punch_in_time: "2024-01-21T08:00:00Z",
    punch_out_time: "2024-01-21T17:00:00Z",
    entry_type: "work",
    is_break_overtime: false,
    created_at: "2024-01-21T08:00:00Z",
    updated_at: "2024-01-21T17:00:00Z"
  };

  const breakEntry = {
    id: "2",
    employee_id: "emp-1",
    punch_in_time: "2024-01-21T12:00:00Z",
    punch_out_time: "2024-01-21T12:30:00Z",
    entry_type: "break",
    is_break_overtime: false,
    created_at: "2024-01-21T12:00:00Z",
    updated_at: "2024-01-21T12:30:00Z"
  };

  const activeEntry: TimeEntry = {
    id: "3",
    employee_id: "emp-1",
    punch_in_time: "2024-01-21T08:00:00Z",
    punch_out_time: undefined,
    entry_type: "work",
    is_break_overtime: false,
    created_at: "2024-01-21T08:00:00Z",
    updated_at: "2024-01-21T08:00:00Z"
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders work entry with punch in and punch out times", () => {
    render(
      <table>
        <tbody>
          <TimeEntryRow entry={workEntry} />
        </tbody>
      </table>
    );

    expect(screen.getByText("2024-01-21")).toBeInTheDocument();
    expect(screen.getByText("Work")).toBeInTheDocument();
    expect(screen.getAllByText("08:00 AM")).toHaveLength(2);
  });

  it("renders break entry with break badge", () => {
    render(
      <table>
        <tbody>
          <TimeEntryRow entry={breakEntry} />
        </tbody>
      </table>
    );

    expect(screen.getByText("Break")).toBeInTheDocument();
  });

  it("renders active entry without punch out time", () => {
    render(
      <table>
        <tbody>
          <TimeEntryRow entry={activeEntry} />
        </tbody>
      </table>
    );

    expect(screen.getByText("Working")).toBeInTheDocument();
  });

  it("shows adjust button when showAdjustButton is true", () => {
    render(
      <table>
        <tbody>
          <TimeEntryRow entry={workEntry} onAdjust={mockOnAdjust} showAdjustButton={true} />
        </tbody>
      </table>
    );

    expect(screen.getByText("Adjust")).toBeInTheDocument();
  });

  it("does not show adjust button when showAdjustButton is false", () => {
    render(
      <table>
        <tbody>
          <TimeEntryRow entry={workEntry} onAdjust={mockOnAdjust} showAdjustButton={false} />
        </tbody>
      </table>
    );

    expect(screen.queryByText("Adjust")).not.toBeInTheDocument();
  });

  it("calls onAdjust with entry when adjust button clicked", async () => {
    const user = userEvent.setup();
    render(
      <table>
        <tbody>
          <TimeEntryRow entry={workEntry} onAdjust={mockOnAdjust} showAdjustButton={true} />
        </tbody>
      </table>
    );

    const adjustButton = screen.getByText("Adjust");
    await user.click(adjustButton);

    await waitFor(() => {
      expect(mockOnAdjust).toHaveBeenCalledWith(workEntry);
    });
  });
});
