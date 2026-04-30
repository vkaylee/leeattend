import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api";

import { ToastProvider } from "../../ui/Toast";
import AdjustmentDrawer from "../AdjustmentDrawer";



// Mock API
vi.mock("@/lib/api", () => ({
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
  approveAttendanceAdjustment: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
}));

describe("AdjustmentDrawer", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders pending adjustments for the specific employee", async () => {
 
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_punch_in_time: "2026-01-10T08:00:00Z",
        requested_punch_out_time: "2026-01-10T17:00:00Z",
        reason: "Forgot to punch",
        status: "pending"
      }
    ] as unknown as never);

    renderWithProviders(
      <AdjustmentDrawer 
        isOpen={true} 
        onClose={() => {}} 
        employeeId="emp-1" 
        employeeName="John Doe" 
      />
    );

    expect(await screen.findByText(/John Doe/i)).toBeDefined();
    expect(await screen.findByTestId("adjustment-reason")).toBeDefined();
    expect(screen.getByText(/Forgot to punch/i)).toBeDefined();
  });

  it("calls approve API when Approve is clicked", async () => {
 
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_punch_in_time: "2026-01-10T08:00:00Z",
        status: "pending"
      }
    ] as unknown as never);
 
    vi.mocked(api.approveAttendanceAdjustment).mockResolvedValue({} as unknown as never);

    renderWithProviders(
      <AdjustmentDrawer 
        isOpen={true} 
        onClose={() => {}} 
        employeeId="emp-1" 
        employeeName="John Doe" 
      />
    );

    const approveBtn = await screen.findByTestId("approve-adjustment-btn");
    fireEvent.click(approveBtn);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith("test-ws-id", "adj-1", { status: "approved" });
    });
  });
});