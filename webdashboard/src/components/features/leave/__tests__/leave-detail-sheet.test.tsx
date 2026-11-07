import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { LeaveDetailSheet } from "../leave-detail-sheet";
import type { Employee, LeaveRequest, LeaveType } from "@/lib/api/types";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, defaultVal?: unknown) => {
      if (typeof defaultVal === "string") return defaultVal;
      return key;
    },
  }),
}));

describe("LeaveDetailSheet Component", () => {
  const onClose = vi.fn();
  const onApprove = vi.fn();

  const mockEmployee: Employee = {
    id: "emp-1",
    employee_id_number: "EMP001",
    full_name: "Alice Smith",
    department_name: "Engineering",
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
    workspace_id: "ws-1",
    tags: [],
  };

  const mockLeaveTypes: LeaveType[] = [
    {
      id: "lt-annual",
      name: "Annual Leave",
      days_allowed: 12,
      is_paid: true,
      requires_approval: true,
    },
  ];

  const mockRequest: LeaveRequest = {
    id: "req-123",
    employment_id: "emp-1",
    leave_type_id: "lt-annual",
    start_date: "2026-11-10",
    end_date: "2026-11-12",
    reason: "Family vacation",
    status: "pending",
    duration_type: "full_day",
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not render when request is null", () => {
    const { container } = render(
      <LeaveDetailSheet
        formatDate={(d) => d}
        isOpen={false}
        leaveTypes={mockLeaveTypes}
        onClose={onClose}
        request={null}
      />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders leave request details with employee info, dates, and reason", () => {
    render(
      <LeaveDetailSheet
        emp={mockEmployee}
        formatDate={(d) => d}
        isOpen={true}
        leaveTypes={mockLeaveTypes}
        onApprove={onApprove}
        onClose={onClose}
        request={mockRequest}
      />
    );

    expect(screen.getByText("Alice Smith")).toBeInTheDocument();
    expect(screen.getByText("Mã: EMP001")).toBeInTheDocument();
    expect(screen.getByText("Annual Leave")).toBeInTheDocument();
    expect(screen.getByText("Family vacation")).toBeInTheDocument();
  });

  it("calls onApprove with approved status when approve button clicked", () => {
    render(
      <LeaveDetailSheet
        emp={mockEmployee}
        formatDate={(d) => d}
        isOpen={true}
        leaveTypes={mockLeaveTypes}
        onApprove={onApprove}
        onClose={onClose}
        request={mockRequest}
      />
    );

    const approveBtn = screen.getByTestId("leave-sheet-approve-btn");
    fireEvent.click(approveBtn);

    expect(onApprove).toHaveBeenCalledWith("req-123", "approved");
    expect(onClose).toHaveBeenCalled();
  });

  it("calls onApprove with rejected status when reject button clicked", () => {
    render(
      <LeaveDetailSheet
        emp={mockEmployee}
        formatDate={(d) => d}
        isOpen={true}
        leaveTypes={mockLeaveTypes}
        onApprove={onApprove}
        onClose={onClose}
        request={mockRequest}
      />
    );

    const rejectBtn = screen.getByTestId("leave-sheet-reject-btn");
    fireEvent.click(rejectBtn);

    expect(onApprove).toHaveBeenCalledWith("req-123", "rejected");
    expect(onClose).toHaveBeenCalled();
  });
});
