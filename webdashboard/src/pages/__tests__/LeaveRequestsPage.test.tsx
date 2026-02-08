import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import LeaveRequestsPage from "../LeaveRequestsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "../../lib/api";
import { ToastProvider } from "../../components/ui/Toast";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  submitLeaveRequest: vi.fn(),
  fetchWorkspaceLeaveRequests: vi.fn(),
  approveLeaveRequest: vi.fn(),
  fetchEmployeeLeaveBalances: vi.fn(),
  fetchLeavePolicies: vi.fn(),
  fetchLeaveTypes: vi.fn(),
  createLeavePolicy: vi.fn(),
  updateLeavePolicy: vi.fn(),
  deleteLeavePolicy: vi.fn(),
  assignLeavePolicy: vi.fn(),
  triggerAccrualRun: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

// Mock EmployeeSearch
vi.mock("@/components/common/EmployeeSearch", () => ({
  EmployeeSearch: ({ value, onChange }: any) => (
    <select 
      aria-label="Nhân viên"
      value={value} 
      onChange={(e) => onChange(e.target.value)}
    >
      <option value="">Chọn...</option>
      <option value="emp-1">John Doe</option>
    </select>
  )
}));

describe("LeaveRequestsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchEmployees as any).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", hire_date: "2025-01-01", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }
      ],
      total: 1,
      page: 1,
      limit: 50,
    });
    (api.fetchWorkspaceLeaveRequests as any).mockResolvedValue([]);
    (api.fetchEmployeeLeaveBalances as any).mockResolvedValue([]);
    (api.fetchLeavePolicies as any).mockResolvedValue([]);
    (api.fetchLeaveTypes as any).mockResolvedValue([
      { id: "lt-vacation", code: "lt-vacation", name: "Vacation", is_paid: true, workspace_id: "ws-1", policy_config: {}, created_at: "2025-01-01T00:00:00Z" }
    ]);
  });

  it("renders without workspace selected", () => {
    (api.getCurrentWorkspaceId as any).mockReturnValue(null);
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <LeaveRequestsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(screen.getByText("Chọn Workspace")).toBeDefined();
  });

  it("renders pending requests by default", async () => {
    (api.fetchWorkspaceLeaveRequests as any).mockResolvedValue([
      { id: "req-1", employee_id: "emp-1", leave_type_id: "lt-vacation", start_date: "2026-01-01", end_date: "2026-01-02", status: "pending", reason: "Test reason" }
    ]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <LeaveRequestsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(await screen.findByText("John Doe")).toBeDefined();
    expect(screen.getByText(/vacation/i)).toBeDefined();
    expect(screen.getAllByText(/Chờ duyệt/i).length).toBeGreaterThan(0);
  });

  it("handles submitting a new leave request", async () => {
    const user = userEvent.setup();
    (api.submitLeaveRequest as any).mockResolvedValue({});
    (api.fetchEmployeeLeaveBalances as any).mockImplementation((wsId: string, empId: string) => {
      if (empId === "emp-1") {
        return Promise.resolve([
          { leave_type_id: "lt-1", leave_type_code: "lt-vacation", leave_type_name: "Vacation", is_paid: true, remaining: 10, entitlement: 12, carried_over: 2, accrued_amount: 0, used: 2 }
        ]);
      }
      return Promise.resolve([]);
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <LeaveRequestsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    await user.click(screen.getByText(/Tạo yêu cầu/i));

    // Select employee
    const empSelect = await screen.findByLabelText(/Nhân viên/i);
    fireEvent.change(empSelect, { target: { value: "emp-1" } });

    // Wait for the balances to appear.
    expect(await screen.findByText(/Còn/i)).toBeDefined();
    expect(screen.getByText("10")).toBeDefined();

    await user.selectOptions(screen.getByLabelText(/Loại/i), "lt-vacation");
    
    // Fill dates
    const startDate = screen.getByLabelText(/Từ ngày/i);
    fireEvent.change(startDate, { target: { value: "2026-01-01" } });
    const endDate = screen.getByLabelText(/Đến ngày/i);
    fireEvent.change(endDate, { target: { value: "2026-01-02" } });
    
    const reasonInput = screen.getByPlaceholderText(/Nhập lý do nghỉ/i);
    await user.type(reasonInput, "Vacation time");

    await user.click(screen.getByRole("button", { name: /Gửi yêu cầu/i }));

    await waitFor(() => {
      expect(api.submitLeaveRequest).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        employee_id: "emp-1",
        leave_type_id: "lt-vacation",
        reason: "Vacation time"
      }));
    }, { timeout: 20000 });
  }, 30000);

  it("handles approve/reject actions", async () => {
    const user = userEvent.setup();
    (api.fetchWorkspaceLeaveRequests as any).mockResolvedValue([
      { id: "req-1", employee_id: "emp-1", leave_type_id: "lt-vacation", start_date: "2026-01-01", end_date: "2026-01-02", status: "pending" }
    ]);
    (api.approveLeaveRequest as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <LeaveRequestsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    const approveBtn = await screen.findByTitle(/phê duyệt/i);
    await user.click(approveBtn);

    await waitFor(() => {
      expect(api.approveLeaveRequest).toHaveBeenCalledWith("test-ws-id", "req-1", { status: "approved" });
    });

    const rejectBtn = await screen.findByTitle(/từ chối/i);
    await user.click(rejectBtn);

    await waitFor(() => {
      expect(api.approveLeaveRequest).toHaveBeenCalledWith("test-ws-id", "req-1", { status: "rejected" });
    });
  });





});