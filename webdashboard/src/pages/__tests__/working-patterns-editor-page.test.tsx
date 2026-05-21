import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import { WorkingPattern, Shift } from "../../lib/api/types";
import WorkingPatternsEditorPage from "../working-patterns-editor-page";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn().mockImplementation(() => vi.fn()),
  useParams: () => ({ patternId: undefined }),
  useBlocker: () => ({ status: "unblocked", proceed: vi.fn(), reset: vi.fn() }),
  useLocation: () => ({ pathname: "/working-patterns/new", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock API
vi.mock("../../lib/api", () => ({
  fetchWorkingPatterns: vi.fn(),
  createWorkingPattern: vi.fn(),
  updateWorkingPattern: vi.fn(),
  fetchPatternDetails: vi.fn(),
  fetchShifts: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
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
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0, staleTime: 0 },
      },
    });

    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("ws-1");
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
});
