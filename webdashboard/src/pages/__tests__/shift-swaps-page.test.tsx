import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import * as shiftSwapsApi from "../../lib/api/shift-swaps";
import { PaginatedResponse, Employee } from "../../lib/api/types";
import ShiftSwapsPage from "../shift-swaps-page";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "shift_swaps.title": "Shift Swaps",
        "shift_swaps.subtitle": "Employee shift swap management",
        "shift_swaps.create_btn": "Create Swap",
        "shift_swaps.requester": "Requester",
        "shift_swaps.target": "Target",
        "shift_swaps.select_employee": "Select employee",
        "shift_swaps.swap_date": "Swap Date",
        "shift_swaps.reason": "Reason",
        "shift_swaps.reason_placeholder": "Optional reason",
        "shift_swaps.day_off": "Day Off",
        "shift_swaps.guide.title": "How it works",
        "shift_swaps.guide.intro": "Shift swaps allow employees to exchange shifts",
        "shift_swaps.guide.step1.title": "Step 1",
        "shift_swaps.guide.step1.desc": "Create a swap request",
        "shift_swaps.guide.step2.title": "Step 2",
        "shift_swaps.guide.step2.desc": "Target accepts",
        "shift_swaps.guide.step3.title": "Step 3",
        "shift_swaps.guide.step3.desc": "Admin approves",
        "shift_swaps.guide.notes_title": "Important",
        "shift_swaps.guide.note1": "Note 1",
        "shift_swaps.guide.note2": "Note 2",
        "shift_swaps.guide.note3": "Note 3",
        "shift_swaps.stats.pending": "Pending",
        "shift_swaps.stats.approved_month": "Approved this month",
        "shift_swaps.stats.rejected_month": "Rejected this month",
        "shift_swaps.stats.total": "Total",
        "shift_swaps.status.pending_target": "Pending Target",
        "shift_swaps.status.pending_admin": "Pending Admin",
        "shift_swaps.status.approved": "Approved",
        "shift_swaps.status.rejected": "Rejected",
        "shift_swaps.status.cancelled": "Cancelled",
        "shift_swaps.messages.no_data": "No swap requests found",
        "shift_swaps.messages.create_success": "Swap created",
        "shift_swaps.messages.create_error": "Failed to create swap",
        "common.cancel": "Cancel",
        "common.submit": "Submit",
        "common.save": "Save",
        "common.loading": "Loading...",
        "common.status": "Status",
        "common.success": "Success",
        "common.approve": "Approve",
        "common.reject": "Reject",
        "common.no_notes": "No notes",
        "common.select_workspace": "Select workspace",
      };
      return translations[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

// Mock useDateTimeFormatter
vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatDate: (date: string) => date,
    formatTime: (time: string) => time,
    formatDateTime: (datetime: string) => datetime,
  }),
}));

// Mock APIs
vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

vi.mock("../../lib/api/shift-swaps", () => ({
  fetchShiftSwaps: vi.fn(),
  createShiftSwap: vi.fn(),
  approveShiftSwap: vi.fn(),
  fetchShiftSwapStats: vi.fn(),
}));

const mockEmployees: PaginatedResponse<Employee> = {
  data: [
    { id: "emp-1", full_name: "Alice Johnson", employee_id_number: "EMP001", workspace_id: "ws-1", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", tags: [] },
    { id: "emp-2", full_name: "Bob Williams", employee_id_number: "EMP002", workspace_id: "ws-1", created_at: "2025-02-01T00:00:00Z", updated_at: "2025-02-01T00:00:00Z", tags: [] },
  ] as Employee[],
  total: 2,
  page: 1,
  limit: 3000,
};

const mockStats: shiftSwapsApi.SwapStats = {
  pending_target: 1,
  pending_admin: 2,
  approved: 5,
  rejected: 1,
  cancelled: 0,
  approved_this_month: 3,
  rejected_this_month: 1,
};

const mockSwaps: shiftSwapsApi.ShiftSwapRequest[] = [
  {
    id: "swap-1",
    workspace_id: "ws-1",
    requester_id: "emp-1",
    target_id: "emp-2",
    swap_date: "2026-05-10",
    requester_shift_id: "shift-1",
    target_shift_id: "shift-2",
    status: "pending_admin",
    reason: "Family event",
    requested_by_user_id: null,
    approved_by_user_id: null,
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    requester_name: "Alice Johnson",
    target_name: "Bob Williams",
    requester_shift_name: "Morning",
    target_shift_name: "Afternoon",
  },
];

describe("ShiftSwapsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("ws-1");
    vi.mocked(api.fetchEmployees).mockResolvedValue(mockEmployees);
    vi.mocked(shiftSwapsApi.fetchShiftSwaps).mockResolvedValue(mockSwaps);
    vi.mocked(shiftSwapsApi.fetchShiftSwapStats).mockResolvedValue(mockStats);
    vi.mocked(shiftSwapsApi.createShiftSwap).mockResolvedValue(mockSwaps[0]);
    vi.mocked(shiftSwapsApi.approveShiftSwap).mockResolvedValue(undefined);
  });

  const renderPage = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <ShiftSwapsPage />
        </ToastProvider>
      </QueryClientProvider>,
    );

  // ── Render Tests ──────────────────────────────────────────

  it("renders the page title and subtitle", async () => {
    renderPage();
    expect(await screen.findByText("Shift Swaps")).toBeInTheDocument();
    expect(screen.getByText("Employee shift swap management")).toBeInTheDocument();
  });

  it("renders the create button", async () => {
    renderPage();
    expect(await screen.findByText("Create Swap")).toBeInTheDocument();
  });

  it("renders stats bar with correct values", async () => {
    renderPage();
    // Pending Admin = 2
    // Approved = 5
    const twos = await screen.findAllByText("2");
    expect(twos.length).toBeGreaterThanOrEqual(1);

    const fives = await screen.findAllByText("5");
    expect(fives.length).toBeGreaterThanOrEqual(1);
  });

  it("renders swap table with data", async () => {
    renderPage();
    expect(await screen.findByText("Alice Johnson")).toBeInTheDocument();
    expect(screen.getByText("Bob Williams")).toBeInTheDocument();
    expect(screen.getByText("Family event")).toBeInTheDocument();
  });

  it("shows empty state when no swaps", async () => {
    vi.mocked(shiftSwapsApi.fetchShiftSwaps).mockResolvedValue([]);
    renderPage();
    expect(await screen.findByText("No swap requests found")).toBeInTheDocument();
  });

  it("renders status tabs", async () => {
    renderPage();
    expect(await screen.findByText("Pending Target")).toBeInTheDocument();
    expect(screen.getAllByText("Pending Admin").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Approved").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Rejected").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Cancelled").length).toBeGreaterThan(0);
  });

  // ── Interaction Tests ─────────────────────────────────────

  it("switches status tabs and refetches data", async () => {
    const user = userEvent.setup();
    renderPage();

    const approvedTabs = await screen.findAllByText("Approved");
    await user.click(approvedTabs[0]);

    await waitFor(() => {
      expect(shiftSwapsApi.fetchShiftSwaps).toHaveBeenCalledWith("ws-1", "approved");
    });
  });

  it("toggles create form when button is clicked", async () => {
    const user = userEvent.setup();
    renderPage();

    const createButton = await screen.findByText("Create Swap");
    await user.click(createButton);

    expect(screen.getAllByText("Requester").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Target").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Swap Date").length).toBeGreaterThan(0);
  });

  it("shows guide section when toggle is clicked", async () => {
    const user = userEvent.setup();
    renderPage();

    const guideButton = await screen.findByText("How it works");
    await user.click(guideButton);

    expect(screen.getByText("Step 1")).toBeInTheDocument();
    expect(screen.getByText("Step 2")).toBeInTheDocument();
    expect(screen.getByText("Step 3")).toBeInTheDocument();
  });

  it("shows approve/reject buttons on hover for pending_admin items", async () => {
    renderPage();
    // Action buttons exist in DOM (hidden via CSS opacity)
    await waitFor(() => {
      const approveButton = screen.getByTitle("Approve");
      expect(approveButton).toBeInTheDocument();
    });
  });

  it("calls approveShiftSwap when approve button is clicked", async () => {
    const user = userEvent.setup();
    renderPage();

    const approveButton = await screen.findByTitle("Approve");
    await user.click(approveButton);

    await waitFor(() => {
      expect(shiftSwapsApi.approveShiftSwap).toHaveBeenCalledWith(
        "ws-1",
        "swap-1",
        { status: "approved" },
      );
    });
  });

  it("calls approveShiftSwap with rejected status when reject is clicked", async () => {
    const user = userEvent.setup();
    renderPage();

    const rejectButton = await screen.findByTitle("Reject");
    await user.click(rejectButton);

    await waitFor(() => {
      expect(shiftSwapsApi.approveShiftSwap).toHaveBeenCalledWith(
        "ws-1",
        "swap-1",
        { status: "rejected" },
      );
    });
  });

  // ── Edge Cases ────────────────────────────────────────────

  it("shows workspace selection prompt when no workspace", async () => {
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue(null);
    renderPage();
    expect(await screen.findByText("Select workspace")).toBeInTheDocument();
  });

  it("does not submit when form fields are empty", async () => {
    const user = userEvent.setup();
    renderPage();

    const createButton = await screen.findByText("Create Swap");
    await user.click(createButton);

    const submitButton = screen.getByText("Save");
    expect(submitButton.closest("button")).toBeDisabled();
  });
});
