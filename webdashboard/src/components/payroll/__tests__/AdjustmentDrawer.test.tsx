import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AdjustmentDrawer from "../AdjustmentDrawer";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../Toast";
import * as api from "@/lib/api";

// Mock API
vi.mock("@/lib/api", async (importOriginal) => {
  const actual: any = await importOriginal();
  return {
    ...actual,
    fetchWorkspaceAttendanceAdjustments: vi.fn(),
    approveAttendanceAdjustment: vi.fn(),
    getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
  };
});

describe("AdjustmentDrawer", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
  });

  it("renders pending adjustments for the specific employee", async () => {
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_punch_in_time: "2026-01-10T08:00:00Z",
        requested_punch_out_time: "2026-01-10T17:00:00Z",
        reason: "Forgot to punch",
        status: "pending"
      },
      {
        id: "adj-2",
        employee_id: "other-emp",
        requested_punch_in_time: "2026-01-10T08:00:00Z",
        status: "pending"
      }
    ]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <AdjustmentDrawer 
            isOpen={true} 
            onClose={() => {}} 
            employeeId="emp-1" 
            employeeName="John Doe" 
          />
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(await screen.findByText(/John Doe/)).toBeDefined();
    expect(await screen.findByTestId("adjustment-reason")).toBeDefined();
    expect(screen.getByText(/Forgot to punch/)).toBeDefined();
    // Should NOT show adj-2
    expect(screen.queryByText("adj-2")).toBeNull();
  });

  it("calls approve API when Approve is clicked", async () => {
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_punch_in_time: "2026-01-10T08:00:00Z",
        status: "pending"
      }
    ]);
    (api.approveAttendanceAdjustment as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <AdjustmentDrawer 
            isOpen={true} 
            onClose={() => {}} 
            employeeId="emp-1" 
            employeeName="John Doe" 
          />
        </ToastProvider>
      </QueryClientProvider>
    );

    const approveBtn = await screen.findByTestId("approve-adjustment-btn");
    fireEvent.click(approveBtn);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith("test-ws-id", "adj-1", { status: "approved" });
    });
  });
});
