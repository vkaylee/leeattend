import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api";
import type { Employee } from "@/lib/api/types";
import EmployeeDetailPage from "../employees/employee-detail-page";

const mockNavigate = vi.fn();
let mockParams = { employeeId: "emp-1" };
let mockSearch = { tab: "profile" };

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
  useParams: () => mockParams,
  useSearch: () => mockSearch,
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...actual,
    fetchEmployee: vi.fn(),
    fetchEmployeeDepartmentHistory: vi.fn(),
    fetchDepartments: vi.fn(),
    fetchWorkspaceMembers: vi.fn(),
    fetchEmployeeEmployments: vi.fn(),
    fetchMyPermissions: vi.fn(),
    getStandaloneOrganizationId: vi.fn(),
  };
});

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasPermission: () => true,
    hasAnyPermission: () => true,
    hasScopedPermission: () => true,
    isLoading: false,
    permissions: ["employees:update", "employees:delete"],
  }),
}));

vi.mock("@/pages/settings/hooks/use-locations", () => ({
  useLocations: () => ({ data: [] }),
}));

vi.mock("@/components/features/employees/employee-detail-drawer", () => ({
  default: () => null,
  DrawerTabContent: ({ activeDetailTab }: { activeDetailTab: string }) => (
    <div data-testid={`tab-content-${activeDetailTab}`}>Content for {activeDetailTab}</div>
  ),
}));

const mockEmployeeData: Employee = {
  id: "emp-1",
  full_name: "Alice Nguyen",
  employee_id_number: "EMP-001",
  workspace_id: "ws-1",
  active_employment_id: "employment-1",
  department_name: "Engineering",
  job_title: "Senior Software Engineer",
  hire_date: "2023-01-15T00:00:00Z",
  created_at: "2023-01-15T00:00:00Z",
  updated_at: "2023-01-15T00:00:00Z",
  tags: [],
};

describe("EmployeeDetailPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    mockParams = { employeeId: "emp-1" };
    mockSearch = { tab: "profile" };

    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0, staleTime: 0 },
      },
    });

    vi.mocked(api.getStandaloneOrganizationId).mockReturnValue("ws-1");
    vi.mocked(api.fetchEmployee).mockResolvedValue(mockEmployeeData);
    vi.mocked(api.fetchDepartments).mockResolvedValue([
      { id: "dept-1", name: "Engineering", workspace_id: "ws-1" },
    ]);
    vi.mocked(api.fetchEmployeeDepartmentHistory).mockResolvedValue([]);
    vi.mocked(api.fetchWorkspaceMembers).mockResolvedValue([
      {
        id: "mem-1",
        user_id: "usr-1",
        email: "alice@company.com",
        role: "admin",
        joined_at: "2023-01-15",
      },
    ]);
    vi.mocked(api.fetchEmployeeEmployments).mockResolvedValue([
      {
        id: "employment-1",
        employee_id: "emp-1",
        status: "active",
        effective_date: "2023-01-15",
        created_at: "2023-01-15",
        updated_at: "2023-01-15",
      },
    ]);
    vi.mocked(api.fetchMyPermissions).mockResolvedValue({
      permissions: ["employees:update", "employees:delete"],
      roles: ["admin"],
      managed_department_ids: [],
    });
  });

  const renderComponent = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <EmployeeDetailPage />
        </ToastProvider>
      </QueryClientProvider>
    );

  it("renders the Executive Hero Banner with employee identity and meta", async () => {
    renderComponent();

    // Verify employee name and code
    expect(await screen.findByRole("heading", { name: "Alice Nguyen", level: 1 })).toBeInTheDocument();
    expect(screen.getAllByText("EMP-001").length).toBeGreaterThan(0);

    // Verify metadata
    expect(screen.getByText("Senior Software Engineer")).toBeInTheDocument();
    expect(screen.getByText("Engineering")).toBeInTheDocument();

    // Verify action buttons
    expect(screen.getByTestId("edit-employee-btn")).toBeInTheDocument();
    expect(screen.getByTestId("transfer-employee-btn")).toBeInTheDocument();
    expect(screen.getByTestId("terminate-employee-btn")).toBeInTheDocument();
  });

  it("renders 10 sticky navigation tabs", async () => {
    renderComponent();

    expect(await screen.findByRole("heading", { name: "Alice Nguyen" })).toBeInTheDocument();

    const expectedTabs = [
      "tab-profile",
      "tab-documents",
      "tab-contracts",
      "tab-phases",
      "tab-assignments",
      "tab-salary",
      "tab-compensation",
      "tab-working-conditions",
      "tab-attendance-policy",
      "tab-lifecycle",
    ];

    for (const tabId of expectedTabs) {
      expect(screen.getByTestId(tabId)).toBeInTheDocument();
    }
  });

  it("changes tab on click and updates navigation", async () => {
    const user = userEvent.setup();
    renderComponent();

    expect(await screen.findByRole("heading", { name: "Alice Nguyen" })).toBeInTheDocument();

    const contractsTab = screen.getByTestId("tab-contracts");
    await user.click(contractsTab);

    expect(mockNavigate).toHaveBeenCalledWith({
      search: expect.any(Function),
      replace: true,
    });
  });

  it("renders error state when employee is not found", async () => {
    vi.mocked(api.fetchEmployee).mockRejectedValueOnce(new Error("Employee not found"));

    renderComponent();

    await waitFor(() => {
      expect(screen.getByText(/not_found|Không tìm thấy/i)).toBeInTheDocument();
    });
    expect(screen.getByText(/back_to_list|Quay lại danh sách/i)).toBeInTheDocument();
  });
});
