import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import LeaveRequestsPage from "../LeaveRequestsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "../../lib/api";
import { ToastProvider } from "../../components/Toast";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  submitLeaveRequest: vi.fn(),
  fetchWorkspaceLeaveRequests: vi.fn(),
  approveLeaveRequest: vi.fn(),
  fetchEmployeeLeaveBalances: vi.fn(),
  fetchLeavePolicies: vi.fn(),
  createLeavePolicy: vi.fn(),
  updateLeavePolicy: vi.fn(),
  deleteLeavePolicy: vi.fn(),
  assignLeavePolicy: vi.fn(),
  triggerAccrualRun: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
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
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe" }
    ]);
    (api.fetchWorkspaceLeaveRequests as any).mockResolvedValue([]);
    (api.fetchEmployeeLeaveBalances as any).mockResolvedValue([]);
    (api.fetchLeavePolicies as any).mockResolvedValue([]);
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
      { id: "req-1", employee_id: "emp-1", leave_type: "vacation", start_date: "2026-01-01", end_date: "2026-01-02", status: "pending", reason: "Test reason" }
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
    expect(screen.getByText("vacation")).toBeDefined();
    expect(screen.getAllByText("Chờ duyệt").length).toBeGreaterThan(0);
  });

  it("handles submitting a new leave request", async () => {
    const user = userEvent.setup();
    (api.submitLeaveRequest as any).mockResolvedValue({});
    (api.fetchEmployeeLeaveBalances as any).mockImplementation((wsId: string, empId: string) => {
      if (empId === "emp-1") {
        return Promise.resolve([
          { leave_type: "vacation", remaining: 10, entitlement: 12, carried_over: 2 }
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

    await user.click(screen.getByText("Tạo yêu cầu"));

    // Select employee
    const empSelect = await screen.findByLabelText("Nhân viên");
    await user.selectOptions(empSelect, "emp-1");

    // Wait for the balances to appear.
    expect(await screen.findByText(/Còn/)).toBeDefined();
    expect(screen.getByText("10")).toBeDefined();

    await user.selectOptions(screen.getByLabelText("Loại"), "vacation");
    
    // Fill dates
    const startDate = screen.getByLabelText("Từ ngày");
    fireEvent.change(startDate, { target: { value: "2026-01-01" } }); // user.type is tricky with type="date"
    const endDate = screen.getByLabelText("Đến ngày");
    fireEvent.change(endDate, { target: { value: "2026-01-02" } });
    
    const reasonInput = screen.getByPlaceholderText("Nhập lý do nghỉ...");
    await user.type(reasonInput, "Vacation time");

    await user.click(screen.getByText("Gửi yêu cầu"));

    await waitFor(() => {
      expect(api.submitLeaveRequest).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        employee_id: "emp-1",
        leave_type: "vacation",
        reason: "Vacation time"
      }));
    });
  });

  it("handles approve/reject actions", async () => {
    const user = userEvent.setup();
    (api.fetchWorkspaceLeaveRequests as any).mockResolvedValue([
      { id: "req-1", employee_id: "emp-1", leave_type: "vacation", start_date: "2026-01-01", end_date: "2026-01-02", status: "pending" }
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

    const approveBtn = await screen.findByTitle("Phê duyệt");
    await user.click(approveBtn);

    await waitFor(() => {
      expect(api.approveLeaveRequest).toHaveBeenCalledWith("test-ws-id", "req-1", { status: "approved" });
    });

    const rejectBtn = screen.getByTitle("Từ chối");
    await user.click(rejectBtn);

    await waitFor(() => {
      expect(api.approveLeaveRequest).toHaveBeenCalledWith("test-ws-id", "req-1", { status: "rejected" });
    });
  });

  it("manages policies", async () => {
    const user = userEvent.setup();
    (api.fetchLeavePolicies as any).mockResolvedValue([
      { id: "pol-1", name: "Standard Policy", leave_type: "vacation", rule_config: { frequency: "monthly", accrual_amount: 1, accrual_timing: "start_of_period", carry_over_limit: 5 } }
    ]);
    (api.createLeavePolicy as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <LeaveRequestsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    await user.click(screen.getByText("Chính sách"));

    expect(await screen.findByText("Standard Policy")).toBeDefined();

    // Create policy
    await user.click(screen.getByText("Thêm chính sách"));
    const nameInput = await screen.findByLabelText("Tên chính sách");
    await user.type(nameInput, "New Policy");
    await user.click(screen.getByText("Lưu chính sách"));

    await waitFor(() => {
      expect(api.createLeavePolicy).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({ name: "New Policy" }));
    });
  });

  it("edits an existing policy", async () => {
    const user = userEvent.setup();
    (api.fetchLeavePolicies as any).mockResolvedValue([
      { id: "pol-1", name: "Old Policy", leave_type: "vacation", rule_config: { frequency: "monthly", accrual_amount: 1, accrual_timing: "start_of_period", carry_over_limit: 5 } }
    ]);
    (api.updateLeavePolicy as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <LeaveRequestsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    await user.click(screen.getByText("Chính sách"));
    expect(await screen.findByText("Old Policy")).toBeDefined();

    const row = screen.getByText("Old Policy").closest("tr");
    const editBtn = row!.querySelector('button');
    await user.click(editBtn!);

    expect(await screen.findByText("Sửa chính sách")).toBeDefined();
    const nameInput = screen.getByLabelText("Tên chính sách");
    await user.clear(nameInput);
    await user.type(nameInput, "Updated Policy");
    await user.click(screen.getByText("Lưu thay đổi"));

    await waitFor(() => {
      expect(api.updateLeavePolicy).toHaveBeenCalledWith("test-ws-id", "pol-1", expect.objectContaining({ name: "Updated Policy" }));
    });
  });

  it("deletes a policy", async () => {
    const user = userEvent.setup();
    (api.fetchLeavePolicies as any).mockResolvedValue([
      { id: "pol-1", name: "To Be Deleted", leave_type: "vacation", rule_config: { frequency: "monthly", accrual_amount: 1, accrual_timing: "start_of_period", carry_over_limit: 5 } }
    ]);
    (api.deleteLeavePolicy as any).mockResolvedValue({});
    vi.spyOn(window, 'confirm').mockReturnValue(true);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <LeaveRequestsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    await user.click(screen.getByText("Chính sách"));
    expect(await screen.findByText("To Be Deleted")).toBeDefined();

    const row = screen.getByText("To Be Deleted").closest("tr");
    const deleteBtn = row!.querySelectorAll('button')[1]; // Second button in actions is Delete
    await user.click(deleteBtn);

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(api.deleteLeavePolicy).toHaveBeenCalledWith("test-ws-id", "pol-1");
    });
  });

  it("handles policy assignment", async () => {
    const user = userEvent.setup();
    (api.fetchLeavePolicies as any).mockResolvedValue([
      { id: "pol-1", name: "Standard", leave_type: "vacation", rule_config: { frequency: "monthly", accrual_amount: 1, accrual_timing: "start_of_period" } }
    ]);
    (api.assignLeavePolicy as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <LeaveRequestsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    await user.click(screen.getByText("Chính sách"));
    
    // Wait for policy list to be updated
    expect(await screen.findByText("Standard")).toBeDefined();

    await user.click(screen.getByText("Gán chính sách"));

    const empSelects = await screen.findAllByLabelText("Nhân viên");
    const empSelect = empSelects.find(el => el.id === 'assign-emp') || empSelects[0];
    await user.selectOptions(empSelect, "emp-1");
    
    const policySelect = await screen.findByLabelText("Chính sách");
    await user.selectOptions(policySelect, "pol-1");
    
    await user.click(screen.getByText("Gán ngay"));

    await waitFor(() => {
      expect(api.assignLeavePolicy).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        employee_id: "emp-1",
        leave_policy_id: "pol-1"
      }));
    });
  });
});