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

    expect(await screen.findByText("John Doe")).toBeDefined();
    expect(screen.getByText("EMP001")).toBeDefined();
    expect(screen.getByText("import")).toBeDefined();
    expect(screen.getByText("Device-A")).toBeDefined();
  });

  it("handles filtering", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" }
    ]);
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

    // Wait for employee options to load
    await waitFor(async () => {
      const option = await screen.queryByText(/John Doe/);
      expect(option).not.toBeNull();
    });

    // Filter by employee
    const empFilter = screen.getByLabelText("Nhân viên");
    await user.selectOptions(empFilter, "emp-1");

    // Filter by date
    const dateInput = screen.getByLabelText("Từ ngày");
    fireEvent.change(dateInput, { target: { value: "2026-01-01" } });

    await waitFor(() => {
      expect(api.fetchRawAttendanceLogs).toHaveBeenCalledWith(expect.objectContaining({
        employee_id: "emp-1",
      }));
    });
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
