import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AttendanceAdjustmentsPage from "../AttendanceAdjustmentsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "../../lib/api";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
  approveAttendanceAdjustment: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchEmployees: vi.fn(),
  uploadAttendance: vi.fn(),
  fetchLatestImportJob: vi.fn(),
  fetchImportJobStatus: vi.fn(),
}));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
});

describe("AttendanceAdjustmentsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    queryClient.clear();
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe" }
    ]);
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);
    (api.fetchLatestImportJob as any).mockResolvedValue(null);
  });

  it("renders without workspace selected", () => {
    (api.getCurrentWorkspaceId as any).mockReturnValue(null);
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(screen.getByText("Chọn Workspace")).toBeDefined();
  });

  it("renders list of adjustment requests", async () => {
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_punch_in_time: "2026-01-01T08:00:00Z",
        requested_punch_out_time: "2026-01-01T17:00:00Z",
        reason: "Missed punch",
        status: "pending"
      }
    ]);

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(await screen.findByText("John Doe")).toBeDefined();
    expect(screen.getByText("Missed punch")).toBeDefined();
    expect(screen.getByText("Chờ duyệt")).toBeDefined();
  });

  it("handles approve action", async () => {
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_punch_in_time: "2026-01-01T08:00:00Z",
        status: "pending"
      }
    ]);
    (api.approveAttendanceAdjustment as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    const approveBtn = await screen.findByTitle("Phê duyệt");
    fireEvent.click(approveBtn);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith("test-ws-id", "adj-1", { status: "approved" });
    });
  });

  it("handles reject action", async () => {
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_punch_in_time: "2026-01-01T08:00:00Z",
        status: "pending"
      }
    ]);
    (api.approveAttendanceAdjustment as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    const rejectBtn = await screen.findByTitle("Từ chối");
    fireEvent.click(rejectBtn);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith("test-ws-id", "adj-1", { status: "rejected" });
    });
  });

  it("shows processing state for import job in dialog", async () => {
    // Mock latest job to be processing so dialog shows up or has state
    (api.fetchLatestImportJob as any).mockResolvedValue({
      id: "job-1",
      status: "processing",
      processed_rows: 50,
      total_rows: 100
    });
    (api.fetchImportJobStatus as any).mockResolvedValue({
      id: "job-1",
      status: "processing",
      processed_rows: 50,
      total_rows: 100
    });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Open dialog
    const importBtn = screen.getByText("Nhập Excel");
    fireEvent.click(importBtn);

    expect(api.fetchLatestImportJob).toHaveBeenCalledWith("test-ws-id", "adjustment");
    expect(await screen.findByText("Đang xử lý dữ liệu...")).toBeDefined();
    expect(screen.getByText("50 / 100 dòng đã xử lý")).toBeDefined();
  });

  it("shows completed state for import job in dialog", async () => {
    (api.fetchLatestImportJob as any).mockResolvedValue({
      id: "job-1",
      status: "completed",
      processed_rows: 100,
      total_rows: 100
    });
    (api.fetchImportJobStatus as any).mockResolvedValue({
      id: "job-1",
      status: "completed",
      processed_rows: 100,
      total_rows: 100
    });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Open dialog
    const importBtn = screen.getByText("Nhập Excel");
    fireEvent.click(importBtn);

    expect(await screen.findByText("Hoàn thành!")).toBeDefined();
    expect(screen.getByText("100 / 100 dòng đã xử lý")).toBeDefined();
  });

  it("shows failed state for import job in dialog", async () => {
    (api.fetchLatestImportJob as any).mockResolvedValue({
      id: "job-1",
      status: "failed",
      error_log: "Invalid column format",
      processed_rows: 0,
      total_rows: 100
    });
    (api.fetchImportJobStatus as any).mockResolvedValue({
      id: "job-1",
      status: "failed",
      error_log: "Invalid column format",
      processed_rows: 0,
      total_rows: 100
    });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Open dialog
    const importBtn = screen.getByText("Nhập Excel");
    fireEvent.click(importBtn);

    expect(await screen.findByText("Có lỗi xảy ra")).toBeDefined();
    expect(screen.getByText("Invalid column format")).toBeDefined();
  });

  it("handles file upload in dialog", async () => {
    (api.uploadAttendance as any).mockResolvedValue({ job_id: "new-job" });
    
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Open dialog
    const importBtn = screen.getByText("Nhập Excel");
    fireEvent.click(importBtn);

    const file = new File(["dummy content"], "test.csv", { type: "text/csv" });
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    
    // Simulate file selection
    Object.defineProperty(input, 'files', {
        value: [file]
    });
    fireEvent.change(input);

    // Click upload button
    const uploadBtn = screen.getByText("Bắt đầu nhập dữ liệu");
    fireEvent.click(uploadBtn);

    await waitFor(() => {
      expect(api.uploadAttendance).toHaveBeenCalledWith("test-ws-id", file, "/attendance/upload");
    });
  });

  it("handles download template", () => {
    // Mock URL methods
    const mockCreateObjectURL = vi.fn(() => "blob:test");
    const mockRevokeObjectURL = vi.fn();
    global.URL.createObjectURL = mockCreateObjectURL;
    global.URL.revokeObjectURL = mockRevokeObjectURL;

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    const downloadBtn = screen.getByText("Mẫu Excel");
    fireEvent.click(downloadBtn);

    expect(mockCreateObjectURL).toHaveBeenCalled();
  });
});