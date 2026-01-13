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
  fetchAttendanceLogs: vi.fn(),
  fetchEmployees: vi.fn(),
  updateTimeEntry: vi.fn(),
  deleteTimeEntry: vi.fn(),
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
    
    // Mock window.confirm
    vi.stubGlobal("confirm", vi.fn(() => true));
  });

  it("renders attendance logs table", async () => {
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe" }
    ]);
    (api.fetchAttendanceLogs as any).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", punch_in_time: "2026-01-01T08:00:00Z", punch_out_time: "2026-01-01T17:00:00Z", entry_type: "work", duration_minutes: 540 }
      ],
      total: 1,
      page: 1,
      limit: 20
    });

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
    expect(screen.getByText("9h 0m")).toBeDefined();
  });

  it("handles filtering", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe" }
    ]);
    (api.fetchAttendanceLogs as any).mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });

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
      expect(api.fetchAttendanceLogs).toHaveBeenCalledWith(expect.objectContaining({
        employee_id: "emp-1",
      }));
    });
  });

  it("opens edit modal and updates log", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe" }
    ]);
    (api.fetchAttendanceLogs as any).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", punch_in_time: "2026-01-01T08:00:00Z", punch_out_time: "2026-01-01T17:00:00Z", entry_type: "work", duration_minutes: 540 }
      ],
      total: 1, page: 1, limit: 20
    });
    (api.updateTimeEntry as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    await screen.findByText("John Doe");

    const editBtn = screen.getByTitle("Sửa bản ghi");
    await user.click(editBtn);

    expect(await screen.findByText("Sửa nhật ký chấm công")).toBeDefined();
    
    const saveBtn = screen.getByText("Lưu thay đổi");
    await user.click(saveBtn);

    await waitFor(() => {
      expect(api.updateTimeEntry).toHaveBeenCalled();
    });
  });

  it("handles log deletion", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([{ id: "emp-1", first_name: "John", last_name: "Doe" }]);
    (api.fetchAttendanceLogs as any).mockResolvedValue({
      data: [{ id: "log-1", employee_id: "emp-1", punch_in_time: "2026-01-01T08:00:00Z", punch_out_time: "2026-01-01T17:00:00Z", entry_type: "work", duration_minutes: 540 }],
      total: 1, page: 1, limit: 20
    });
    (api.deleteTimeEntry as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    await screen.findByText("John Doe");

    const deleteBtn = screen.getByTitle("Xóa bản ghi");
    await user.click(deleteBtn);

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(api.deleteTimeEntry).toHaveBeenCalledWith("test-ws-id", "log-1");
    });
  });

  it("handles pagination", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([]);
    (api.fetchAttendanceLogs as any).mockResolvedValue({
      data: [], total: 50, page: 1, limit: 20
    });

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
      expect(api.fetchAttendanceLogs).toHaveBeenCalledWith(expect.objectContaining({
        page: 2
      }));
    });

    const prevBtn = screen.getByText("Trước");
    await user.click(prevBtn);

    await waitFor(() => {
      expect(api.fetchAttendanceLogs).toHaveBeenCalledWith(expect.objectContaining({
        page: 1
      }));
    });
  });
});