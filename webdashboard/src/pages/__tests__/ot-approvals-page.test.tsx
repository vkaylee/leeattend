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
        "ot_approvals.delegated_claim.table.reason": "ot_approvals.delegated_claim.table.reason",
        "ot_approvals.delegated_claim.actions.claim": "ot_approvals.delegated_claim.actions.claim",
        "ot_approvals.delegated_claim.actions.claim_selected": "ot_approvals.delegated_claim.actions.claim_selected",
        "ot_approvals.delegated_claim.confirm_title": "ot_approvals.delegated_claim.confirm_title",
        "ot_approvals.delegated_claim.warning.title": "ot_approvals.delegated_claim.warning.title",
        "ot_approvals.delegated_claim.warning.btn_confirm": "ot_approvals.delegated_claim.warning.btn_confirm",
        "ot_approvals.delegated_claim.empty_state.title": "ot_approvals.delegated_claim.empty_state.title",
        "ot_approvals.delegated_claim.default_reason": "Delegated OT claim",
        "common.submit": "common.submit",
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

vi.mock("@/hooks/use-workspace", () => ({
  useWorkspace: vi.fn(() => ({
    workspace: { id: "ws-123", name: "Test Workspace" },
    isLoading: false,
    isError: false,
  })),
}));

vi.mock("@/hooks/use-attendance", () => ({
  useAttendanceSummaries: vi.fn(),
  useApproveOt: vi.fn(),
  useEnterPaperOt: vi.fn(() => ({ enterPaperOt: vi.fn(), isSubmitting: false })),
  useBulkUploadPaperOt: vi.fn(() => ({ bulkUploadPaperOt: vi.fn(), isUploading: false })),
  useClaimOt: vi.fn(() => ({ claimOt: vi.fn(), isSubmitting: false })),
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...actual,
    fetchAttendanceReport: vi.fn(),
  };
});

const mockSummaries = [
  {
    id: "summary-1",
    employee_id: "EMP-001",
    date: "2026-05-01",
    claimed_ot_minutes: 120,
    system_detected_ot_minutes: 60,
    ot_reason: "Project deadline",
    entered_via: "self_service",
    ot_status: "pending_manager"
  },
  {
    id: "summary-2",
    employee_id: "EMP-002",
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
  let refetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    approveOtMock = vi.fn().mockResolvedValue({});
    refetchMock = vi.fn();
    
    vi.mocked(attendanceHooks.useApproveOt).mockReturnValue({
      approveOt: approveOtMock,
      isProcessing: false,
    } as unknown as ReturnType<typeof attendanceHooks.useApproveOt>);

    vi.mocked(attendanceHooks.useAttendanceSummaries).mockReturnValue({
      summaries: mockSummaries,
      isLoading: false,
      refetch: refetchMock,
    } as unknown as ReturnType<typeof attendanceHooks.useAttendanceSummaries>);
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

    expect(screen.getByText("EMP-001")).toBeInTheDocument();
    expect(screen.getAllByText("120m").length).toBeGreaterThan(0);
    expect(screen.getByText('"Project deadline"')).toBeInTheDocument();
    
    expect(screen.getByText("EMP-002")).toBeInTheDocument();
    expect(screen.getByText('"Covering shift"')).toBeInTheDocument();
  });

  it("calls approveOt and refetch when approve button is clicked", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByText("ot_approvals.tabs.post_ot"));
    
    const approveButtons = screen.getAllByTitle("ot_approvals.actions.approve");
    await user.click(approveButtons[0]);

    await waitFor(() => {
      expect(approveOtMock).toHaveBeenCalledWith({
        employeeId: "EMP-001",
        date: "2026-05-01",
        approved_minutes: 120,
        status: "approved",
        manager_note: "Approved via Manager Portal"
      });
      expect(refetchMock).toHaveBeenCalled();
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
        employeeId: "EMP-001",
        date: "2026-05-01",
        approved_minutes: 0,
        status: "rejected",
        manager_note: "Rejected via Manager Portal"
      });
      expect(refetchMock).toHaveBeenCalled();
    });
  });

  it("filters by source when filter buttons are clicked", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByText("ot_approvals.tabs.post_ot"));
    
    // Initially both should be visible
    expect(screen.getByText("EMP-001")).toBeInTheDocument();
    expect(screen.getByText("EMP-002")).toBeInTheDocument();
    
    // Click paper_form filter
    const paperFilter = screen.getByText("approvals.filter.paper_forms");
    await user.click(paperFilter);
    
    // Only EMP-002 (paper_form) should be visible
    expect(screen.queryByText("EMP-001")).not.toBeInTheDocument();
    expect(screen.getByText("EMP-002")).toBeInTheDocument();
  });
  
  it("shows empty state when no pending summaries", async () => {
    vi.mocked(attendanceHooks.useAttendanceSummaries).mockReturnValue({
      summaries: [],
      isLoading: false,
      refetch: refetchMock,
    } as unknown as ReturnType<typeof attendanceHooks.useAttendanceSummaries>);
    
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
            employee_id: "EMP-101",
            employee_name: "John Doe",
            employee_id_number: "JD101",
            date: "2026-05-10",
            computed_std_hours: 8,
            computed_ot_hours: 2.5,
            ot_claim_status: "pending_employee",
          },
          {
            employee_id: "EMP-102",
            employee_name: "Jane Smith",
            employee_id_number: "JS102",
            date: "2026-05-11",
            computed_std_hours: 8,
            computed_ot_hours: 1.5,
            ot_claim_status: "pending_employee",
          },
          {
            employee_id: "EMP-103",
            employee_name: "Bob Johnson",
            employee_id_number: "BJ103",
            date: "2026-05-12",
            computed_std_hours: 8,
            computed_ot_hours: 0,
            ot_claim_status: "pending_employee",
          },
          {
            employee_id: "EMP-104",
            employee_name: "Alice Brown",
            employee_id_number: "AB104",
            date: "2026-05-13",
            computed_std_hours: 8,
            computed_ot_hours: 3.0,
            ot_claim_status: "claimed",
          },
        ],
      } as any);
    });

    it("renders delegated claim tab button when user has overtime:claim_on_behalf", () => {
      renderPage();
      expect(screen.getByText("ot_approvals.tabs.delegated_claim")).toBeInTheDocument();
      expect(screen.queryByText("ot_approvals.tabs.pre_ot")).not.toBeInTheDocument();
      expect(screen.queryByText("ot_approvals.tabs.post_ot")).not.toBeInTheDocument();
    });

    it("renders list of claimable entries in table with correct hours", async () => {
      renderPage();

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
          employeeId: "EMP-101",
          date: "2026-05-10",
          claimed_minutes: 150,
          reason: "Finished inventory check",
        });
      });
    });

    it("submits bulk delegated claims when multiple rows are checked and bulk claim is clicked", async () => {
      const user = userEvent.setup();
      renderPage();

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
          employeeId: "EMP-101",
          date: "2026-05-10",
          claimed_minutes: 150,
          reason: "Delegated OT claim",
        });
        expect(claimOtMock).toHaveBeenNthCalledWith(2, {
          employeeId: "EMP-102",
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
          employeeId: "EMP-101",
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
          employeeId: "EMP-102",
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
});
