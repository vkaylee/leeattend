import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PunchCard } from "../PunchCard";

const mockTranslation = {
  "time_tracking.status.on_break": "On Break",
  "time_tracking.status.working": "Working",
  "time_tracking.status.off_duty": "Off Duty",
  "time_tracking.status.break_start_at": "Break Started At",
  "time_tracking.status.shift_start_at": "Shift Started At",
  "time_tracking.actions.return_to_work": "Return to Work",
  "time_tracking.actions.start_break": "Start Break",
  "time_tracking.actions.punch_out": "Punch Out",
  "time_tracking.actions.start_work": "Start Work",
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

describe("PunchCard", () => {
  const mockOnPunchIn = vi.fn();
  const mockOnPunchOut = vi.fn();
  const mockOnRequestAdjustment = vi.fn();

  const defaultProps = {
    activeEntry: undefined,
    isOnBreak: false,
    userName: "John Doe",
    onPunchIn: mockOnPunchIn,
    onPunchOut: mockOnPunchOut,
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
    expect(screen.getByText("Start Work")).toBeInTheDocument();
  });

  it("calls onPunchIn with 'work' when start work button clicked", async () => {
    const user = userEvent.setup();
    render(<PunchCard {...defaultProps} />);

    const startWorkButton = screen.getByText("Start Work");
    await user.click(startWorkButton);

    await waitFor(() => {
      expect(mockOnPunchIn).toHaveBeenCalledWith("work");
    });
  });

  it("renders working status with active entry", () => {
    const activeEntry = {
      id: "1",
      employee_id: "emp-1",
      punch_in_time: "2024-01-21T08:00:00Z",
      punch_out_time: undefined,
      entry_type: "work",
      is_break_overtime: false,
      created_at: "2024-01-21T08:00:00Z",
      updated_at: "2024-01-21T08:00:00Z"
    };

    render(<PunchCard {...defaultProps} activeEntry={activeEntry} />);

    expect(screen.getByText("Working")).toBeInTheDocument();
    expect(screen.getByText("Shift Started At")).toBeInTheDocument();
    expect(screen.getByText("Start Break")).toBeInTheDocument();
    expect(screen.getByText("Punch Out")).toBeInTheDocument();
  });

  it("calls onPunchIn with 'break' when start break button clicked", async () => {
    const user = userEvent.setup();
    const activeEntry = {
      id: "1",
      employee_id: "emp-1",
      punch_in_time: "2024-01-21T08:00:00Z",
      punch_out_time: undefined,
      entry_type: "work",
      is_break_overtime: false,
      created_at: "2024-01-21T08:00:00Z",
      updated_at: "2024-01-21T08:00:00Z"
    };

    render(<PunchCard {...defaultProps} activeEntry={activeEntry} />);

    const startBreakButton = screen.getByText("Start Break");
    await user.click(startBreakButton);

    await waitFor(() => {
      expect(mockOnPunchIn).toHaveBeenCalledWith("break");
    });
  });

  it("calls onPunchOut when punch out button clicked", async () => {
    const user = userEvent.setup();
    const activeEntry = {
      id: "1",
      employee_id: "emp-1",
      punch_in_time: "2024-01-21T08:00:00Z",
      punch_out_time: undefined,
      entry_type: "work",
      is_break_overtime: false,
      created_at: "2024-01-21T08:00:00Z",
      updated_at: "2024-01-21T08:00:00Z"
    };

    render(<PunchCard {...defaultProps} activeEntry={activeEntry} />);

    const punchOutButton = screen.getByText("Punch Out");
    await user.click(punchOutButton);

    await waitFor(() => {
      expect(mockOnPunchOut).toHaveBeenCalled();
    });
  });

  it("renders on-break status", () => {
    const activeEntry = {
      id: "1",
      employee_id: "emp-1",
      punch_in_time: "2024-01-21T08:00:00Z",
      punch_out_time: undefined,
      entry_type: "break",
      is_break_overtime: false,
      created_at: "2024-01-21T08:00:00Z",
      updated_at: "2024-01-21T08:00:00Z"
    };

    render(<PunchCard {...defaultProps} activeEntry={activeEntry} isOnBreak={true} />);

    expect(screen.getByText("On Break")).toBeInTheDocument();
    expect(screen.getByText("Break Started At")).toBeInTheDocument();
    expect(screen.getByText("Return to Work")).toBeInTheDocument();
  });

  it("calls onPunchOut when return to work button clicked", async () => {
    const user = userEvent.setup();
    const activeEntry = {
      id: "1",
      employee_id: "emp-1",
      punch_in_time: "2024-01-21T08:00:00Z",
      punch_out_time: undefined,
      entry_type: "break",
      is_break_overtime: false,
      created_at: "2024-01-21T08:00:00Z",
      updated_at: "2024-01-21T08:00:00Z"
    };

    render(<PunchCard {...defaultProps} activeEntry={activeEntry} isOnBreak={true} />);

    const returnButton = screen.getByText("Return to Work");
    await user.click(returnButton);

    await waitFor(() => {
      expect(mockOnPunchOut).toHaveBeenCalled();
    });
  });

  it("disables buttons when isPunching is true", () => {
    render(<PunchCard {...defaultProps} isPunching={true} />);

    const startWorkButton = screen.getByText("Start Work");
    expect(startWorkButton).toBeDisabled();
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
