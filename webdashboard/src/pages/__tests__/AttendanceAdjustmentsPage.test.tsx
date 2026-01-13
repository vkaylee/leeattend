import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import AttendanceAdjustmentsPage from "../AttendanceAdjustmentsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchWorkspaceAttendanceAdjustments: vi.fn(() => Promise.resolve([])),
  approveAttendanceAdjustment: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
  fetchEmployees: vi.fn(() => Promise.resolve([])),
  uploadAttendanceExcel: vi.fn(),
  fetchLatestImportJob: vi.fn(() => Promise.resolve(null)),
}));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
});

describe("AttendanceAdjustmentsPage", () => {
  it("should have correct file upload accept types including .xls", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Tìm input file ẩn
    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    
    expect(fileInput).toBeDefined();
    // Kiểm tra xem có chứa .xls không
    expect(fileInput.getAttribute("accept")).toContain(".xls");
    expect(fileInput.getAttribute("accept")).toContain(".xlsx");
    expect(fileInput.getAttribute("accept")).toContain(".csv");
  });

  it("renders the action buttons", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AttendanceAdjustmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(screen.getByText("Nhập Excel")).toBeDefined();
    expect(screen.getByText("Mẫu Excel")).toBeDefined();
  });
});
