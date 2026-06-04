import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import { AttendanceAdjustmentRequest } from "../../lib/api/types";
import AttendanceAdjustmentsPage from "../attendance-adjustments-page";

(globalThis as any).__mockPermissions = ["attendance:adjustments:manage_behalf"];

vi.mock("../../hooks/use-permissions", () => ({
  useMyPermissions: vi.fn(() => {
    const perms = (globalThis as any).__mockPermissions || [];
    return {
      hasPermission: (perm: string) => perms.includes(perm),
      hasAnyPermission: (...p: string[]) => p.some((x) => perms.includes(x)),
      hasScopedPermission: () => true,
      hasRole: () => true,
      isLoading: false,
      permissions: perms,
      roles: [],
      managedDepartmentIds: null,
    };
  }),
}));

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock useBehalfEmployees
vi.mock("@/hooks/use-behalf-employees", () => ({
  useBehalfEmployees: vi.fn(() => ({
    employees: [
      { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", is_active: true }
    ],
    hasAssignments: true,
    isLoading: false,
    isError: false,
  })),
}));

// Mock Combobox
vi.mock("@/components/ui/combobox", () => ({
  Combobox: ({ id, value, onChange, options }: any) => (
    <select
      data-testid="mock-employee-search"
      id={id}
      onChange={(e) => onChange(e.target.value)}
      value={value}
    >
      <option value="">Select...</option>
      {options.map((opt: any) => (
        <option key={opt.value} value={opt.value}>{opt.label}</option>
      ))}
    </select>
  ),
}));


// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
  fetchEmployees: vi.fn(),
  approveAttendanceAdjustment: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  submitAttendanceAdjustment: vi.fn(),
  fetchWorkspace: vi.fn(),
}));

// Override i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("AttendanceAdjustmentsPage", () => {
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
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, original_hire_date: "2020-01-01", hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }
      ],
      total: 1,
      page: 1,
      limit: 50,
    });
    vi.mocked(api.fetchWorkspace).mockResolvedValue({
      id: "test-ws-id",
      name: "Test Workspace",
      timezone: "Asia/Ho_Chi_Minh",
    });
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([
      { 
        id: "adj-1", 
        employee_id: "emp-1", 
        requested_date: "2026-01-01", 
        requested_punch_in: "08:00", 
        requested_punch_out: "17:00", 
        status: "pending", 
        reason: "Forgot to punch" 
      }
    ] as unknown as AttendanceAdjustmentRequest[]);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders list of adjustment requests", async () => {
    renderWithProviders(<AttendanceAdjustmentsPage />);
    expect(await screen.findByText(/John Doe/i)).toBeDefined();
    expect(screen.getByText(/Forgot to punch/i)).toBeDefined();
  });

  it("handles approve action", async () => {
    (globalThis as any).__mockPermissions = [
      "attendance:adjustments:manage_behalf",
      "attendance:adjustments:approve"
    ];
    const user = userEvent.setup();
    vi.mocked(api.approveAttendanceAdjustment).mockResolvedValue(undefined);

    renderWithProviders(<AttendanceAdjustmentsPage />);
    
    const approveButton = await screen.findByTitle(/common.approve/i);
    await user.click(approveButton);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith("test-ws-id", "adj-1", { status: "approved" });
    });
  });

  it("handles request on behalf action", async () => {
    (globalThis as any).__mockPermissions = [
      "attendance:adjustments:manage_behalf",
      "attendance:adjustments:submit_on_behalf"
    ];
    const user = userEvent.setup();
    vi.mocked(api.submitAttendanceAdjustment).mockResolvedValue({} as any);

    renderWithProviders(<AttendanceAdjustmentsPage />);

    // 1. Verify "Request Adjustment" button is visible
    const requestButton = await screen.findByRole("button", { name: /time_tracking.modal.new_title/i });
    expect(requestButton).toBeDefined();

    // 2. Click to open modal
    await user.click(requestButton);

    // 3. Verify modal title is displayed
    expect(await screen.findByText("time_tracking.modal.new_title")).toBeDefined();

    // 4. Fill form
    const selectElement = screen.getByTestId("mock-employee-search");
    await user.selectOptions(selectElement, "emp-1");

    const dateInput = screen.getByLabelText(/common.work_day/i);
    await user.clear(dateInput);
    await user.type(dateInput, "2026-06-05");

    const reasonInput = screen.getByPlaceholderText("time_tracking.modal.reason_placeholder");
    await user.type(reasonInput, "On behalf test reason");

    // Submit form
    const submitButton = screen.getByRole("button", { name: /common.submit_request/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(api.submitAttendanceAdjustment).toHaveBeenCalled();
    });
  });
});
