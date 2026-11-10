import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../../lib/api";
import { ToastProvider } from "../../ui/toast";
import AdjustmentDrawer from "../adjustment-drawer";



// Mock API
vi.mock("@/lib/api", () => ({
  fetchOrganization: vi.fn().mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC", default_language: "en" }),
  fetchAttendanceAdjustments: vi.fn(),
  approveAttendanceAdjustment: vi.fn(),
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
 
    vi.mocked(api.fetchAttendanceAdjustments).mockResolvedValue({
      data: [
        {
          id: "adj-1",
          employee_id: "emp-1",
          employment_id: "employment-1",
          requested_punch_time: "2026-01-10T08:00:00Z",
          requested_punch_in_time: "2026-01-10T08:00:00Z",
          requested_punch_out_time: "2026-01-10T17:00:00Z",
          reason: "Forgot to punch",
          status: "pending"
        }
      ],
      total: 1,
      page: 1,
      limit: 20
    } as any);

    renderWithProviders(
      <AdjustmentDrawer 
        employeeId="emp-1" 
        employeeName="John Doe"
        employmentId="employment-1" 
        isOpen={true} 
        onClose={() => { /* empty */ }} 
      />
    );

    expect(await screen.findByText(/John Doe/i)).toBeDefined();
    expect(await screen.findByTestId("adjustment-reason")).toBeDefined();
    expect(screen.getByText(/Forgot to punch/i)).toBeDefined();
  });

  it("calls approve API when Approve is clicked", async () => {
 
    vi.mocked(api.fetchAttendanceAdjustments).mockResolvedValue({
      data: [
        {
          id: "adj-1",
          employee_id: "emp-1",
          employment_id: "employment-1",
          requested_punch_time: "2026-01-10T08:00:00Z",
          requested_punch_in_time: "2026-01-10T08:00:00Z",
          status: "pending"
        }
      ],
      total: 1,
      page: 1,
      limit: 20
    } as any);
 
    vi.mocked(api.approveAttendanceAdjustment).mockResolvedValue({} as unknown as never);

    renderWithProviders(
      <AdjustmentDrawer 
        employeeId="emp-1" 
        employeeName="John Doe"
        employmentId="employment-1" 
        isOpen={true} 
        onClose={() => { /* empty */ }} 
      />
    );

    const approveButton = await screen.findByTestId("approve-adjustment-btn");
    fireEvent.click(approveButton);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith("adj-1", { status: "approved" });
    });
  });
});