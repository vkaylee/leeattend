import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import { WorkingPattern, Shift } from "../../lib/api/types";
import WorkingPatternsEditorPage from "../working-patterns-editor-page";

// Mock TanStack Router
const mockProceed = vi.fn();
const mockReset = vi.fn();
let mockBlockerStatus = "unblocked";

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn().mockImplementation(() => vi.fn()),
  useParams: () => ({ patternId: undefined }),
  useBlocker: () => ({ status: mockBlockerStatus, proceed: mockProceed, reset: mockReset }),
  useLocation: () => ({ pathname: "/working-patterns/new", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock API
vi.mock("../../lib/api", () => ({
  fetchOrganization: vi.fn().mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC", default_language: "en" }),
  fetchWorkingPatterns: vi.fn(),
  createWorkingPattern: vi.fn(),
  updateWorkingPattern: vi.fn(),
  fetchPatternDetails: vi.fn(),
  fetchShifts: vi.fn(),
}));

// Mock Toast
const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();
vi.mock("../../components/ui/toast", () => ({
  useToast: () => ({
    success: mockToastSuccess,
    error: mockToastError,
    warning: vi.fn(),
  }),
}));

// Override i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, _options?: Record<string, unknown>) => {
      if (key === "patterns.form.name_placeholder") return "Enter pattern name";
      if (key === "patterns.modal.submit_btn") return "Save Pattern";
      if (key === "patterns.form.prefill_standard") return "Weekly Standard";
      return key;
    },
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("WorkingPatternsEditorPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    mockBlockerStatus = "unblocked";
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0, staleTime: 0 },
      },
    });

    vi.mocked(api.fetchShifts).mockResolvedValue([
      { id: "shift-1", name: "Morning Shift", start_time: "08:00", end_time: "17:00" }
    ] as unknown as Shift[]);
  });

  it("submits the create form with correct payload", async () => {
    const user = userEvent.setup();
    vi.mocked(api.createWorkingPattern).mockResolvedValue({ id: "pat-new" } as unknown as WorkingPattern);

    render(
      <QueryClientProvider client={queryClient}>
        <WorkingPatternsEditorPage />
      </QueryClientProvider>
    );

    // Fill pattern name
    const nameInput = await screen.findByPlaceholderText("Enter pattern name");
    await user.type(nameInput, "Weekly Rotation Plan");

    // Click submit/save
    const submitBtn = screen.getByText("SAVE PATTERN");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.createWorkingPattern).toHaveBeenCalledWith(
        "ws-1",
        expect.objectContaining({
          name: "Weekly Rotation Plan",
          pattern_type: "weekly",
          details: expect.any(Array) as unknown as unknown[],
        })
      );
    });

    expect(mockToastSuccess).toHaveBeenCalled();
  });

  it("allows switching view mode and rendering rotation schedule layout", async () => {
    const user = userEvent.setup();
    render(
      <QueryClientProvider client={queryClient}>
        <WorkingPatternsEditorPage />
      </QueryClientProvider>
    );

    // Switch to rotation pattern
    const rotationBtn = await screen.findByText("patterns.form.pattern_type_rotation");
    await user.click(rotationBtn);

    // Verify Design and Preview tabs are visible
    const designTab = await screen.findByText("patterns.form.view_mode_design");
    const previewTab = await screen.findByText("patterns.form.view_mode_preview");
    expect(designTab).toBeInTheDocument();
    expect(previewTab).toBeInTheDocument();

    // Verify day labels are shown
    const dayLabels = await screen.findAllByText("patterns.form.day_index");
    expect(dayLabels.length).toBeGreaterThan(0);

    // Switch to preview mode
    await user.click(previewTab);

    // Verify anchor date input and month labels are shown
    const anchorLabel = await screen.findByText("patterns.form.anchor_date_preview");
    expect(anchorLabel).toBeInTheDocument();
    const monthLabel = await screen.findByText("patterns.form.preview_month");
    expect(monthLabel).toBeInTheDocument();

    // Cycle day markers D1, D2, etc. should be shown on the preview calendar
    const d1Markers = await screen.findAllByText("D1");
    expect(d1Markers.length).toBeGreaterThan(0);
  });

  it("opens ConfirmDialog when navigation is blocked, and proceeds or resets on confirm/cancel", async () => {
    const user = userEvent.setup();
    mockBlockerStatus = "blocked";

    render(
      <QueryClientProvider client={queryClient}>
        <WorkingPatternsEditorPage />
      </QueryClientProvider>
    );

    // Verify warning dialog is open
    const warningTitle = await screen.findByText("patterns.form.unsaved_changes");
    expect(warningTitle).toBeInTheDocument();

    // Verify stay (cancel) button and leave (confirm) button
    const stayBtn = screen.getByText("patterns.form.unsaved_changes_cancel");
    const leaveBtn = screen.getByText("patterns.form.unsaved_changes_confirm");
    expect(stayBtn).toBeInTheDocument();
    expect(leaveBtn).toBeInTheDocument();

    // Click stay (cancel)
    await user.click(stayBtn);
    expect(mockReset).toHaveBeenCalled();

    // Click leave (confirm)
    await user.click(leaveBtn);
    expect(mockProceed).toHaveBeenCalled();
  });

  it("opens ConfirmDialog when clearing shifts, and clears when confirmed", async () => {
    const user = userEvent.setup();
    render(
      <QueryClientProvider client={queryClient}>
        <WorkingPatternsEditorPage />
      </QueryClientProvider>
    );

    // Prefill some shifts first
    const prefillBtn = await screen.findByText("WEEKLY STANDARD");
    await user.click(prefillBtn);

    // Verify shift is rendered in document (Mon-Fri)
    const morningShiftElements = await screen.findAllByText("Morning Shift");
    expect(morningShiftElements.length).toBeGreaterThan(0);

    // Click clear all
    const clearBtn = await screen.findByRole("button", { name: /clear_all/i });
    await user.click(clearBtn);

    // Verify confirm dialog is open
    const dialogMessage = await screen.findByText("patterns.form.clear_all_confirm");
    expect(dialogMessage).toBeInTheDocument();

    const cancelBtn = screen.getByText("common.cancel");
    const confirmBtn = screen.getByText("common.confirm");
    expect(cancelBtn).toBeInTheDocument();
    expect(confirmBtn).toBeInTheDocument();

    // Click cancel first
    await user.click(cancelBtn);
    expect(screen.queryByText("patterns.form.clear_all_confirm")).not.toBeInTheDocument();
    
    const isMorningShiftAssigned = () => {
      const elements = screen.queryAllByText("Morning Shift");
      return elements.some(el => el.tagName.toLowerCase() === "span");
    };
    expect(isMorningShiftAssigned()).toBe(true);

    // Click clear all again
    await user.click(clearBtn);

    // Click confirm
    const confirmBtn2 = await screen.findByText("common.confirm");
    await user.click(confirmBtn2);

    // Verify dialog is closed and shifts are cleared
    expect(screen.queryByText("patterns.form.clear_all_confirm")).not.toBeInTheDocument();
    expect(isMorningShiftAssigned()).toBe(false);
  });
});
