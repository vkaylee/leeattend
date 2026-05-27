import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as attendanceHooks from "../../hooks/use-attendance";
import OtApprovalsPage from "../approvals/ot-approvals-page";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "ot_approvals.title": "OT Approvals",
        "ot_approvals.subtitle": "Manage overtime",
        "ot_approvals.empty_state.title": "No pending OT",
        "ot_approvals.empty_state.description": "You are all caught up",
      };
      return translations[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

vi.mock("../../hooks/use-attendance", () => ({
  useAttendanceSummaries: vi.fn(),
  useApproveOt: vi.fn(),
  useEnterPaperOt: vi.fn(() => ({ enterPaperOt: vi.fn(), isSubmitting: false })),
  useBulkUploadPaperOt: vi.fn(() => ({ bulkUploadPaperOt: vi.fn(), isUploading: false })),
}));

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
});
