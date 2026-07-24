import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";

import { Shift } from "../../../lib/api";
import { ShiftPickerModal } from "../shift-picker-modal";

// ── Mocks ────────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      if (key === "shifts.modal.select_shift_subtitle") return `Select shift for ${opts?.count ?? 0} days`;
      return key;
    },
    i18n: { language: "en" },
  }),
}));

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatTime: (time: string) => time ? time.substring(0, 5) : "",
  }),
}));

// ── Helpers ──────────────────────────────────────────────────────────

function makeShift(overrides: Partial<Shift> & {
  intervals?: {
    start_time?: string;
    end_time?: string;
    break_start?: string | null;
    break_end?: string | null;
    break_duration_minutes?: number;
    is_break_auto_deduct?: boolean;
    is_night_shift?: boolean;
    interval_name?: string;
  };
} = {}): Shift {
  const id = overrides.id ?? `shift-${Math.random().toString(36).slice(2, 8)}`;
  return {
    id,
    name: overrides.name ?? "Morning Shift",
    report_symbol: overrides.report_symbol ?? null,
    is_active: overrides.is_active ?? true,
    intervals: overrides.intervals ?? [{
      start_time: "08:00:00",
      end_time: "17:00:00",
      break_start: "12:00:00",
      break_end: "13:00:00",
      break_duration_minutes: 60,
      is_break_auto_deduct: true,
      is_night_shift: false,
    }],
    employer_id: overrides.employer_id ?? null,
    location_id: overrides.location_id ?? null,
    workspace_id: overrides.workspace_id ?? "ws-1",
    created_at: overrides.created_at ?? "2025-01-01T00:00:00Z",
    updated_at: overrides.updated_at ?? "2025-01-01T00:00:00Z",
    parent_shift_id: overrides.parent_shift_id ?? null,
  } as unknown as Shift;
}

const defaultShifts: Shift[] = [
  makeShift({ id: "s1", name: "Morning Shift", report_symbol: "MS", intervals: [{ start_time: "08:00:00", end_time: "17:00:00", break_duration_minutes: 60, is_break_auto_deduct: true, is_night_shift: false }] }),
  makeShift({ id: "s2", name: "Night Shift", report_symbol: "NS", intervals: [{ start_time: "22:00:00", end_time: "06:00:00", break_duration_minutes: 30, is_break_auto_deduct: false, is_night_shift: true }] }),
  makeShift({ id: "s3", name: "Split Shift", report_symbol: "SP", intervals: [
    { start_time: "08:00:00", end_time: "12:00:00", break_duration_minutes: 0, is_break_auto_deduct: false, is_night_shift: false },
    { start_time: "14:00:00", end_time: "18:00:00", break_duration_minutes: 0, is_break_auto_deduct: false, is_night_shift: false },
  ] }),
  makeShift({ id: "s4", name: "Afternoon Shift", report_symbol: "AS", intervals: [{ start_time: "14:00:00", end_time: "22:00:00", break_duration_minutes: 30, is_break_auto_deduct: true, is_night_shift: false }] }),
];

// ── Tests ────────────────────────────────────────────────────────────

describe("ShiftPickerModal", () => {
  const onClose = vi.fn();
  const onSelectShift = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  function renderModal(shifts = defaultShifts) {
    return render(
      <ShiftPickerModal
        isOpen={true}
        onClose={onClose}
        onSelectShift={onSelectShift}
        shifts={shifts}
        targetDates={["2025-07-23"]}
      />
    );
  }

  describe("rendering", () => {
    it("renders all shifts by default", () => {
      renderModal();
      expect(screen.getByText("Morning Shift")).toBeInTheDocument();
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
      expect(screen.getByText("Split Shift")).toBeInTheDocument();
      expect(screen.getByText("Afternoon Shift")).toBeInTheDocument();
    });

    it("renders the search input", () => {
      renderModal();
      expect(screen.getByPlaceholderText("shifts.picker.search_placeholder")).toBeInTheDocument();
    });

    it("renders filter chips", () => {
      renderModal();
      expect(screen.getByText("shifts.picker.filter_all")).toBeInTheDocument();
      // filter_night/filter_split may appear in both chip + badge; use getAllByText
      expect(screen.getAllByText("shifts.picker.filter_night").length).toBeGreaterThanOrEqual(1);
      expect(screen.getByText("shifts.picker.filter_single")).toBeInTheDocument();
      expect(screen.getAllByText("shifts.picker.filter_split").length).toBeGreaterThanOrEqual(1);
    });

    it("hides frequent chip when no frequent shifts", () => {
      renderModal();
      expect(screen.queryByText("shifts.picker.filter_frequent")).not.toBeInTheDocument();
    });

    it("renders OFF day button", () => {
      renderModal();
      expect(screen.getByText("shifts.off_day")).toBeInTheDocument();
    });

    it("renders cancel button", () => {
      renderModal();
      expect(screen.getByText("common.cancel")).toBeInTheDocument();
    });
  });

  describe("search filtering", () => {
    it("filters shifts by name", async () => {
      const user = userEvent.setup();
      renderModal();
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");
      await user.type(input, "Night");
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
      expect(screen.queryByText("Morning Shift")).not.toBeInTheDocument();
      expect(screen.queryByText("Split Shift")).not.toBeInTheDocument();
    });

    it("filters shifts by report symbol", async () => {
      const user = userEvent.setup();
      renderModal();
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");
      await user.type(input, "NS");
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
      expect(screen.queryByText("Morning Shift")).not.toBeInTheDocument();
    });

    it("filters shifts by interval time", async () => {
      const user = userEvent.setup();
      renderModal();
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");
      await user.type(input, "22:00");
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
      expect(screen.getByText("Afternoon Shift")).toBeInTheDocument(); // ends at 22:00
      expect(screen.queryByText("Morning Shift")).not.toBeInTheDocument();
    });

    it("is case-insensitive", async () => {
      const user = userEvent.setup();
      renderModal();
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");
      await user.type(input, "night");
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
    });

    it("shows empty state when no matches", async () => {
      const user = userEvent.setup();
      renderModal();
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");
      await user.type(input, "ZZZ_NONEXISTENT");
      expect(screen.getByText("shifts.picker.no_results_title")).toBeInTheDocument();
      expect(screen.getByText("shifts.picker.no_results_desc")).toBeInTheDocument();
      expect(screen.getByText("shifts.picker.clear_search")).toBeInTheDocument();
    });
  });

  describe("filter chips", () => {
    it("filters to night shifts only", async () => {
      const user = userEvent.setup();
      renderModal();
      // Click the filter chip button (has aria-pressed), not the badge
      const nightChip = screen.getAllByText("shifts.picker.filter_night")
        .map(el => el.closest("button"))
        .find(btn => btn?.getAttribute("aria-pressed") !== null);
      expect(nightChip).toBeTruthy();
      await user.click(nightChip!);
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
      expect(screen.queryByText("Morning Shift")).not.toBeInTheDocument();
      expect(screen.queryByText("Split Shift")).not.toBeInTheDocument();
    });

    it("filters to single-block shifts only", async () => {
      const user = userEvent.setup();
      renderModal();
      await user.click(screen.getByText("shifts.picker.filter_single"));
      expect(screen.getByText("Morning Shift")).toBeInTheDocument();
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
      expect(screen.getByText("Afternoon Shift")).toBeInTheDocument();
      expect(screen.queryByText("Split Shift")).not.toBeInTheDocument();
    });

    it("filters to split shifts only", async () => {
      const user = userEvent.setup();
      renderModal();
      const splitChip = screen.getAllByText("shifts.picker.filter_split")
        .map(el => el.closest("button"))
        .find(btn => btn?.getAttribute("aria-pressed") !== null);
      expect(splitChip).toBeTruthy();
      await user.click(splitChip!);
      expect(screen.getByText("Split Shift")).toBeInTheDocument();
      expect(screen.queryByText("Morning Shift")).not.toBeInTheDocument();
      expect(screen.queryByText("Night Shift")).not.toBeInTheDocument();
    });

    it("returns to all when all chip is clicked", async () => {
      const user = userEvent.setup();
      renderModal();
      // Click night chip first
      const nightChip = screen.getAllByText("shifts.picker.filter_night")
        .map(el => el.closest("button"))
        .find(btn => btn?.getAttribute("aria-pressed") !== null);
      await user.click(nightChip!);
      expect(screen.queryByText("Morning Shift")).not.toBeInTheDocument();
      // Click all chip
      await user.click(screen.getByText("shifts.picker.filter_all"));
      expect(screen.getByText("Morning Shift")).toBeInTheDocument();
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
    });
  });

  describe("frequently used section", () => {
    it("hides frequent chip when localStorage is empty", () => {
      renderModal();
      expect(screen.queryByText("shifts.picker.filter_frequent")).not.toBeInTheDocument();
    });

    it("frequent chip filters to only frequent shifts", async () => {
      const user = userEvent.setup();
      localStorage.setItem("leeattend_frequent_shifts", JSON.stringify({ s2: 5, s1: 3 }));

      renderModal();
      // After rendering with localStorage data, the frequent chip should appear
      // (useEffect reads localStorage on mount)
      await waitFor(() => {
        expect(screen.getByText("shifts.picker.filter_frequent")).toBeInTheDocument();
      }, { timeout: 3000 });

      await user.click(screen.getByText("shifts.picker.filter_frequent"));

      expect(screen.getByText("Night Shift")).toBeInTheDocument();
      expect(screen.getByText("Morning Shift")).toBeInTheDocument();
      expect(screen.queryByText("Split Shift")).not.toBeInTheDocument();
      expect(screen.queryByText("Afternoon Shift")).not.toBeInTheDocument();
    });
  });

  describe("keyboard interaction", () => {
    it("clears search on Escape when search has value", async () => {
      const user = userEvent.setup();
      renderModal();
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");
      await user.type(input, "Night");
      expect(input).toHaveValue("Night");

      fireEvent.keyDown(input, { key: "Escape" });
      expect(input).toHaveValue("");
    });

    it("calls onSelectShift when a shift button is clicked", async () => {
      const user = userEvent.setup();
      renderModal();
      await user.click(screen.getByText("Morning Shift").closest("button")!);
      expect(onSelectShift).toHaveBeenCalledWith("s1");
    });

    it("calls onSelectShift with null for OFF day", async () => {
      const user = userEvent.setup();
      renderModal();
      await user.click(screen.getByText("shifts.off_day").closest("button")!);
      expect(onSelectShift).toHaveBeenCalledWith(null);
    });

    it("clear search button resets search", async () => {
      const user = userEvent.setup();
      renderModal();
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");
      await user.type(input, "Night");

      // Click clear search in empty state (shouldn't exist when there are results)
      // Instead, look for the X clear button in SearchBox
      const clearBtn = screen.getByLabelText("common.clear");
      await user.click(clearBtn);

      expect(input).toHaveValue("");
      // All shifts should be back
      expect(screen.getByText("Morning Shift")).toBeInTheDocument();
      expect(screen.getByText("Night Shift")).toBeInTheDocument();
    });
  });

  describe("localStorage persistence", () => {
    it("writes selected shift to localStorage on selection", async () => {
      const user = userEvent.setup();
      renderModal();

      const morningBtn = screen.getByText("Morning Shift").closest("button")!;
      await user.click(morningBtn);

      // handleSelect should have called onSelectShift
      expect(onSelectShift).toHaveBeenCalledWith("s1");

      // localStorage should have been written
      const raw = localStorage.getItem("leeattend_frequent_shifts");
      expect(raw).not.toBeNull();
      const stored = JSON.parse(raw!);
      expect(stored.s1).toBeGreaterThanOrEqual(1);
    });

    it("increments count for repeated selections", async () => {
      const user = userEvent.setup();

      // Pre-populate localStorage
      localStorage.setItem("leeattend_frequent_shifts", JSON.stringify({ s1: 3 }));

      renderModal();
      const morningBtn = screen.getByText("Morning Shift").closest("button")!;
      await user.click(morningBtn);

      const stored = JSON.parse(localStorage.getItem("leeattend_frequent_shifts")!);
      expect(stored.s1).toBe(4);
    });
  });

  describe("empty state", () => {
    it("shows empty state when shifts array is empty", () => {
      renderModal([]);
      expect(screen.getByText("shifts.picker.no_results_title")).toBeInTheDocument();
      expect(screen.getByText("shifts.picker.no_results_desc")).toBeInTheDocument();
    });

    it("clear search button in empty state resets filters", async () => {
      const user = userEvent.setup();
      renderModal([]);

      const clearBtn = screen.getByText("shifts.picker.clear_search");
      await user.click(clearBtn);

      // Empty state should still show (no shifts available)
      expect(screen.getByText("shifts.picker.no_results_title")).toBeInTheDocument();
    });
  });
});
