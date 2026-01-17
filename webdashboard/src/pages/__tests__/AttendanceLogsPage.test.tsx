import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AttendanceLogsPage from "../AttendanceLogsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";

// Mock EmployeeSearch
vi.mock("@/components/common/EmployeeSearch", () => ({
  EmployeeSearch: ({ value, onChange, placeholder }: any) => (
    <select 
      data-testid="employee-search-mock" 
      value={value === "all" ? "" : value} 
      onChange={(e) => onChange(e.target.value || "all")}
    >
      <option value="">{placeholder || "All"}</option>
      <option value="emp-1">John Doe</option>
    </select>
  )
}));

// Mock AttendanceImportDialog
vi.mock("@/components/AttendanceImportDialog", () => ({
  default: () => <div data-testid="import-dialog-mock" />
}));

// Mock API
vi.mock("@/lib/api", () => ({
  fetchRawAttendanceLogs: vi.fn(),
  fetchEmployees: vi.fn(),
  fetchWorkspace: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "ws-1"),
}));

import * as api from "@/lib/api";

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, gcTime: 0 } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>
    <ToastProvider>
      <BrowserRouter>{children}</BrowserRouter>
    </ToastProvider>
  </QueryClientProvider>
);

describe("AttendanceLogsPage", () => {
  beforeEach(() => {
    queryClient.clear();
    vi.clearAllMocks();
    (api.fetchEmployees as any).mockResolvedValue([{ id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" }]);
    (api.fetchWorkspace as any).mockResolvedValue({ timezone: "UTC" });
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({
      data: [{ id: "1", employee_id: "emp-1", punch_time: new Date().toISOString(), source: "device" }],
      total: 1
    });
  });

  it("renders the page and employee search", async () => {
    render(<AttendanceLogsPage />, { wrapper });
    expect(screen.getByText(/Nhật ký quẹt thẻ/i)).toBeDefined();
    expect(screen.getByTestId("employee-search-mock")).toBeDefined();
    
    await waitFor(() => {
      expect(screen.getByText("EMP001")).toBeDefined();
    });
  });

  it("handles employee filtering", async () => {
    render(<AttendanceLogsPage />, { wrapper });
    
    const select = screen.getByTestId("employee-search-mock");
    fireEvent.change(select, { target: { value: "emp-1" } });

    await waitFor(() => {
      expect(api.fetchRawAttendanceLogs).toHaveBeenCalledWith(expect.objectContaining({
        employee_id: "emp-1"
      }));
    });
  });
});
