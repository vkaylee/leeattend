import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import "@testing-library/jest-dom";

import type { ShiftVersionHistoryItem } from "@/lib/api/types";

import { ShiftRevisionHistoryDialog } from "../shift-revision-history-dialog";

// ── Mocks ───────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (params?.name) return `${key}_${String(params.name)}`;
      if (params?.count !== undefined) return `${key}_${String(params.count)}`;
      return key;
    },
    i18n: { language: "en" },
  }),
}));

const getShiftVersions = vi.fn();
vi.mock("@/lib/api/shifts", () => ({
  getShiftVersions: (...args: unknown[]) => getShiftVersions(...args),
}));

// ── Helpers ─────────────────────────────────────────────────────────

function makeVersion(overrides: Partial<ShiftVersionHistoryItem> = {}): ShiftVersionHistoryItem {
  return {
    change_reason: null,
    change_type: "create",
    created_at: "2026-07-20T08:00:00Z",
    created_by: "user-1",
    effective_from: "2026-07-01",
    effective_to: null,
    employer_id: null,
    id: "v1",
    intervals: [
      {
        allow_early_ot: false,
        break_duration_minutes: 60,
        break_end: null,
        break_is_overtime: false,
        break_start: null,
        created_at: null,
        early_punch_in_grace_period_minutes: 15,
        end_time: "17:00:00",
        id: "iv1",
        interval_name: null,
        is_break_auto_deduct: true,
        is_night_shift: false,
        max_early_ot_minutes: null,
        ot_aggregation_mode: "aggregate",
        shift_id: "v1",
        sort_order: 0,
        start_time: "08:00:00",
        updated_at: null,
      },
    ],
    location_id: null,
    name: "Office Shift",
    report_symbol: "OS",
    superseded_by: null,
    template_id: "t1",
    version_number: 1,
    ...overrides,
  };
}

function renderDialog(overrides: Partial<React.ComponentProps<typeof ShiftRevisionHistoryDialog>> = {}) {
  const onClose = vi.fn();
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  render(
    <QueryClientProvider client={queryClient}>
      <ShiftRevisionHistoryDialog
        isOpen
        onClose={onClose}
        shiftId="shift-1"
        shiftName="Office Shift"
        wsId="ws-1"
        {...overrides}
      />
    </QueryClientProvider>,
  );
  return { onClose };
}

// ── Tests ───────────────────────────────────────────────────────────

describe("ShiftRevisionHistoryDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default: pending query (no resolve yet)
    getShiftVersions.mockImplementation(
      () => new Promise(() => { /* never resolves → loading state */ }),
    );
  });

  it("shows skeleton loader while fetching versions", () => {
    renderDialog();
    // The 3 skeleton bars should have animate-pulse
    const skeletonCards = document.querySelectorAll(".animate-pulse");
    expect(skeletonCards.length).toBeGreaterThan(0);
  });

  it("shows empty state when no versions exist", async () => {
    getShiftVersions.mockResolvedValue([]);
    renderDialog();

    await waitFor(() => {
      expect(screen.getByText("shifts.revisions.empty_state")).toBeDefined();
    });
  });

  it("renders version cards in timeline when versions exist", async () => {
    getShiftVersions.mockResolvedValue([
      makeVersion({ id: "v3", version_number: 3, change_type: "revise", change_reason: "Updated break time" }),
      makeVersion({ id: "v2", version_number: 2, change_type: "correct", change_reason: "Fixed typo" }),
      makeVersion({ id: "v1", version_number: 1, change_type: "create" }),
    ]);
    renderDialog();

    await waitFor(() => {
      expect(screen.getByText("v3")).toBeDefined();
      expect(screen.getByText("v2")).toBeDefined();
      expect(screen.getByText("v1")).toBeDefined();
    });

    // Badges
    expect(screen.getByText("shifts.revisions.badge_revision")).toBeDefined();
    expect(screen.getByText("shifts.revisions.badge_correction")).toBeDefined();
    expect(screen.getByText("shifts.revisions.badge_create")).toBeDefined();

    // Active version badge on latest (v3)
    expect(screen.getByText("shifts.revisions.active_version")).toBeDefined();
  });

  it("shows change_reason quote block when present", async () => {
    getShiftVersions.mockResolvedValue([
      makeVersion({ id: "v1", version_number: 1, change_reason: "Initial setup for Q3" }),
    ]);
    renderDialog();

    await waitFor(() => {
      expect(screen.getByText("Initial setup for Q3")).toBeDefined();
    });
  });

  it("expands and collapses diff section on click", async () => {
    const user = userEvent.setup();

    getShiftVersions.mockResolvedValue([
      makeVersion({
        change_type: "revise",
        id: "v2",
        intervals: [
          {
            allow_early_ot: false,
            break_duration_minutes: 30,
            break_end: null,
            break_is_overtime: false,
            break_start: null,
            created_at: null,
            early_punch_in_grace_period_minutes: 15,
            end_time: "18:00:00",
            id: "iv2",
            interval_name: null,
            is_break_auto_deduct: true,
            is_night_shift: false,
            max_early_ot_minutes: null,
            ot_aggregation_mode: "aggregate",
            shift_id: "v2",
            sort_order: 0,
            start_time: "08:00:00",
            updated_at: null,
          },
        ],
        version_number: 2,
      }),
      makeVersion({
        change_type: "create",
        id: "v1",
        intervals: [
          {
            allow_early_ot: false,
            break_duration_minutes: 60,
            break_end: null,
            break_is_overtime: false,
            break_start: null,
            created_at: null,
            early_punch_in_grace_period_minutes: 15,
            end_time: "17:00:00",
            id: "iv1",
            interval_name: null,
            is_break_auto_deduct: true,
            is_night_shift: false,
            max_early_ot_minutes: null,
            ot_aggregation_mode: "aggregate",
            shift_id: "v1",
            sort_order: 0,
            start_time: "08:00:00",
            updated_at: null,
          },
        ],
        version_number: 1,
      }),
    ]);
    renderDialog();

    // Wait for data to load
    await waitFor(() => {
      expect(screen.getByText("v2")).toBeDefined();
    });

    // Click "Show changes" button
    const showDiffBtn = screen.getByText("shifts.revisions.show_diff");
    await user.click(showDiffBtn);

    // After expand, should see "Hide changes"
    await waitFor(() => {
      expect(screen.getByText("shifts.revisions.hide_diff")).toBeDefined();
    });

    // Click "Hide changes" to collapse
    const hideDiffBtn = screen.getByText("shifts.revisions.hide_diff");
    await user.click(hideDiffBtn);

    await waitFor(() => {
      expect(screen.getByText("shifts.revisions.show_diff")).toBeDefined();
    });
  });

  it("calls onClose when close button is clicked", async () => {
    const user = userEvent.setup();
    getShiftVersions.mockResolvedValue([
      makeVersion({ id: "v1", version_number: 1 }),
    ]);
    const { onClose } = renderDialog();

    await waitFor(() => {
      expect(screen.getByText("v1")).toBeDefined();
    });

    // Click the "Close" button in the footer
    const closeBtn = screen.getByText("common.close");
    await user.click(closeBtn);

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("does not fetch when dialog is closed", () => {
    renderDialog({ isOpen: false });
    // getShiftVersions should not be called because query is disabled
    expect(getShiftVersions).not.toHaveBeenCalled();
  });
});
