import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import { LeaveRequest, EmployeeLeaveBalance } from "../../lib/api/types";
import LeaveRequestsPage from "../leave-requests-page";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

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
  // LeaveImportDialog mocks
  uploadLeaveExcel: vi.fn(),
  fetchLatestLeaveImport: vi.fn().mockResolvedValue(null),
  getLeaveImportErrorsUrl: vi.fn().mockReturnValue("#"),
  getLeaveImportTemplateUrl: vi.fn().mockReturnValue("#"),
  downloadLeaveImportTemplate: vi.fn().mockResolvedValue(undefined),
  downloadLeaveImportErrors: vi.fn().mockResolvedValue(undefined),
}));

// Mock EmployeeSearch
 
vi.mock("@/components/common/employee-search", () => ({
  EmployeeSearch: ({ value, onChange }: { value: string; onChange: (value_: string) => void }) => (
    <select 
      aria-label="Nhân viên"
      onChange={(e) => { onChange(e.target.value); }} 
      value={value}
    >
      <option value="">Chọn...</option>
      <option value="emp-1">John Doe</option>
    </select>
  )
}));

const renderPage = (queryClient: QueryClient) =>
  render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <LeaveRequestsPage />
      </ToastProvider>
    </QueryClientProvider>
  );

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
 
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", original_hire_date: "2020-01-01", hire_date: "2025-01-01", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }
      ],
      total: 1,
      page: 1,
      limit: 50,
 
    });
 
    vi.mocked(api.fetchWorkspaceLeaveRequests).mockResolvedValue([]);
 
    vi.mocked(api.fetchEmployeeLeaveBalances).mockResolvedValue([]);
 
    vi.mocked(api.fetchLeavePolicies).mockResolvedValue([]);
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([
      { id: "lt-vacation", code: "lt-vacation", name: "Vacation", is_paid: true, workspace_id: "ws-1", policy_config: {}, created_at: "2025-01-01T00:00:00Z" }
    ]);
  });

 
  it("renders without workspace selected", () => {
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue(null);
    renderPage(queryClient);

    expect(screen.getByText("Chọn Workspace")).toBeDefined();
  });

 
  it("renders pending requests by default", async () => {
    vi.mocked(api.fetchWorkspaceLeaveRequests).mockResolvedValue([
      { id: "req-1", employee_id: "emp-1", leave_type_id: "lt-vacation", start_date: "2026-01-01", end_date: "2026-01-02", status: "pending", reason: "Test reason", duration_type: "full_day" }
    ] as unknown as LeaveRequest[]);

    renderPage(queryClient);

    expect(await screen.findByText("John Doe")).toBeDefined();
    expect(screen.getByText(/vacation/i)).toBeDefined();
    expect(screen.getAllByText(/Chờ duyệt/i).length).toBeGreaterThan(0);
  });

  it("handles submitting a new leave request", async () => {
 
    const user = userEvent.setup();
 
    vi.mocked(api.submitLeaveRequest).mockResolvedValue({ id: "req-new" } as unknown as LeaveRequest);
    vi.mocked(api.fetchEmployeeLeaveBalances).mockImplementation((wsId: string, empId: string) => {
      if (empId === "emp-1") {
        return Promise.resolve([
          { leave_type_id: "lt-1", leave_type_code: "lt-vacation", leave_type_name: "Vacation", is_paid: true, remaining: 10, entitlement: 12, carried_over: 2, accrued_amount: 0, used: 2 }
        ] as unknown as EmployeeLeaveBalance[]);
      }
      return Promise.resolve([] as unknown as EmployeeLeaveBalance[]);
    });

    renderPage(queryClient);

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
        reason: "Vacation time",
        duration_type: "full_day",
      }));
    }, { timeout: 20000 });
  }, 30000);

  it("handles approve/reject actions", async () => {
 
    const user = userEvent.setup();
    vi.mocked(api.fetchWorkspaceLeaveRequests).mockResolvedValue([
      { id: "req-1", employee_id: "emp-1", leave_type_id: "lt-vacation", start_date: "2026-01-01", end_date: "2026-01-02", status: "pending", duration_type: "full_day" }
 
    ] as unknown as LeaveRequest[]);
    vi.mocked(api.approveLeaveRequest).mockResolvedValue(undefined);

    renderPage(queryClient);

    const approveButton = await screen.findByTitle(/phê duyệt/i);
    await user.click(approveButton);

    await waitFor(() => {
      expect(api.approveLeaveRequest).toHaveBeenCalledWith("test-ws-id", "req-1", { status: "approved" });
    });

    const rejectButton = await screen.findByTitle(/từ chối/i);
    await user.click(rejectButton);

    await waitFor(() => {
      expect(api.approveLeaveRequest).toHaveBeenCalledWith("test-ws-id", "req-1", { status: "rejected" });
    });
  });

  // ═══════════════════════════════════════════════════════════════════
  // Partial Day / Duration Type Tests
  // ═══════════════════════════════════════════════════════════════════

  describe("Duration Type - Segmented Buttons", () => {
    it("renders all 4 duration type buttons in the modal", async () => {
      const user = userEvent.setup();
      renderPage(queryClient);

      await user.click(screen.getByText(/Tạo yêu cầu/i));

      // Wait for modal to be open and check all 4 buttons exist
      expect(await screen.findByText(/Cả ngày/i)).toBeDefined();
      expect(screen.getByText(/Sáng/i)).toBeDefined();
      expect(screen.getByText(/Chiều/i)).toBeDefined();
      expect(screen.getByText(/Tùy chỉnh/i)).toBeDefined();
    });

    it("disables end_date when half-day is selected", async () => {
      const user = userEvent.setup();
      renderPage(queryClient);

      await user.click(screen.getByText(/Tạo yêu cầu/i));
      await screen.findByText(/Cả ngày/i);

      // Initially end_date should be enabled
      const endDateInput = screen.getByLabelText(/Đến ngày/i);
      expect(endDateInput).not.toBeDisabled();

      // Click "Sáng" (first_half)
      await user.click(screen.getByText(/Sáng/i));

      // end_date should now be disabled
      expect(endDateInput).toBeDisabled();
    });

    it("shows time pickers only when custom_hours is selected", async () => {
      const user = userEvent.setup();
      renderPage(queryClient);

      await user.click(screen.getByText(/Tạo yêu cầu/i));
      await screen.findByText(/Cả ngày/i);

      // Time pickers should NOT be visible initially
      expect(screen.queryByLabelText(/Từ giờ/i)).toBeNull();
      expect(screen.queryByLabelText(/Đến giờ/i)).toBeNull();

      // Click "Tùy chỉnh" (custom_hours)
      await user.click(screen.getByText(/Tùy chỉnh/i));

      // Time pickers SHOULD be visible now
      expect(screen.getByLabelText(/Từ giờ/i)).toBeDefined();
      expect(screen.getByLabelText(/Đến giờ/i)).toBeDefined();

      // Switch back to "Cả ngày" and time pickers should hide
      await user.click(screen.getByText(/Cả ngày/i));
      expect(screen.queryByLabelText(/Từ giờ/i)).toBeNull();
    });

    it("shows half-day info note when first_half is selected", async () => {
      const user = userEvent.setup();
      renderPage(queryClient);

      await user.click(screen.getByText(/Tạo yêu cầu/i));
      await screen.findByText(/Cả ngày/i);

      // Info note should NOT be visible initially
      expect(screen.queryByText(/tự động bằng ngày bắt đầu/i)).toBeNull();

      // Click "Sáng"
      await user.click(screen.getByText(/Sáng/i));

      // Info note SHOULD be visible
      expect(screen.getByText(/tự động bằng ngày bắt đầu/i)).toBeDefined();
    });
  });

  describe("Duration Type - Submit Behavior", () => {
    const setupSubmitTest = async () => {
 
      const user = userEvent.setup();
 
      vi.mocked(api.submitLeaveRequest).mockResolvedValue({ id: "req-setup" } as unknown as LeaveRequest);
      vi.mocked(api.fetchEmployeeLeaveBalances).mockImplementation((_wsId: string, empId: string) => {
        if (empId === "emp-1") {
          return Promise.resolve([
            { leave_type_id: "lt-1", leave_type_code: "lt-vacation", leave_type_name: "Vacation", is_paid: true, remaining: 10, entitlement: 12, carried_over: 2, accrued_amount: 0, used: 2 }
          ] as unknown as EmployeeLeaveBalance[]);
        }
        return Promise.resolve([] as unknown as EmployeeLeaveBalance[]);
      });
      renderPage(queryClient);

      await user.click(screen.getByText(/Tạo yêu cầu/i));
      const empSelect = await screen.findByLabelText(/Nhân viên/i);
      fireEvent.change(empSelect, { target: { value: "emp-1" } });
      await screen.findByText(/Còn/i);

      await user.selectOptions(screen.getByLabelText(/Loại/i), "lt-vacation");
      const startDate = screen.getByLabelText(/Từ ngày/i);
      fireEvent.change(startDate, { target: { value: "2026-03-15" } });

      return user;
    };

    it("submits with first_half duration_type", async () => {
      const user = await setupSubmitTest();

      // Set end_date before switching (it gets disabled after)
      const endDate = screen.getByLabelText(/Đến ngày/i);
      fireEvent.change(endDate, { target: { value: "2026-03-15" } });

      // Select "Sáng" (first_half)
      await user.click(screen.getByText(/Sáng/i));

      const reasonInput = screen.getByPlaceholderText(/Nhập lý do nghỉ/i);
      await user.type(reasonInput, "Morning doctor visit");

      await user.click(screen.getByRole("button", { name: /Gửi yêu cầu/i }));

      await waitFor(() => {
        expect(api.submitLeaveRequest).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
          employee_id: "emp-1",
          leave_type_id: "lt-vacation",
          start_date: "2026-03-15",
          duration_type: "first_half",
          reason: "Morning doctor visit",
        }));
      }, { timeout: 20000 });

 
      // Verify no start_time/end_time sent for half-day
      const callArguments = vi.mocked(api.submitLeaveRequest).mock.calls[0][1] as unknown as { start_time?: string; end_time?: string };
      expect(callArguments.start_time).toBeUndefined();
      expect(callArguments.end_time).toBeUndefined();
    }, 30000);

    it("submits with custom_hours and time values", async () => {
      const user = await setupSubmitTest();

      // Set end_date before switching
      const endDate = screen.getByLabelText(/Đến ngày/i);
      fireEvent.change(endDate, { target: { value: "2026-03-15" } });

      // Select "Tùy chỉnh" (custom_hours)
      await user.click(screen.getByText(/Tùy chỉnh/i));

      // Fill time pickers
      const startTimeInput = screen.getByLabelText(/Từ giờ/i);
      fireEvent.change(startTimeInput, { target: { value: "09:00" } });
      const endTimeInput = screen.getByLabelText(/Đến giờ/i);
      fireEvent.change(endTimeInput, { target: { value: "12:00" } });

      const reasonInput = screen.getByPlaceholderText(/Nhập lý do nghỉ/i);
      await user.type(reasonInput, "Personal errand");

      await user.click(screen.getByRole("button", { name: /Gửi yêu cầu/i }));

      await waitFor(() => {
        expect(api.submitLeaveRequest).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
          employee_id: "emp-1",
          leave_type_id: "lt-vacation",
          start_date: "2026-03-15",
          duration_type: "custom_hours",
          start_time: "09:00",
          end_time: "12:00",
          reason: "Personal errand",
        }));
      }, { timeout: 20000 });
    }, 30000);

    it("submits with second_half duration_type", async () => {
      const user = await setupSubmitTest();

      const endDate = screen.getByLabelText(/Đến ngày/i);
      fireEvent.change(endDate, { target: { value: "2026-03-15" } });

      // Select "Chiều" (second_half)
      await user.click(screen.getByText(/Chiều/i));

      const reasonInput = screen.getByPlaceholderText(/Nhập lý do nghỉ/i);
      await user.type(reasonInput, "Afternoon off");

      await user.click(screen.getByRole("button", { name: /Gửi yêu cầu/i }));

      await waitFor(() => {
        expect(api.submitLeaveRequest).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
          employee_id: "emp-1",
          duration_type: "second_half",
          reason: "Afternoon off",
        }));
      }, { timeout: 20000 });
    }, 30000);
  });

  describe("Duration Type - Table Display", () => {
 
    it("shows half-day badge for first_half requests", async () => {
      vi.mocked(api.fetchWorkspaceLeaveRequests).mockResolvedValue([
        { id: "req-half", employee_id: "emp-1", leave_type_id: "lt-vacation", start_date: "2026-03-15", end_date: "2026-03-15", status: "pending", reason: "Morning off", duration_type: "first_half" }
      ] as unknown as LeaveRequest[]);

      renderPage(queryClient);

      // Wait for the request to render
      expect(await screen.findByText("John Doe")).toBeDefined();
      // The badge should show "Sáng" (i18n key: leave_requests.duration.first_half)
      expect(screen.getByText(/Sáng/i)).toBeDefined();
    });

 
    it("shows custom hours time range badge for custom_hours requests", async () => {
      vi.mocked(api.fetchWorkspaceLeaveRequests).mockResolvedValue([
        { id: "req-hours", employee_id: "emp-1", leave_type_id: "lt-vacation", start_date: "2026-03-15", end_date: "2026-03-15", status: "pending", reason: "Errand", duration_type: "custom_hours", start_time: "09:00:00", end_time: "12:00:00" }
      ] as unknown as LeaveRequest[]);

      renderPage(queryClient);

      expect(await screen.findByText("John Doe")).toBeDefined();
      // The badge should show the time range "09:00–12:00"
      expect(screen.getByText(/09:00/)).toBeDefined();
      expect(screen.getByText(/12:00/)).toBeDefined();
    });

 
    it("does NOT show duration badge for full_day requests", async () => {
      vi.mocked(api.fetchWorkspaceLeaveRequests).mockResolvedValue([
        { id: "req-full", employee_id: "emp-1", leave_type_id: "lt-vacation", start_date: "2026-01-01", end_date: "2026-01-02", status: "pending", reason: "Vacation", duration_type: "full_day" }
      ] as unknown as LeaveRequest[]);

      renderPage(queryClient);

      expect(await screen.findByText("John Doe")).toBeDefined();
      // For full_day, the duration badge (with Sun/Moon/Timer icons) should NOT appear in the table
      // The time cell only shows date range, no "Sáng"/"Chiều" labels inside a badge span
      const timeCells = document.querySelectorAll("td");
      let foundDurationBadge = false;
      timeCells.forEach(cell => {
        // Look for the specific badge wrapper that only appears for partial-day types
        const badges = cell.querySelectorAll('[class*="inline-flex"]');
        badges.forEach(badge => {
          const text = badge.textContent || "";
          if (text.includes("Sáng") || text.includes("Chiều") || text.includes("09:00")) {
            foundDurationBadge = true;
          }
        });
      });
      expect(foundDurationBadge).toBe(false);
    });
  });

  describe("Duration Type - State Reset", () => {
    it("resets duration type when modal is reopened", async () => {
      const user = userEvent.setup();
      renderPage(queryClient);

      // Open modal
      await user.click(screen.getByText(/Tạo yêu cầu/i));
      await screen.findByText(/Cả ngày/i);

      // Select half-day
      await user.click(screen.getByText(/Chiều/i));
      expect(screen.getByLabelText(/Đến ngày/i)).toBeDisabled();

      // Close modal
      await user.click(screen.getByText(/HỦY BỎ/i));

      // Reopen modal
      await user.click(screen.getByText(/Tạo yêu cầu/i));
      await screen.findByText(/Cả ngày/i);

      // end_date should be enabled again (reset to full_day)
      expect(screen.getByLabelText(/Đến ngày/i)).not.toBeDisabled();
    });
  });
});