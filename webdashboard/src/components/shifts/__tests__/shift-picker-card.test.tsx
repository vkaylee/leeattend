import { render, screen } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";

import { Shift } from "../../../lib/api";
import { ShiftPickerModal } from "../shift-picker-modal";

// ── Mocks ────────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      if (key === "shifts.picker.card.blocks") return `${String(opts?.count ?? 0)} blocks`;
      if (key === "shifts.picker.card.used_times") return `Used ${String(opts?.count ?? 0)} times`;
      if (key === "shifts.picker.card.work_hours") return "Paid";
      return key;
    },
    i18n: { language: "en" },
  }),
}));

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatTime: (time: string) => (time ? time.substring(0, 5) : ""),
  }),
}));

// ── Fixtures ─────────────────────────────────────────────────────────

type IntervalFixture = Partial<{
  break_duration_minutes: number;
  break_end: string | null;
  break_punch_mode: "none" | "required";
  break_start: string | null;
  end_time: string;
  sort_order: number;
  start_time: string;
}>;

function iv(overrides: IntervalFixture = {}) {
  return {
    break_duration_minutes: 0,
    break_end: null,
    break_punch_mode: "none" as const,
    break_start: null,
    end_day_offset: 0,
    end_time: "17:00:00",
    id: `iv-${Math.random().toString(36).slice(2, 8)}`,
    interval_name: null,
    sort_order: 0,
    start_day_offset: 0,
    start_time: "08:00:00",
    version_id: "v-1",
    ...overrides,
  };
}

function makeShift(id: string, name: string, intervals: ReturnType<typeof iv>[]): Shift {
  return {
    created_at: "2025-01-01T00:00:00Z",
    employer_id: null,
    id,
    intervals,
    is_active: true,
    location_id: null,
    name,
    parent_shift_id: null,
    report_symbol: null,
    updated_at: "2025-01-01T00:00:00Z",
    workspace_id: "ws-1",
  } as unknown as Shift;
}

const MORNING = makeShift("s1", "Morning Shift", [
  iv({
    break_duration_minutes: 60,
    break_end: "13:00:00",
    break_start: "12:00:00",
    end_time: "17:00:00",
    start_time: "08:00:00",
  }),
]);

const SPLIT = makeShift("s2", "Split Shift", [
  iv({ end_time: "12:00:00", sort_order: 0, start_time: "08:00:00" }),
  iv({ end_time: "18:00:00", sort_order: 1, start_time: "14:00:00" }),
]);

const NIGHT = makeShift("s3", "Night Shift", [
  iv({ end_time: "06:00:00", is_night_shift: true, start_time: "22:00:00" }),
]);

function renderModal(shifts: Shift[], onSelectShift = vi.fn()) {
  render(
    <ShiftPickerModal
      isOpen={true}
      onClose={vi.fn()}
      onSelectShift={onSelectShift}
      shifts={shifts}
      targetDates={["2025-07-23"]}
    />,
  );
  return { onSelectShift };
}

// ── Tests ────────────────────────────────────────────────────────────

describe("ShiftPickerCard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  describe("all intervals are rendered", () => {
    it("shows both blocks of a split shift, not just the first", () => {
      renderModal([SPLIT]);
      const card = screen.getByText("Split Shift").closest("button")!;

      expect(card).toHaveTextContent("08:00");
      expect(card).toHaveTextContent("12:00");
      // The second block was previously dropped entirely.
      expect(card).toHaveTextContent("14:00");
      expect(card).toHaveTextContent("18:00");
    });

    it("labels the block count for split shifts", () => {
      renderModal([SPLIT]);
      expect(screen.getByText("2 blocks")).toBeInTheDocument();
    });

    it("does not show a block count for single-block shifts", () => {
      renderModal([MORNING]);
      expect(screen.queryByText("1 blocks")).not.toBeInTheDocument();
    });
  });

  describe("paid duration", () => {
    it("subtracts a fixed break from the total (08:00-17:00 less 1h = 8h)", () => {
      renderModal([MORNING]);
      const card = screen.getByText("Morning Shift").closest("button")!;
      expect(card).toHaveTextContent("8h");
    });

    it("sums every block of a split shift (4h + 4h = 8h)", () => {
      renderModal([SPLIT]);
      const card = screen.getByText("Split Shift").closest("button")!;
      expect(card).toHaveTextContent("8h");
    });
  });

  describe("cross-midnight", () => {
    it("marks a night shift with a +1 day indicator", () => {
      renderModal([NIGHT]);
      const card = screen.getByText("Night Shift").closest("button")!;
      expect(card).toHaveTextContent("+1");
    });

    it("does not mark a same-day shift", () => {
      renderModal([MORNING]);
      const card = screen.getByText("Morning Shift").closest("button")!;
      expect(card).not.toHaveTextContent("+1");
    });
  });

  describe("accessible label", () => {
    it("carries name, times and paid hours so nothing relies on colour alone", () => {
      renderModal([SPLIT]);
      const card = screen.getByText("Split Shift").closest("button")!;
      const label = card.getAttribute("aria-label") ?? "";

      expect(label).toContain("Split Shift");
      expect(label).toContain("08:00–12:00");
      expect(label).toContain("14:00–18:00");
      expect(label).toContain("Paid: 8h");
    });
  });

  describe("usage count badge", () => {
    it("shows the count once a shift has been used more than once", () => {
      localStorage.setItem("leeattend_frequent_shifts", JSON.stringify({ s1: 4 }));
      renderModal([MORNING]);
      expect(screen.getByTitle("Used 4 times")).toBeInTheDocument();
    });

    it("stays hidden for a shift used only once", () => {
      localStorage.setItem("leeattend_frequent_shifts", JSON.stringify({ s1: 1 }));
      renderModal([MORNING]);
      expect(screen.queryByTitle("Used 1 times")).not.toBeInTheDocument();
    });
  });

  describe("keyboard navigation", () => {
    it("ArrowDown then Enter selects the first shift", async () => {
      const user = userEvent.setup();
      const { onSelectShift } = renderModal([MORNING, SPLIT, NIGHT]);
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");

      await user.type(input, "{ArrowDown}{Enter}");
      expect(onSelectShift).toHaveBeenCalledWith("s1");
    });

    it("ArrowDown twice moves the cursor to the second shift", async () => {
      const user = userEvent.setup();
      const { onSelectShift } = renderModal([MORNING, SPLIT, NIGHT]);
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");

      await user.type(input, "{ArrowDown}{ArrowDown}{Enter}");
      expect(onSelectShift).toHaveBeenCalledWith("s2");
    });

    it("ArrowUp from the top wraps to the last shift", async () => {
      const user = userEvent.setup();
      const { onSelectShift } = renderModal([MORNING, SPLIT, NIGHT]);
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");

      await user.type(input, "{ArrowUp}{Enter}");
      expect(onSelectShift).toHaveBeenCalledWith("s3");
    });

    it("Enter with no cursor picks the top result after typing", async () => {
      const user = userEvent.setup();
      const { onSelectShift } = renderModal([MORNING, SPLIT, NIGHT]);
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");

      await user.type(input, "Night{Enter}");
      expect(onSelectShift).toHaveBeenCalledWith("s3");
    });

    it("Enter does nothing when no shift matches", async () => {
      const user = userEvent.setup();
      const { onSelectShift } = renderModal([MORNING]);
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");

      await user.type(input, "ZZZ_NONE{Enter}");
      expect(onSelectShift).not.toHaveBeenCalled();
    });

    it("marks the cursor target with data-active", async () => {
      const user = userEvent.setup();
      renderModal([MORNING, SPLIT]);
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");

      await user.type(input, "{ArrowDown}");
      expect(screen.getByText("Morning Shift").closest("button")).toHaveAttribute("data-active");
      expect(screen.getByText("Split Shift").closest("button")).not.toHaveAttribute("data-active");
    });

    it("resets the cursor when the result set changes", async () => {
      const user = userEvent.setup();
      const { onSelectShift } = renderModal([MORNING, SPLIT, NIGHT]);
      const input = screen.getByPlaceholderText("shifts.picker.search_placeholder");

      // Park the cursor on the third card, then narrow the list to one result.
      await user.type(input, "{ArrowDown}{ArrowDown}{ArrowDown}");
      await user.type(input, "Split");
      await user.type(input, "{Enter}");

      // Enter must select the visible match, never the stale index-2 card.
      expect(onSelectShift).toHaveBeenCalledWith("s2");
    });
  });

  describe("shifts with no intervals", () => {
    it("renders without crashing and shows an em dash", () => {
      renderModal([makeShift("s9", "Empty Shift", [])]);
      const card = screen.getByText("Empty Shift").closest("button")!;
      expect(card).toBeInTheDocument();
      expect(card).toHaveTextContent("—");
    });
  });
});
