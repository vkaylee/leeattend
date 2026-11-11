import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as attendanceHooks from "@/hooks/use-attendance";
import { useMyPermissions } from "@/hooks/use-permissions";
import { fetchAttendanceReport } from "@/lib/api";

import { ToastProvider } from "../../components/ui/toast";
import OtApprovalsPage from "../approvals/ot-approvals-page";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue: string }) => {
      const translations: Record<string, string> = {
        "ot_approvals.title": "OT Approvals",
        "ot_approvals.subtitle": "Manage overtime",
        "ot_approvals.empty_state.title": "No pending OT",
        "ot_approvals.empty_state.description": "You are all caught up",
        "ot_approvals.tabs.post_ot": "ot_approvals.tabs.post_ot",
        "ot_approvals.tabs.pre_ot": "ot_approvals.tabs.pre_ot",
        "ot_approvals.tabs.delegated_claim": "ot_approvals.tabs.delegated_claim",
        "ot_approvals.tabs.behalf_assignments": "ot_approvals.tabs.behalf_assignments",
        "ot_approvals.delegated_claim.table.reason": "ot_approvals.delegated_claim.table.reason",
        "ot_approvals.delegated_claim.actions.claim": "ot_approvals.delegated_claim.actions.claim",
        "ot_approvals.delegated_claim.actions.claim_selected": "ot_approvals.delegated_claim.actions.claim_selected",
        "ot_approvals.delegated_claim.confirm_title": "ot_approvals.delegated_claim.confirm_title",
        "ot_approvals.delegated_claim.warning.title": "ot_approvals.delegated_claim.warning.title",
        "ot_approvals.delegated_claim.warning.btn_confirm": "ot_approvals.delegated_claim.warning.btn_confirm",
        "ot_approvals.delegated_claim.empty_state.title": "ot_approvals.delegated_claim.empty_state.title",
        "ot_approvals.delegated_claim.default_reason": "Delegated OT claim",
        "common.submit": "common.submit",
        "approvals.pre_ot_dialog.new_request_btn": "New Pre-OT Request",
        "approvals.bulk_upload.btn": "Bulk Upload",
      };
      const normalizedKey = key.replace(":", ".");
      return translations[normalizedKey] || translations[key] || options?.defaultValue || key;
    },
    i18n: { language: "en" },
  }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: vi.fn(() => ({
    hasPermission: (perm: string) => perm === "overtime:approve",
    hasAnyPermission: () => true,
    hasScopedPermission: () => true,
    hasRole: () => true,
    isLoading: false,
    permissions: ["overtime:approve"],
    roles: ["admin"],
    managedDepartmentIds: null,
  })),
}));

vi.mock("@/hooks/use-organization", () => ({
  useOrganization: vi.fn(() => ({
    organization: { id: "org-123", name: "Test Organization" },
    workspace: { id: "org-123", name: "Test Organization" },
    isLoading: false,
    isError: false,
  })),
}));

vi.mock("@/hooks/use-attendance", () => ({
  useAttendanceSummaries: vi.fn(),
  useAttendanceSummariesInfinite: vi.fn(),
  useApproveOt: vi.fn(),
  useEnterPaperOt: vi.fn(() => ({ enterPaperOt: vi.fn(), isSubmitting: false })),
  useBulkUploadPaperOt: vi.fn(() => ({ bulkUploadPaperOt: vi.fn(), isUploading: false })),
  useClaimOt: vi.fn(() => ({ claimOt: vi.fn(), isSubmitting: false })),
}));

vi.mock("@/hooks/use-overtime-requests", () => ({
  useOvertimeRequests: vi.fn(() => ({
    data: {
      data: [
        {
          id: "req-123",
          status: "pending_approval",
          planned_date: "2026-06-04",
          planned_start_time: "17:00:00",
          planned_end_time: "19:00:00",
          planned_minutes: 120,
          reason: "Late night deploy",
          employees: [{ employee_id: "employee-001", full_name: "Test Employee One" }],
        }
      ],
      total: 1,
      page: 1,
      limit: 20,
    },
    isLoading: false,
  })),
  useCreateOvertimeRequest: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
}));

vi.mock("@/hooks/use-approval-workflows", () => ({
  useProcessApproval: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
}));

vi.mock("@/lib/api/overtime-requests", () => ({
  overtimeRequestsApi: {
    listRequests: vi.fn(() => Promise.resolve({
      data: [
        {
          id: "req-123",
          status: "pending_approval",
          planned_date: "2026-06-04",
          planned_start_time: "17:00:00",
          planned_end_time: "19:00:00",
          planned_minutes: 120,
          reason: "Late night deploy",
          employees: [{ employee_id: "employee-001", full_name: "Test Employee One" }],
        },
      ],
      total: 1,
      page: 1,
      limit: 20,
    })),
    getRequest: vi.fn(),
    createRequest: vi.fn(),
    submitRequest: vi.fn(),
    cancelRequest: vi.fn(),
    employeeConsent: vi.fn(),
    getMyAssignments: vi.fn(),
    listRequestBehalfAssignments: vi.fn(),
    createRequestBehalfAssignment: vi.fn(),
    deleteRequestBehalfAssignment: vi.fn(),
  },
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...actual,
    fetchAttendanceReport: vi.fn(),
    fetchEmployees: vi.fn(() => Promise.resolve({
      data: [
        { id: "employee-001", employee_id_number: "EMP-001", full_name: "Test Employee One", active_employment_id: "employment-001" },
        { id: "employee-002", employee_id_number: "EMP-002", full_name: "Test Employee Two", active_employment_id: "employment-002" },
        { id: "employee-101", employee_id_number: "JD101", full_name: "John Doe", active_employment_id: "employment-101" },
        { id: "employee-102", employee_id_number: "JS102", full_name: "Jane Smith", active_employment_id: "employment-102" },
        { id: "employee-103", employee_id_number: "BJ103", full_name: "Bob Johnson", active_employment_id: "employment-103" },
        { id: "employee-104", employee_id_number: "AB104", full_name: "Alice Brown", active_employment_id: "employment-104" }
      ],
      total: 6,
      page: 1,
      limit: 1000
    }))
  };
});

vi.mock("../approvals/components/behalf-assignments-tab", () => ({
  BehalfAssignmentsTab: () => <div data-testid="behalf-assignments-tab">Mocked Behalf Assignments</div>,
}));

const mockSummaries = [
  {
    id: "summary-1",
    employment_id: "employment-001",
    date: "2026-05-01",
    claimed_ot_minutes: 120,
    system_detected_ot_minutes: 60,
    ot_reason: "Project deadline",
    entered_via: "self_service",
    ot_status: "pending_manager"
  },
  {
    id: "summary-2",
    employment_id: "employment-002",
    date: "2026-05-02",
    claimed_ot_minutes: 60,
    system_detected_ot_minutes: 60,
    ot_reason: "Covering shift",
    entered_via: "paper_form",
    ot_status: "pending_manager"
  }
];

describe("OtApprovalsPage", () => {
  let approveOtMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    approveOtMock = vi.fn().mockResolvedValue({});

    vi.mocked(attendanceHooks.useApproveOt).mockReturnValue({
      approveOt: approveOtMock,
      isProcessing: false,
    } as unknown as ReturnType<typeof attendanceHooks.useApproveOt>);

    vi.mocked(attendanceHooks.useAttendanceSummariesInfinite).mockReturnValue({
      summaries: mockSummaries,
      isLoading: false,
      fetchNextPage: vi.fn(),
      hasNextPage: false,
      isFetchingNextPage: false,
    } as unknown as ReturnType<typeof attendanceHooks.useAttendanceSummariesInfinite>);
  });

  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

  const renderPage = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <OtApprovalsPage />
        </ToastProvider>
      </QueryClientProvider>
    );

  it("renders page title and subtitle", () => {
    renderPage();
    expect(screen.getByText("OT Approvals")).toBeInTheDocument();
    expect(screen.getByText("Manage overtime")).toBeInTheDocument();
  });

  it("renders pending summaries in table", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByText("ot_approvals.tabs.post_ot"));

    // ResponsiveTable renders both desktop table and mobile cards, so use getAllByText
    expect(screen.getAllByText("EMP-001").length).toBeGreaterThan(0);
    expect(screen.getAllByText("120m").length).toBeGreaterThan(0);
    expect(screen.getAllByText('"Project deadline"').length).toBeGreaterThan(0);

    expect(screen.getAllByText("EMP-002").length).toBeGreaterThan(0);
    expect(screen.getAllByText('"Covering shift"').length).toBeGreaterThan(0);
  });

  it("calls approveOt and refetch when approve button is clicked", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByText("ot_approvals.tabs.post_ot"));
    
    const approveButtons = screen.getAllByTitle("ot_approvals.actions.approve");
    await user.click(approveButtons[0]);

    await waitFor(() => {
      expect(approveOtMock).toHaveBeenCalledWith({
        employeeId: "employee-001",
        date: "2026-05-01",
        approved_minutes: 120,
        status: "approved",
        manager_note: "Approved via Manager Portal"
      });
    });
  });

  it("calls approveOt with 0 minutes when reject button is clicked", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByText("ot_approvals.tabs.post_ot"));

    const rejectButtons = screen.getAllByTitle("ot_approvals.actions.reject");
    await user.click(rejectButtons[0]);

    await waitFor(() => {
      expect(approveOtMock).toHaveBeenCalledWith({
        employeeId: "employee-001",
        date: "2026-05-01",
        approved_minutes: 0,
        status: "rejected",
        manager_note: "Rejected via Manager Portal"
      });
    });
  });

  it("filters by source when filter buttons are clicked", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByText("ot_approvals.tabs.post_ot"));

    // Initially both should be visible (ResponsiveTable renders both desktop and mobile)
    expect(screen.getAllByText("EMP-001").length).toBeGreaterThan(0);
    expect(screen.getAllByText("EMP-002").length).toBeGreaterThan(0);

    // Click paper_form filter
    const paperFilter = screen.getByText("approvals.filter.paper_forms");
    await user.click(paperFilter);

    // Only EMP-002 (paper_form) should be visible
    expect(screen.queryByText("EMP-001")).not.toBeInTheDocument();
    expect(screen.getAllByText("EMP-002").length).toBeGreaterThan(0);
  });
  
  it("shows empty state when no pending summaries", async () => {
    vi.mocked(attendanceHooks.useAttendanceSummariesInfinite).mockReturnValue({
      summaries: [],
      isLoading: false,
      fetchNextPage: vi.fn(),
      hasNextPage: false,
      isFetchingNextPage: false,
    } as unknown as ReturnType<typeof attendanceHooks.useAttendanceSummariesInfinite>);

    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByText("ot_approvals.tabs.post_ot"));
    expect(screen.getByText("No pending OT")).toBeInTheDocument();
  });

  describe("Delegated Claim Tab", () => {
    let claimOtMock: ReturnType<typeof vi.fn>;

    beforeEach(() => {
      claimOtMock = vi.fn().mockResolvedValue({});
      vi.mocked(attendanceHooks.useClaimOt).mockReturnValue({
        claimOt: claimOtMock,
        isSubmitting: false,
      } as unknown as ReturnType<typeof attendanceHooks.useClaimOt>);

      vi.mocked(useMyPermissions).mockReturnValue({
        hasPermission: (perm: string) => perm === "overtime:claim_on_behalf",
        hasAnyPermission: () => true,
        hasScopedPermission: () => true,
        hasRole: () => true,
        isLoading: false,
        permissions: ["overtime:claim_on_behalf"],
        roles: ["clerk"],
        managedDepartmentIds: null,
      });

      vi.mocked(fetchAttendanceReport).mockResolvedValue({
        data: [
          {
            employee_id: "employee-101",
            employment_id: "employment-101",
            employee_name: "John Doe",
            employee_id_number: "JD101",
            date: "2026-05-10",
            computed_std_hours: 8,
            computed_ot_hours: 2.5,
            ot_claim_status: "pending_employee",
          },
          {
            employee_id: "employee-102",
            employment_id: "employment-102",
            employee_name: "Jane Smith",
            employee_id_number: "JS102",
            date: "2026-05-11",
            computed_std_hours: 8,
            computed_ot_hours: 1.5,
            ot_claim_status: "pending_employee",
          },
          {
            employee_id: "employee-103",
            employment_id: "employment-103",
            employee_name: "Bob Johnson",
            employee_id_number: "BJ103",
            date: "2026-05-12",
            computed_std_hours: 8,
            computed_ot_hours: 0,
            ot_claim_status: "pending_employee",
          },
          {
            employee_id: "employee-104",
            employment_id: "employment-104",
            employee_name: "Alice Brown",
            employee_id_number: "AB104",
            date: "2026-05-13",
            computed_std_hours: 8,
            computed_ot_hours: 3.0,
            ot_claim_status: "claimed",
          },
        ],
        total: 4,
        page: 1,
        limit: 20,
      } as any);
    });

    it("renders delegated claim tab button when user has overtime:claim_on_behalf", () => {
      renderPage();
      expect(screen.getByText("ot_approvals.tabs.delegated_claim")).toBeInTheDocument();
      expect(screen.queryByText("ot_approvals.tabs.pre_ot")).not.toBeInTheDocument();
      expect(screen.queryByText("ot_approvals.tabs.post_ot")).not.toBeInTheDocument();
    });

    it("renders list of claimable entries in table with correct hours", async () => {
      const user = userEvent.setup();
      renderPage();

      await user.click(screen.getByText("ot_approvals.tabs.delegated_claim"));

      await waitFor(() => {
        expect(screen.getByText("John Doe")).toBeInTheDocument();
        expect(screen.getByText("Jane Smith")).toBeInTheDocument();
        expect(screen.queryByText("Bob Johnson")).not.toBeInTheDocument();
        expect(screen.queryByText("Alice Brown")).not.toBeInTheDocument();

        expect(screen.getByText("2.5 hrs")).toBeInTheDocument();
        expect(screen.getByText("1.5 hrs")).toBeInTheDocument();
      });
    });

    it("submits delegated claim with custom reason when claim button is clicked", async () => {
      const user = userEvent.setup();
      renderPage();

      await user.click(screen.getByText("ot_approvals.tabs.delegated_claim"));

      await waitFor(() => {
        expect(screen.getByText("John Doe")).toBeInTheDocument();
      });

      const reasonInputs = screen.getAllByPlaceholderText("ot_approvals.delegated_claim.table.reason");
      await user.type(reasonInputs[0], "Finished inventory check");

      const claimButtons = screen.getAllByText("ot_approvals.delegated_claim.actions.claim");
      await user.click(claimButtons[0]);

      // Assert confirmation dialog is opened
      await waitFor(() => {
        expect(screen.getByText("ot_approvals.delegated_claim.confirm_title")).toBeInTheDocument();
      });

      // Submit from dialog
      const submitBtn = screen.getByText("common.submit");
      await user.click(submitBtn);

      await waitFor(() => {
        expect(claimOtMock).toHaveBeenCalledWith({
          employeeId: "employee-101",
          date: "2026-05-10",
          claimed_minutes: 150,
          reason: "Finished inventory check",
        });
      });
    });

    it("submits bulk delegated claims when multiple rows are checked and bulk claim is clicked", async () => {
      const user = userEvent.setup();
      renderPage();

      await user.click(screen.getByText("ot_approvals.tabs.delegated_claim"));

      await waitFor(() => {
        expect(screen.getByText("John Doe")).toBeInTheDocument();
      });

      const checkboxes = screen.getAllByRole("checkbox", { hidden: true });
      await user.click(checkboxes[1]);
      await user.click(checkboxes[2]);

      const bulkClaimBtn = screen.getByText("ot_approvals.delegated_claim.actions.claim_selected");
      await user.click(bulkClaimBtn);

      // Assert confirmation dialog is opened
      await waitFor(() => {
        expect(screen.getByText("ot_approvals.delegated_claim.confirm_title")).toBeInTheDocument();
      });

      // Submit from dialog
      const submitBtn = screen.getByText("common.submit");
      await user.click(submitBtn);

      await waitFor(() => {
        expect(claimOtMock).toHaveBeenCalledTimes(2);
        expect(claimOtMock).toHaveBeenNthCalledWith(1, {
          employeeId: "employee-101",
          date: "2026-05-10",
          claimed_minutes: 150,
          reason: "Delegated OT claim",
        });
        expect(claimOtMock).toHaveBeenNthCalledWith(2, {
          employeeId: "employee-102",
          date: "2026-05-11",
          claimed_minutes: 90,
          reason: "Delegated OT claim",
        });
      });
    });

    it("shows discrepancy warning modal when claimed minutes differ from system OT, and submits when confirmed", async () => {
      const user = userEvent.setup();
      renderPage();

      await waitFor(() => {
        expect(screen.getByText("John Doe")).toBeInTheDocument();
      });

      const claimButtons = screen.getAllByText("ot_approvals.delegated_claim.actions.claim");
      await user.click(claimButtons[0]);

      // Assert confirmation dialog is opened
      await waitFor(() => {
        expect(screen.getByText("ot_approvals.delegated_claim.confirm_title")).toBeInTheDocument();
      });

      // Change minutes input from 150 to 180
      const minutesInput = screen.getByRole("spinbutton");
      await user.clear(minutesInput);
      await user.type(minutesInput, "180");

      // Click submit
      const submitBtn = screen.getByText("common.submit");
      await user.click(submitBtn);

      // Assert discrepancy warning screen is shown
      await waitFor(() => {
        expect(screen.getAllByText("ot_approvals.delegated_claim.warning.title").length).toBeGreaterThan(0);
      });

      // Confirm and submit
      const confirmBtn = screen.getByText("ot_approvals.delegated_claim.warning.btn_confirm");
      await user.click(confirmBtn);

      await waitFor(() => {
        expect(claimOtMock).toHaveBeenCalledWith({
          employeeId: "employee-101",
          date: "2026-05-10",
          claimed_minutes: 180,
          reason: "Delegated OT claim",
        });
      });
    });

    it("filters list of claimable entries by employee name or ID search query", async () => {
      const user = userEvent.setup();
      renderPage();

      await waitFor(() => {
        expect(screen.getByText("John Doe")).toBeInTheDocument();
        expect(screen.getByText("Jane Smith")).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText("Search by employee name or ID...");
      await user.type(searchInput, "Jane");

      expect(screen.queryByText("John Doe")).not.toBeInTheDocument();
      expect(screen.getByText("Jane Smith")).toBeInTheDocument();

      // Clear and search by ID number
      await user.clear(searchInput);
      await user.type(searchInput, "JD101");

      expect(screen.getByText("John Doe")).toBeInTheDocument();
      expect(screen.queryByText("Jane Smith")).not.toBeInTheDocument();
    });

    it("only selects filtered/visible rows when search filter is active and select-all is clicked", async () => {
      const user = userEvent.setup();
      renderPage();

      await waitFor(() => {
        expect(screen.getByText("John Doe")).toBeInTheDocument();
        expect(screen.getByText("Jane Smith")).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText("Search by employee name or ID...");
      await user.type(searchInput, "Jane");

      // Select all (should only select Jane Smith, which is checkboxes[0] since John is filtered out)
      const checkboxes = screen.getAllByRole("checkbox", { hidden: true });
      const selectAllCheckbox = checkboxes[0];
      await user.click(selectAllCheckbox);

      const bulkClaimBtn = screen.getByText("ot_approvals.delegated_claim.actions.claim_selected");
      await user.click(bulkClaimBtn);

      // Submit from dialog
      const submitBtn = screen.getByText("common.submit");
      await user.click(submitBtn);

      await waitFor(() => {
        expect(claimOtMock).toHaveBeenCalledTimes(1);
        expect(claimOtMock).toHaveBeenCalledWith({
          employeeId: "employee-102",
          date: "2026-05-11",
          claimed_minutes: 90,
          reason: "Delegated OT claim",
        });
      });
    });

    it("renders empty state when there are no pending employee OT entries to claim", async () => {
      vi.mocked(fetchAttendanceReport).mockResolvedValue({ data: [] } as any);
      renderPage();

      await waitFor(() => {
        expect(screen.getByText("ot_approvals.delegated_claim.empty_state.title")).toBeInTheDocument();
      });
    });
  });

  describe("Behalf Assignments Tab Integration", () => {
    it("hides Behalf tab by default", () => {
      vi.mocked(useMyPermissions).mockReturnValue({
        hasPermission: (perm: string) => perm === "overtime:approve",
        hasAnyPermission: () => true,
        hasScopedPermission: () => true,
        hasRole: () => true,
        isLoading: false,
        permissions: ["overtime:approve"],
        roles: ["admin"],
        managedDepartmentIds: null,
      });

      renderPage();
      expect(screen.queryByText("ot_approvals.tabs.behalf_assignments")).not.toBeInTheDocument();
    });

    it("shows Behalf tab and switches content when user has permission", async () => {
      vi.mocked(useMyPermissions).mockReturnValue({
        hasPermission: (perm: string) => perm === "overtime:policies:manage",
        hasAnyPermission: () => true,
        hasScopedPermission: () => true,
        hasRole: () => true,
        isLoading: false,
        permissions: ["overtime:policies:manage"],
        roles: ["admin"],
        managedDepartmentIds: null,
      });

      const user = userEvent.setup();
      renderPage();

      const behalfTab = screen.getByText("ot_approvals.tabs.behalf_assignments");
      expect(behalfTab).toBeInTheDocument();

      await user.click(behalfTab);

      expect(screen.getAllByTestId("behalf-assignments-tab").length).toBe(2);
    });
  });

  describe("Header & Filter Controls Visibility", () => {
    it("shows only New Pre-OT Request and no filters on pre tab, and only Bulk Upload and filters on post tab", async () => {
      vi.mocked(useMyPermissions).mockReturnValue({
        hasPermission: (perm: string) =>
          perm === "overtime:approve" || perm === "overtime:claim_on_behalf" || perm === "overtime:request:create_on_behalf",
        hasAnyPermission: () => true,
        hasScopedPermission: () => true,
        hasRole: () => true,
        isLoading: false,
        permissions: ["overtime:approve", "overtime:claim_on_behalf", "overtime:request:create_on_behalf"],
        roles: ["admin"],
        managedDepartmentIds: null,
      });

      const user = userEvent.setup();
      renderPage();

      // 1. Initially on "pre" tab: should see "New Pre-OT Request" but NOT "Bulk Upload" or filter
      expect(screen.getByText("New Pre-OT Request")).toBeInTheDocument();
      expect(screen.queryByText("Bulk Upload")).not.toBeInTheDocument();
      expect(screen.queryByText("approvals.filter.paper_forms")).not.toBeInTheDocument();

      // 2. Switch to "post" tab: should see "Bulk Upload" and source filter, but NOT "New Pre-OT Request"
      await user.click(screen.getByText("ot_approvals.tabs.post_ot"));
      expect(screen.queryByText("New Pre-OT Request")).not.toBeInTheDocument();
      expect(screen.getByText("Bulk Upload")).toBeInTheDocument();
      expect(screen.getByText("approvals.filter.paper_forms")).toBeInTheDocument();

      // 3. Switch to "delegated_claim" tab: should see "Bulk Upload" but NOT "New Pre-OT Request" or filter
      await user.click(screen.getByText("ot_approvals.tabs.delegated_claim"));
      expect(screen.queryByText("New Pre-OT Request")).not.toBeInTheDocument();
      expect(screen.getByText("Bulk Upload")).toBeInTheDocument();
      expect(screen.queryByText("approvals.filter.paper_forms")).not.toBeInTheDocument();
    });

    it("shows no action buttons or filters on behalf_assignments tab", async () => {
      vi.mocked(useMyPermissions).mockReturnValue({
        hasPermission: (perm: string) => perm === "overtime:policies:manage",
        hasAnyPermission: () => true,
        hasScopedPermission: () => true,
        hasRole: () => true,
        isLoading: false,
        permissions: ["overtime:policies:manage"],
        roles: ["admin"],
        managedDepartmentIds: null,
      });

      const user = userEvent.setup();
      renderPage();

      // Switch to behalf tab
      await user.click(screen.getByText("ot_approvals.tabs.behalf_assignments"));
      expect(screen.queryByText("New Pre-OT Request")).not.toBeInTheDocument();
      expect(screen.queryByText("Bulk Upload")).not.toBeInTheDocument();
      expect(screen.queryByText("approvals.filter.paper_forms")).not.toBeInTheDocument();
    });
  });

  describe("Pre-OT Tab Permissions", () => {
    it("shows Pre-OT tab and creation button for create_on_behalf only user, but hides approve/reject buttons and bulk upload", async () => {
      vi.mocked(useMyPermissions).mockReturnValue({
        hasPermission: (perm: string) => perm === "overtime:request:create_on_behalf",
        hasAnyPermission: () => true,
        hasScopedPermission: () => true,
        hasRole: () => true,
        isLoading: false,
        permissions: ["overtime:request:create_on_behalf"],
        roles: ["clerk"],
        managedDepartmentIds: null,
      });

      renderPage();

      // Should see Pre-OT tab
      expect(screen.getByText("ot_approvals.tabs.pre_ot")).toBeInTheDocument();
      // Should NOT see Post-OT tab
      expect(screen.queryByText("ot_approvals.tabs.post_ot")).not.toBeInTheDocument();

      // Should see "New Pre-OT Request"
      expect(screen.getByText("New Pre-OT Request")).toBeInTheDocument();
      // Should NOT see "Bulk Upload"
      expect(screen.queryByText("Bulk Upload")).not.toBeInTheDocument();

      // Should see the pending requests but action buttons (Approve/Reject) should be hidden
      expect(screen.getByText("Test Employee One")).toBeInTheDocument();
      expect(screen.queryByText("common.approve")).not.toBeInTheDocument();
      expect(screen.queryByText("common.reject")).not.toBeInTheDocument();
    });

    it("shows Approve/Reject buttons in Pre-OT tab for users with overtime:approve", async () => {
      vi.mocked(useMyPermissions).mockReturnValue({
        hasPermission: (perm: string) => perm === "overtime:approve" || perm === "overtime:request:create_on_behalf",
        hasAnyPermission: () => true,
        hasScopedPermission: () => true,
        hasRole: () => true,
        isLoading: false,
        permissions: ["overtime:approve", "overtime:request:create_on_behalf"],
        roles: ["admin"],
        managedDepartmentIds: null,
      });

      renderPage();

      // Should see the pending requests and action buttons (Approve/Reject)
      expect(screen.getByText("Test Employee One")).toBeInTheDocument();
      expect(screen.getByText("common.approve")).toBeInTheDocument();
      expect(screen.getByText("common.reject")).toBeInTheDocument();
    });
  });
});
