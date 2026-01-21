import { render, screen, fireEvent, within, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import PayrollPage from "../PayrollPage";
import TimeTrackingPage from "../TimeTrackingPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock useDebounce to be immediate
vi.mock("@/hooks/use-debounce", () => ({
  useDebounce: (value: any) => value,
}));

// Mock EmployeeSearch to avoid internal queries
vi.mock("@/components/common/EmployeeSearch", () => ({
  EmployeeSearch: ({ value, onChange }: any) => (
    <select 
      data-testid="employee-select" 
      value={value} 
      onChange={(e) => onChange(e.target.value)}
    >
      <option value="">Select...</option>
      <option value="emp-1">John Doe</option>
    </select>
  )
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  searchEmployees: vi.fn(),
  fetchEmployeePayslips: vi.fn(),
  fetchTimeHistory: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
  fetchWorkspace: vi.fn(),
  submitAttendanceAdjustment: vi.fn(),
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
}));

describe("Date Format Integration", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            {ui}
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  const selectEmployee = async () => {
    // EmployeeSearch is a mocked select with testId
    const select = await screen.findByTestId("employee-select");
    fireEvent.change(select, { target: { value: 'emp-1' } });
  };

  it("renders dates in DD/MM/YYYY format when workspace setting is DD/MM/YYYY", async () => {
    (api.fetchEmployees as any).mockResolvedValue([{ id: "emp-1", first_name: "John", last_name: "Doe" }]);
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "DD/MM/YYYY" });
    (api.fetchTimeHistory as any).mockResolvedValue([
      { id: "th-1", employee_id: "emp-1", punch_in_time: "2026-01-15T08:00:00Z", punch_out_time: "2026-01-15T17:00:00Z", entry_type: 'work' }
    ]);

    renderWithProviders(<TimeTrackingPage />);
    await selectEmployee();
    
    expect(await screen.findByText(/15\/01\/2026/)).toBeInTheDocument();
  });

  it("renders dates in YYYY-MM-DD format when workspace setting is YYYY-MM-DD", async () => {
    (api.fetchEmployees as any).mockResolvedValue([{ id: "emp-1", first_name: "John", last_name: "Doe" }]);
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "YYYY-MM-DD" });
    (api.fetchTimeHistory as any).mockResolvedValue([
      { id: "th-1", employee_id: "emp-1", punch_in_time: "2026-01-15T08:00:00Z", punch_out_time: "2026-01-15T17:00:00Z", entry_type: 'work' }
    ]);

    renderWithProviders(<TimeTrackingPage />);
    await selectEmployee();
    
    expect(await screen.findByText(/2026-01-15/)).toBeInTheDocument();
  });

  it("renders month/year correctly in PayrollPage", async () => {
    (api.fetchEmployees as any).mockResolvedValue([{ id: "emp-1", first_name: "John", last_name: "Doe", hire_date: "2025-01-01" }]);
    (api.fetchEmployeePayslips as any).mockResolvedValue([]);
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

    renderWithProviders(<PayrollPage />);
    
    // Select employee from the sidebar directly
    const empBtns = await screen.findAllByText("John Doe");
    fireEvent.click(empBtns[0]);
    
    // Check for the month header in the details view
    expect(await screen.findByText(/Tháng/i)).toBeDefined();
  });
});