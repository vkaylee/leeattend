import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi } from "vitest";

import { DailyAttendanceReport } from "@/lib/api";

import { DailyReportTimeline, TimeEntry } from "../daily-report-timeline";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

// Mock useWorkspace — timeline now reads workspace.timezone for correct
// punch positioning in the workspace timezone (Excel parity).
vi.mock("@/hooks/use-workspace", () => ({
    useWorkspace: () => ({
        workspace: { timezone: "UTC" },
    }),
}));

// Mock lucide-react
vi.mock("lucide-react", () => ({
    RefreshCw: () => null,
    Bug: () => null,
    CheckCircle: () => null,
    ThumbsUp: () => null,
    ThumbsDown: () => null,
    ChevronRight: () => null,
}));

// Mock ScheduleTracePopover
vi.mock("../../../shifts/schedule-trace-popover", () => ({
    ScheduleTracePopover: () => null,
}));

// Use a standard non-DST date for deterministic timezone behavior
const BASE_DATE = "2024-06-15"; // June — no DST transition

/** Build an ISO datetime string from a date and a local HH:MM time */
function iso(date: string, hhmm: string): string {
    const [h, m] = hhmm.split(":").map(Number);
    return `${date}T${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:00`;
}

describe("DailyReportTimeline — Axis filtering", () => {
    const formatTime = (t: string | null) => t ?? "---";
    const t = (k: string) => k;

    const baseRow: DailyAttendanceReport = {
        date: BASE_DATE,
        employee_id: "EMP001",
        employment_id: "EMP001",
        employee_name: "Test",
        scheduled_start: "07:30:00",
        scheduled_end: "16:30:00",
        break_start: "11:30:00",
        break_end: "12:30:00",
        break_duration_minutes: 60,
        is_break_auto_deduct: true,
        actual_punch_in: iso(BASE_DATE, "07:19"),
        actual_punch_out: iso(BASE_DATE, "17:42"),
        computed_std_hours: 8.0,
        computed_ot_hours: 0.5,
        std_work_units: 480,
        ot_work_units: 30,
        unit_type: "HOUR",
        unit_coefficient_hours: 1.0,
        status: "present",
        ot_claim_status: "none",
        generation_metadata: null,
        workspace_id: "WS1",
        employee_identifier: "E01",
        is_holiday: false,
    } as DailyAttendanceReport;

    it("axis shows only work entry boundaries, not auto-computed break times", () => {
        const timeEntries: TimeEntry[] = [
            {
                id: "te-1",
                entry_type: "work",
                punch_in_time: iso(BASE_DATE, "07:19"),
                punch_out_time: iso(BASE_DATE, "11:30"),
                duration_minutes: 251,
                note: null,
            },
            {
                id: "te-2",
                entry_type: "break",
                punch_in_time: iso(BASE_DATE, "11:30"),
                punch_out_time: iso(BASE_DATE, "12:30"),
                duration_minutes: 60,
                note: null,
                is_break_overtime: false,
            },
            {
                id: "te-3",
                entry_type: "work",
                punch_in_time: iso(BASE_DATE, "12:30"),
                punch_out_time: iso(BASE_DATE, "17:42"),
                duration_minutes: 312,
                note: null,
            },
        ];

        const row = {
            ...baseRow,
            generation_metadata: { time_entries: timeEntries },
        } as DailyAttendanceReport;

        render(<DailyReportTimeline formatTime={formatTime} row={row} t={t} />);

        // Work entry boundaries should be in the DOM
        expect(screen.getByText("07:19")).toBeInTheDocument();
        expect(screen.getByText("11:30")).toBeInTheDocument();
        expect(screen.getByText("12:30")).toBeInTheDocument();
        expect(screen.getByText("17:42")).toBeInTheDocument();

        // The axis labels for work entries use emerald color
        const workInLabels = screen.getAllByText("07:19");
        // One is the axis tick, should exist
        expect(workInLabels.length).toBeGreaterThanOrEqual(1);
    });

    it("does not render axis ticks for break-only entries", () => {
        // Only break entries — axis should be empty (workEntries empty)
        const breakOnlyEntries: TimeEntry[] = [
            {
                id: "te-break",
                entry_type: "break",
                punch_in_time: iso(BASE_DATE, "12:00"),
                punch_out_time: iso(BASE_DATE, "13:00"),
                duration_minutes: 60,
                note: null,
                is_break_overtime: false,
            },
        ];

        const row = {
            ...baseRow,
            scheduled_start: null,
            scheduled_end: null,
            break_start: null,
            break_end: null,
            generation_metadata: { time_entries: breakOnlyEntries },
        } as DailyAttendanceReport;

        render(<DailyReportTimeline formatTime={formatTime} row={row} t={t} />);

        // No work entries → axis should not show any times (fallback to empty)
        // The axis div exists but should have no time labels
        const axisContainer = document.querySelectorAll(".text-emerald-600");
        // No work entry labels should be present
        expect(axisContainer.length).toBe(0);
    });

    it("shows all work entry boundaries for genuine multi-session (non-auto-deduct)", () => {
        const multiSessionEntries: TimeEntry[] = [
            {
                id: "te-1",
                entry_type: "work",
                punch_in_time: iso(BASE_DATE, "07:00"),
                punch_out_time: iso(BASE_DATE, "09:00"),
                duration_minutes: 120,
                note: null,
            },
            {
                id: "te-2",
                entry_type: "work",
                punch_in_time: iso(BASE_DATE, "13:00"),
                punch_out_time: iso(BASE_DATE, "17:00"),
                duration_minutes: 240,
                note: null,
            },
        ];

        const row = {
            ...baseRow,
            is_break_auto_deduct: false,
            generation_metadata: { time_entries: multiSessionEntries },
        } as DailyAttendanceReport;

        render(<DailyReportTimeline formatTime={formatTime} row={row} t={t} />);

        // Both work sessions should have their in/out times in the axis
        expect(screen.getByText("07:00")).toBeInTheDocument();
        expect(screen.getByText("09:00")).toBeInTheDocument();
        expect(screen.getByText("13:00")).toBeInTheDocument();
        expect(screen.getByText("17:00")).toBeInTheDocument();
    });

    it("visual bar still renders break blocks even though axis skips them", () => {
        const timeEntries: TimeEntry[] = [
            {
                id: "te-work",
                entry_type: "work",
                punch_in_time: iso(BASE_DATE, "08:00"),
                punch_out_time: iso(BASE_DATE, "12:00"),
                duration_minutes: 240,
                note: null,
            },
            {
                id: "te-break",
                entry_type: "break",
                punch_in_time: iso(BASE_DATE, "12:00"),
                punch_out_time: iso(BASE_DATE, "13:00"),
                duration_minutes: 60,
                note: null,
            },
            {
                id: "te-work2",
                entry_type: "work",
                punch_in_time: iso(BASE_DATE, "13:00"),
                punch_out_time: iso(BASE_DATE, "17:00"),
                duration_minutes: 240,
                note: null,
            },
        ];

        const row = {
            ...baseRow,
            generation_metadata: { time_entries: timeEntries },
        } as DailyAttendanceReport;

        render(<DailyReportTimeline formatTime={formatTime} row={row} t={t} />);

        // Break label should appear in the visual bar (gray block)
        const breakLabels = screen.getAllByText("common.break");
        expect(breakLabels.length).toBeGreaterThanOrEqual(1);
    });

    it("renders segment label when segment data is present", () => {
        const segmentRow = {
            ...baseRow,
            segment_type: "work",
            segment_count: 2,
            sort_order: 0,
            interval_name: "Ca sáng",
            generation_metadata: { time_entries: [] as TimeEntry[] },
        } as DailyAttendanceReport;

        render(<DailyReportTimeline formatTime={formatTime} row={segmentRow} t={t} />);

        expect(screen.getByText("Ca sáng")).toBeInTheDocument();
        // Segment counter shows "1 / 2" in the badge
        expect(screen.getByText("1 / 2")).toBeInTheDocument();
    });

    it("handles empty time_entries gracefully", () => {
        const row = {
            ...baseRow,
            generation_metadata: { time_entries: [] as TimeEntry[] },
        } as DailyAttendanceReport;

        render(<DailyReportTimeline formatTime={formatTime} row={row} t={t} />);

        // Should render without crashing — scheduled axis should still show
        expect(screen.getByText("07:30")).toBeInTheDocument();
        expect(screen.getByText("16:30")).toBeInTheDocument();
    });
});
