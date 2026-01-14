import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AttendanceLogsPage from "../AttendanceLogsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchRawAttendanceLogs: vi.fn(),
  fetchEmployees: vi.fn(),
  fetchWorkspace: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
}));

// Mock useDateTimeFormatter hook
vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: vi.fn(() => ({
    formatDate: vi.fn((d) => d),
    formatTime: vi.fn((d) => d),
  })),
}));

describe("AttendanceLogsPage", () => {
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

  it("renders raw attendance logs table", async () => {
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" }
    ]);
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", punch_time: "2026-01-01T08:00:00Z", source: "import", device_id: "Device-A" }
      ],
      total: 1,
      page: 1,
      limit: 20
    });
    (api.fetchWorkspace as any).mockResolvedValue({ time_format: "24h" });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect((await screen.findAllByText("John Doe")).length).toBeGreaterThan(0);
    expect(screen.getByText("EMP001")).toBeDefined();
    expect(screen.getByText("import")).toBeDefined();
    expect(screen.getByText("Device-A")).toBeDefined();
  });

  it("handles month filtering", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([]);
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });
    (api.fetchWorkspace as any).mockResolvedValue({ time_format: "24h" });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Initial fetch should be for this month
    await waitFor(() => {
      expect(api.fetchRawAttendanceLogs).toHaveBeenCalled();
    });

    // Click "Tháng trước"
    const lastMonthBtn = screen.getByText("Tháng trước");
    await user.click(lastMonthBtn);

    await waitFor(() => {
      // It should trigger another fetch
      expect(api.fetchRawAttendanceLogs).toHaveBeenCalledTimes(2);
    });
  });

  it("handles custom date range visibility", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([]);
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });
    (api.fetchWorkspace as any).mockResolvedValue({ time_format: "24h" });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Should not see date inputs initially
    expect(screen.queryByLabelText(/Từ ngày/i)).toBeNull();

    // Click "Tùy chỉnh"
    const customBtn = screen.getByText("Tùy chỉnh");
    await user.click(customBtn);

    // Now should see inputs
    expect(screen.getByLabelText(/Từ ngày/i)).toBeDefined();
  });

  it("handles pagination", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([]);
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({
      data: [], total: 50, page: 1, limit: 20
    });
    (api.fetchWorkspace as any).mockResolvedValue({ time_format: "24h" });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    const nextBtn = await screen.findByText("Sau");
    await user.click(nextBtn);

    await waitFor(() => {
      expect(api.fetchRawAttendanceLogs).toHaveBeenCalledWith(expect.objectContaining({
        page: 2
      }));
    });

    const prevBtn = screen.getByText("Trước");
    await user.click(prevBtn);

    await waitFor(() => {
      expect(api.fetchRawAttendanceLogs).toHaveBeenCalledWith(expect.objectContaining({
        page: 1
      }));
    });
  });
});