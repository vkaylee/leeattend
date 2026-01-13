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
  uploadAttendanceExcel: vi.fn(),
  fetchLatestImportJob: vi.fn(),
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

    expect(screen.getByText("Vui lòng chọn Workspace")).toBeDefined();
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

  it("shows processing state for import job", async () => {
    (api.fetchLatestImportJob as any).mockResolvedValue({
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

    expect(await screen.findByText("Đang xử lý file chấm công...")).toBeDefined();
    expect(screen.getByText("Đã xử lý 50 / 100 dòng")).toBeDefined();
  });

  it("shows completed state for import job", async () => {
    (api.fetchLatestImportJob as any).mockResolvedValue({
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

    expect(await screen.findByText("File tải lên gần nhất đã hoàn tất (100 dòng).")).toBeDefined();
  });

  it("shows failed state for import job", async () => {
    (api.fetchLatestImportJob as any).mockResolvedValue({
      id: "job-1",
      status: "failed",
      error_log: "Invalid column format\nSome details"
    });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(await screen.findByText("Lỗi xử lý file:")).toBeDefined();
    expect(screen.getByText("Invalid column format")).toBeDefined();
  });

  it("handles file upload", async () => {
    (api.uploadAttendanceExcel as any).mockResolvedValue({});
    
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    const file = new File(["dummy content"], "test.csv", { type: "text/csv" });
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    
    fireEvent.change(input, { target: { files: [file] } });

    await waitFor(() => {
      expect(api.uploadAttendanceExcel).toHaveBeenCalledWith("test-ws-id", file);
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