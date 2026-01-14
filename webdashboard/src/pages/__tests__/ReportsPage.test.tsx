import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import ReportsPage from "../ReportsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchAttendanceReport: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
}));

describe("ReportsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
        },
      },
    });
  });

  it("renders report table and default to current month", async () => {
    (api.fetchAttendanceReport as any).mockResolvedValue([
      { employee_id: "emp-1", employee_name: "John Doe", total_hours: 160, total_leave_days: 0 }
    ]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <ReportsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(await screen.findByText("John Doe")).toBeDefined();
    const totalHoursElements = screen.getAllByText("160.0h");
    expect(totalHoursElements.length).toBeGreaterThan(0);
    
    // Default should be "Tháng này"
    const thisMonthBtn = screen.getByText("Tháng này");
    expect(thisMonthBtn.className).toContain("text-primary-600");
  });

  it("changes date range when clicking 'Tháng trước'", async () => {
    const user = userEvent.setup();
    (api.fetchAttendanceReport as any).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <ReportsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    const lastMonthBtn = screen.getByText("Tháng trước");
    await user.click(lastMonthBtn);

    // Verify button state
    expect(lastMonthBtn.className).toContain("text-primary-600");

    const now = new Date();
    const lastMonth = new Date(now.getFullYear(), now.getMonth() - 1, 1);
    const expectedStart = lastMonth.toISOString().split('T')[0];

    await waitFor(() => {
      expect(api.fetchAttendanceReport).toHaveBeenCalledWith(
        "test-ws-id",
        expectedStart,
        expect.any(String),
        "summary"
      );
    });
  });

  it("shows custom date inputs when clicking 'Tùy chỉnh'", async () => {
    const user = userEvent.setup();
    (api.fetchAttendanceReport as any).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <ReportsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Should not see date inputs initially
    expect(screen.queryByLabelText(/Từ ngày/i)).toBeNull();

    const customBtn = screen.getByText("Tùy chỉnh");
    await user.click(customBtn);

    // Should see date inputs now
    expect(screen.getByLabelText(/Từ ngày/i)).toBeDefined();
    expect(screen.getByLabelText(/Đến ngày/i)).toBeDefined();
  });
});
