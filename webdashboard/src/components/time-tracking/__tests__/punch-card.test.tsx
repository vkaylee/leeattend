import { render, screen, waitFor } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { TimeEntry } from "@/lib/api";

import Button from "@/components/ui/button";

import { PunchCard } from "../punch-card";

const mockTranslation = {
  "time_tracking.status.on_break": "On Break",
  "time_tracking.status.working": "Working",
  "time_tracking.status.off_duty": "Off Duty",
  "time_tracking.status.break_start_at": "Break Started At",
  "time_tracking.status.shift_start_at": "Shift Started At",
  "time_tracking.actions.hold_to_punch": "Hold to Punch",
  "time_tracking.forgot_punch": "Forgot to punch?",
};

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => mockTranslation[key as keyof typeof mockTranslation] || key,
  }),
}));

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatTime: (date: string) => new Date(date).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
    }),
  }),
}));

// Mock the HoldToPunchButton since we test it separately and we just want to test if it calls the right props here
vi.mock("../hold-to-punch-button", () => ({
  HoldToPunchButton: ({ onPunch, isPunching, label }: { isPunching: boolean; label: string; onPunch: () => void; }) => (
    <Button disabled={isPunching} onClick={onPunch} variant="ghost">
      {label}
    </Button>
  ),
}));

describe("PunchCard", () => {
  const mockOnPunch = vi.fn();
  const mockOnRequestAdjustment = vi.fn();

  const defaultProps = {
    activeEntry: undefined,
    userName: "John Doe",
    onPunch: mockOnPunch,
    onRequestAdjustment: mockOnRequestAdjustment,
    isPunching: false,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders off-duty status when no active entry", () => {
    render(<PunchCard {...defaultProps} />);

    expect(screen.getByText("John Doe")).toBeInTheDocument();
    expect(screen.getByText("Off Duty")).toBeInTheDocument();
    expect(screen.getByText("Hold to Punch")).toBeInTheDocument();
  });

  it("calls onPunchIn with 'work' when hold to punch button is triggered while off-duty", async () => {
    const user = userEvent.setup();
    render(<PunchCard {...defaultProps} />);

    const punchButton = screen.getByText("Hold to Punch");
    await user.click(punchButton);

    await waitFor(() => {
      expect(mockOnPunch).toHaveBeenCalled();
    });
  });

  it("renders working status with active entry", () => {
    const activeEntry = {
      id: "1",
      employment_id: "emp-1",
      workspace_id: "ws-1",
      punch_in_time: "2024-01-21T08:00:00Z",
      punch_out_time: undefined,
      entry_type: "work",
      is_break_overtime: false,
      created_at: "2024-01-21T08:00:00Z",
      updated_at: "2024-01-21T08:00:00Z"
    } as TimeEntry;

    render(<PunchCard {...defaultProps} activeEntry={activeEntry} />);

    expect(screen.getByText("Working")).toBeInTheDocument();
    expect(screen.getByText("Shift Started At")).toBeInTheDocument();
    expect(screen.getByText("Hold to Punch")).toBeInTheDocument();
  });

  it("calls onPunchOut when hold to punch button is triggered while working", async () => {
    const user = userEvent.setup();
    const activeEntry = {
      id: "1",
      employment_id: "emp-1",
      workspace_id: "ws-1",
      punch_in_time: "2024-01-21T08:00:00Z",
      punch_out_time: undefined,
      entry_type: "work",
      is_break_overtime: false,
      created_at: "2024-01-21T08:00:00Z",
      updated_at: "2024-01-21T08:00:00Z"
    } as TimeEntry;

    render(<PunchCard {...defaultProps} activeEntry={activeEntry} />);

    const punchButton = screen.getByText("Hold to Punch");
    await user.click(punchButton);

    await waitFor(() => {
      expect(mockOnPunch).toHaveBeenCalled();
    });
  });

  it("disables buttons when isPunching is true", () => {
    render(<PunchCard {...defaultProps} isPunching={true} />);

    const punchButton = screen.getByText("Hold to Punch");
    expect(punchButton).toBeDisabled();
  });

  it("calls onRequestAdjustment when forgot punch button clicked", async () => {
    const user = userEvent.setup();
    render(<PunchCard {...defaultProps} />);

    const forgotPunchButton = screen.getByText("Forgot to punch?");
    await user.click(forgotPunchButton);

    await waitFor(() => {
      expect(mockOnRequestAdjustment).toHaveBeenCalled();
    });
  });
});
