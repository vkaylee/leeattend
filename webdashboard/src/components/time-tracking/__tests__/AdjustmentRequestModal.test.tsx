import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { TimeEntry } from "@/lib/api/types";

import { AdjustmentRequestModal } from "../AdjustmentRequestModal";


const mockTranslation = {
  "time_tracking.modal.new_title": "Request Time Adjustment",
  "time_tracking.modal.edit_title": "Edit Time Adjustment",
  "time_tracking.modal.subtitle": "Submit a request to adjust your time entry",
  "time_tracking.modal.reason_label": "Reason",
  "time_tracking.modal.reason_placeholder": "Please explain why you need this adjustment",
  "common.work_day": "Work Day",
  "common.punch_in": "Punch In",
  "common.punch_out": "Punch Out",
  "common.cancel": "Cancel",
  "common.submit_request": "Submit Request",
  "common.sending": "Sending",
  "attendance_confirm.ot_label": "Break/Overtime",
};

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => mockTranslation[key as keyof typeof mockTranslation] || key,
  }),
}));

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatTime24h: (_date: string) => "08:00",
  }),
}));

vi.mock("@/components/ui/Modal", () => ({
  __esModule: true,
  default: ({ isOpen, children, title }: { isOpen: boolean; children: React.ReactNode; title: string }) => {
    if (!isOpen) return null;
    return (
 
      <div data-testid="modal">
        <h2>{title}</h2>
        {children}
      </div>
    );
  },
}));

describe("AdjustmentRequestModal", () => {
  const mockOnClose = vi.fn();
  const mockOnSubmit = vi.fn();

  const defaultProps = {
    isOpen: true,
    onClose: mockOnClose,
    onSubmit: mockOnSubmit,
    timezone: "UTC",
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockOnSubmit.mockResolvedValue(undefined);
  });

  it("renders modal with form fields when open", () => {
    render(<AdjustmentRequestModal {...defaultProps} />);

    expect(screen.getByTestId("modal")).toBeInTheDocument();
    expect(screen.getByText("Request Time Adjustment")).toBeInTheDocument();
    expect(screen.getByText("Work Day")).toBeInTheDocument();
    expect(screen.getByText("Punch In")).toBeInTheDocument();
    expect(screen.getByText("Punch Out")).toBeInTheDocument();
    expect(screen.getByText("Reason")).toBeInTheDocument();
  });

  it("does not render when isOpen is false", () => {
    render(<AdjustmentRequestModal {...defaultProps} isOpen={false} />);

    expect(screen.queryByTestId("modal")).not.toBeInTheDocument();
  });

  it("calls onClose when cancel button clicked", async () => {
    const user = userEvent.setup();
    render(<AdjustmentRequestModal {...defaultProps} />);

    const cancelButton = screen.getByText("Cancel");
    await user.click(cancelButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it("renders with edit title when timeEntry provided", () => {
    const mockEntry: TimeEntry = { 
      id: "entry-1", 
      employee_id: "emp-1",
      punch_in_time: "2024-01-01T08:00:00Z", 
      punch_out_time: "2024-01-01T17:00:00Z", 
      entry_type: "work", 
      is_break_overtime: false,
      created_at: "2024-01-01T08:00:00Z",
      updated_at: "2024-01-01T17:00:00Z"
    };

    render(<AdjustmentRequestModal {...defaultProps} timeEntry={mockEntry} />);

    expect(screen.getByText("Edit Time Adjustment")).toBeInTheDocument();
  });

  it("submits form with correct data", async () => {
    const user = userEvent.setup();
    render(<AdjustmentRequestModal {...defaultProps} />);

    const reasonTextarea = screen.getByPlaceholderText("Please explain why you need this adjustment");
    await user.type(reasonTextarea, "I forgot to clock out");

    const submitButton = screen.getByText("Submit Request");
    await user.click(submitButton);

    await waitFor(() => {
      expect(mockOnSubmit).toHaveBeenCalledWith(
        expect.objectContaining({
          reason: "I forgot to clock out",
          isBreakOvertime: false,
        })
      );
    });
  });

  it("closes modal after successful submission", async () => {
    const user = userEvent.setup();
    render(<AdjustmentRequestModal {...defaultProps} />);

    const reasonTextarea = screen.getByPlaceholderText("Please explain why you need this adjustment");
    await user.type(reasonTextarea, "Test reason");

    const submitButton = screen.getByText("Submit Request");
    await user.click(submitButton);

    await waitFor(() => {
      expect(mockOnClose).toHaveBeenCalled();
    });
  });
});
