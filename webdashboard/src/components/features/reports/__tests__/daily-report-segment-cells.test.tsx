import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi } from "vitest";

import { DailyAttendanceReport } from "@/lib/api";

import DailyReportTable, { type DailyReportTableProps } from "../daily-report-table";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => fallback ?? key,
    }),
}));

vi.mock("../../../shifts/schedule-trace-popover", () => ({
    ScheduleTracePopover: () => <span data-testid="mock-trace-popover">TRACE</span>,
}));

vi.mock("@/hooks/use-workspace", () => ({
    useWorkspace: () => ({
        workspace: { timezone: "UTC" },
    }),
}));

/**
 * Segment-cell behaviour previously lived in daily-report-table-row.test.tsx against
 * the hand-rolled <tr> component. That component was replaced by SmartTable columns
 * (see daily-report-columns.tsx), so these cases now drive the whole table and assert
 * on the rendered desktop grid instead.
 */
describe("DailyReportTable — segment cells and grouping", () => {
    const mockFormatDate = (date: string) => date;
    const mockFormatTime = (time: string | null) => time ?? "---";

    const baseSegmentRow: DailyAttendanceReport = {
        date: "2024-03-20",
        employee_id: "EMP001",
        employment_id: "EMP001",
        employee_name: "John Doe",
        segment_id: "seg-1",
        segment_type: "work",
        sort_order: 0,
        segment_count: 2,
        interval_name: "Ca sáng",
        scheduled_start: "08:00:00",
        scheduled_end: "12:00:00",
        break_duration_minutes: null,
        break_start: null,
        break_end: null,
        actual_punch_in: "07:55:00",
        actual_punch_out: "12:00:00",
        std_work_units: 240,
        ot_work_units: 0,
        computed_std_hours: 4.0,
        computed_ot_hours: 0,
        unit_type: "HOUR",
        unit_coefficient_hours: 1.0,
        status: "present",
        ot_claim_status: "none",
        generation_metadata: null,
        workspace_id: "WS1",
        employee_identifier: "E01",
        is_break_auto_deduct: false,
        is_holiday: false,
    } as DailyAttendanceReport;

    const baseNonSegmentRow: DailyAttendanceReport = {
        date: "2024-03-21",
        employee_id: "EMP002",
        employment_id: "EMP002",
        employee_name: "Jane Smith",
        segment_id: null,
        segment_type: null,
        sort_order: null,
        segment_count: null,
        interval_name: null,
        scheduled_start: "08:00:00",
        scheduled_end: "17:00:00",
        break_duration_minutes: 60,
        break_start: null,
        break_end: null,
        actual_punch_in: "08:00:00",
        actual_punch_out: "17:00:00",
        std_work_units: 480,
        ot_work_units: 0,
        computed_std_hours: 8.0,
        computed_ot_hours: 0,
        unit_type: "DAY",
        unit_coefficient_hours: 8.0,
        status: "present",
        ot_claim_status: "none",
        generation_metadata: null,
        workspace_id: "WS1",
        employee_identifier: "E02",
        is_break_auto_deduct: false,
        is_holiday: false,
    } as DailyAttendanceReport;

    const defaultProps: DailyReportTableProps = {
        data: [baseSegmentRow],
        isLoading: false,
        t: (key: string) => key,
        formatDate: mockFormatDate,
        formatTime: mockFormatTime,
        hideEmployeeName: false,
        isDebugMode: false,
        expandedDebugRows: new Set<string>(),
        reprocessingRow: null,
        confirmingRow: null,
        adjustmentMap: {},
        onToggleExpandRow: vi.fn(),
        onReprocessClick: vi.fn(),
        onConfirmClick: vi.fn(),
    };

    /** Desktop grid rows only — the mobile card list renders the same data. */
    function desktopRows(container: HTMLElement): HTMLElement[] {
        return Array.from(container.querySelectorAll(".hidden.lg\\:block tbody tr"));
    }

    // ── SegmentTypeCell ──

    it("renders segment type badge when segment_type is present", () => {
        render(<DailyReportTable {...defaultProps} />);
        expect(screen.getAllByText("work_segments.segment_type.work").length).toBeGreaterThan(0);
    });

    it("renders dash when segment_type is null", () => {
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[baseNonSegmentRow]} />,
        );
        const desktop = container.querySelector(".hidden.lg\\:block");
        expect(desktop?.textContent).toContain("—");
    });

    it("renders all known segment types with correct i18n keys", () => {
        const types = ["work", "break", "standby", "overtime"];
        types.forEach((segType) => {
            const typeRow = { ...baseSegmentRow, segment_type: segType } as DailyAttendanceReport;
            const { unmount } = render(<DailyReportTable {...defaultProps} data={[typeRow]} />);
            expect(
                screen.getAllByText(`work_segments.segment_type.${segType}`).length,
            ).toBeGreaterThan(0);
            unmount();
        });
    });

    // ── SegmentCounterCell ──

    it("renders segment counter when multi-segment and segment_type is present", () => {
        const { container } = render(<DailyReportTable {...defaultProps} />);
        const desktop = container.querySelector(".hidden.lg\\:block");
        // sort_order 0 of segment_count 2 → "1" over "/2"
        expect(desktop?.textContent).toContain("/2");
    });

    it("renders dash for single segment rows", () => {
        const singleSegmentRow = {
            ...baseSegmentRow,
            segment_count: 1,
        } as DailyAttendanceReport;
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[singleSegmentRow]} />,
        );
        const desktop = container.querySelector(".hidden.lg\\:block");
        expect(desktop?.textContent).not.toContain("/1");
    });

    it("renders correct counter for different sort positions", () => {
        const secondSegmentRow = {
            ...baseSegmentRow,
            segment_id: "seg-2",
            sort_order: 1,
        } as DailyAttendanceReport;
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[secondSegmentRow]} />,
        );
        const desktop = container.querySelector(".hidden.lg\\:block");
        expect(desktop?.textContent).toContain("2");
        expect(desktop?.textContent).toContain("/2");
    });

    // ── ScheduledCell ──

    it("renders scheduled start and end times", () => {
        render(<DailyReportTable {...defaultProps} />);
        expect(screen.getAllByText(/08:00 — 12:00/).length).toBeGreaterThan(0);
    });

    it("renders no-shift message when scheduled_start is null", () => {
        const noShiftRow = { ...baseSegmentRow, scheduled_start: null } as DailyAttendanceReport;
        render(<DailyReportTable {...defaultProps} data={[noShiftRow]} />);
        expect(screen.getAllByText("reports.no_shift").length).toBeGreaterThan(0);
    });

    it("shows interval_name above scheduled time in segment mode", () => {
        render(<DailyReportTable {...defaultProps} />);
        expect(screen.getAllByText("Ca sáng").length).toBeGreaterThan(0);
    });

    it("shows break info for non-segment rows", () => {
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[baseNonSegmentRow]} />,
        );
        const desktop = container.querySelector(".hidden.lg\\:block");
        expect(desktop?.textContent).toContain("common.break: 60m");
    });

    it("does not show break info for segment rows", () => {
        const { container } = render(<DailyReportTable {...defaultProps} />);
        const desktop = container.querySelector(".hidden.lg\\:block");
        expect(desktop?.textContent).not.toContain("common.break");
    });

    it("shows break info with start-end times when available", () => {
        const breakTimedRow = {
            ...baseNonSegmentRow,
            break_start: "12:00:00",
            break_end: "13:00:00",
        } as DailyAttendanceReport;
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[breakTimedRow]} />,
        );
        const desktop = container.querySelector(".hidden.lg\\:block");
        expect(desktop?.textContent).toContain("common.break: 12:00-13:00");
    });

    // ── Visual grouping ──

    it("applies multi-segment group border accents to the first row of a group", () => {
        const second = {
            ...baseSegmentRow,
            segment_id: "seg-2",
            sort_order: 1,
        } as DailyAttendanceReport;
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[baseSegmentRow, second]} />,
        );
        const rows = desktopRows(container);
        expect(rows[0].className).toContain("border-t-2");
        expect(rows[0].className).toContain("border-l-2");
        // Second row shares the employee+date group, so it is not "first"
        expect(rows[1].className).not.toContain("border-t-2");
    });

    it("mutes date text for non-first segments in a group", () => {
        const second = {
            ...baseSegmentRow,
            segment_id: "seg-2",
            sort_order: 1,
        } as DailyAttendanceReport;
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[baseSegmentRow, second]} />,
        );
        const dates = Array.from(
            container.querySelectorAll(".hidden.lg\\:block tbody tr td span.font-mono"),
        ).filter((el) => el.textContent === "2024-03-20");
        expect(dates[0].className).not.toContain("text-slate-300");
        expect(dates[1].className).toContain("text-slate-300");
    });

    it("does not apply multi-segment styling when not multi-segment", () => {
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[baseNonSegmentRow]} />,
        );
        const rows = desktopRows(container);
        expect(rows[0].className).not.toContain("border-l-2");
        expect(rows[0].className).not.toContain("border-t-2");
    });

    // ── Key generation & expansion ──

    it("generates unique segmentKey with segment_id", () => {
        const toggleSpy = vi.fn();
        const { container } = render(
            <DailyReportTable {...defaultProps} onToggleExpandRow={toggleSpy} />,
        );
        const desktop = container.querySelector(".hidden.lg\\:block");
        const buttons = desktop?.querySelectorAll("button") ?? [];
        fireEvent.click(buttons[0]);
        expect(toggleSpy).toHaveBeenCalledWith(expect.stringContaining("seg-1"));
    });

    it("uses sort_order as fallback when segment_id is null", () => {
        const toggleSpy = vi.fn();
        const noSegIdRow = {
            ...baseSegmentRow,
            segment_id: null,
            sort_order: 2,
        } as DailyAttendanceReport;
        const { container } = render(
            <DailyReportTable {...defaultProps} data={[noSegIdRow]} onToggleExpandRow={toggleSpy} />,
        );
        const desktop = container.querySelector(".hidden.lg\\:block");
        const buttons = desktop?.querySelectorAll("button") ?? [];
        fireEvent.click(buttons[0]);
        expect(toggleSpy).toHaveBeenCalledWith(expect.stringContaining("-2"));
    });

    it("uses '0' as ultimate fallback in segmentKey", () => {
        const toggleSpy = vi.fn();
        const { container } = render(
            <DailyReportTable
                {...defaultProps}
                data={[baseNonSegmentRow]}
                onToggleExpandRow={toggleSpy}
            />,
        );
        const desktop = container.querySelector(".hidden.lg\\:block");
        const buttons = desktop?.querySelectorAll("button") ?? [];
        fireEvent.click(buttons[0]);
        expect(toggleSpy).toHaveBeenCalledWith(expect.stringContaining("-0"));
    });

    it("looks up adjustment by employment_id-based dayKey", () => {
        const adjustmentMap = {
            "EMP002-2024-03-21": {
                id: "adj-1",
                employment_id: "EMP002",
                requested_date: "2024-03-21",
            },
        } as unknown as DailyReportTableProps["adjustmentMap"];

        render(
            <DailyReportTable
                {...defaultProps}
                adjustmentMap={adjustmentMap}
                data={[baseNonSegmentRow]}
            />,
        );
        const confirmButtons = screen.getAllByTitle("reports.confirm.pending_badge");
        expect(confirmButtons.some((b) => b.className.includes("bg-orange-50"))).toBe(true);
    });

    it("compares reprocessing/confirming by employee_id-based dayKey", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                confirmingRow="EMP002-2024-03-21"
                data={[baseNonSegmentRow]}
                reprocessingRow="EMP002-2024-03-21"
            />,
        );
        expect(screen.getAllByTitle("reports.reprocess.tooltip").length).toBeGreaterThan(0);
    });

    // ── Expanded row ──

    it("shows expanded timeline row when segmentKey is in expandedDebugRows", () => {
        const expandedSet = new Set<string>(["EMP001-2024-03-20-seg-1"]);
        render(<DailyReportTable {...defaultProps} expandedDebugRows={expandedSet} />);
        expect(screen.getByText("reports.timeline.title")).toBeInTheDocument();
    });

    it("does not show expanded row when segmentKey not in expandedDebugRows", () => {
        render(<DailyReportTable {...defaultProps} expandedDebugRows={new Set<string>()} />);
        expect(screen.queryByText("reports.timeline.title")).not.toBeInTheDocument();
    });

    /**
     * Replaces the old hardcoded-colSpan assertion. SmartTable owns the detail row
     * wrapper, so colSpan is derived from the real visible-column count rather than
     * a hand-maintained constant that drifted when columns were added.
     */
    it("spans the expanded detail row across every visible column", () => {
        const expandedSet = new Set<string>(["EMP001-2024-03-20-seg-1"]);
        const { container } = render(
            <DailyReportTable {...defaultProps} expandedDebugRows={expandedSet} isDebugMode />,
        );
        const expandedCell = screen.getByText("reports.timeline.title").closest("td");
        const headerCells = container.querySelectorAll(".hidden.lg\\:block thead th");
        expect(expandedCell?.getAttribute("colSpan")).toBe(String(headerCells.length));
    });

    it("keeps the detail-row colSpan correct when the employee column is hidden", () => {
        const expandedSet = new Set<string>(["EMP001-2024-03-20-seg-1"]);
        const { container } = render(
            <DailyReportTable {...defaultProps} expandedDebugRows={expandedSet} hideEmployeeName />,
        );
        const expandedCell = screen.getByText("reports.timeline.title").closest("td");
        const headerCells = container.querySelectorAll(".hidden.lg\\:block thead th");
        expect(expandedCell?.getAttribute("colSpan")).toBe(String(headerCells.length));
    });

    // ── Split badge ──

    it("shows Split badge when time_entries has multiple work entries", () => {
        const multiSessionRow = {
            ...baseSegmentRow,
            generation_metadata: {
                time_entries: [
                    { id: "te-1", punch_in_time: "2024-03-20T07:55:00", punch_out_time: "2024-03-20T10:00:00", entry_type: "work", duration_minutes: 125, note: null },
                    { id: "te-2", punch_in_time: "2024-03-20T10:15:00", punch_out_time: "2024-03-20T12:00:00", entry_type: "work", duration_minutes: 105, note: null },
                ],
            },
        } as unknown as DailyAttendanceReport;
        render(<DailyReportTable {...defaultProps} data={[multiSessionRow]} />);
        expect(screen.getAllByText("Split").length).toBeGreaterThan(0);
    });

    it("hides Split badge for auto-deduct break with exactly 2 work blocks", () => {
        const autoDeductRow = {
            ...baseSegmentRow,
            is_break_auto_deduct: true,
            generation_metadata: {
                time_entries: [
                    { id: "te-1", punch_in_time: "2024-03-20T08:00:00", punch_out_time: "2024-03-20T12:00:00", entry_type: "work", duration_minutes: 240, note: null },
                    { id: "te-2", punch_in_time: "2024-03-20T13:00:00", punch_out_time: "2024-03-20T17:00:00", entry_type: "work", duration_minutes: 240, note: null },
                ],
            },
        } as unknown as DailyAttendanceReport;
        render(<DailyReportTable {...defaultProps} data={[autoDeductRow]} />);
        expect(screen.queryByText("Split")).not.toBeInTheDocument();
    });

    it("shows Split badge for auto-deduct break with >2 work blocks", () => {
        const multiSessionAutoDeductRow = {
            ...baseSegmentRow,
            is_break_auto_deduct: true,
            generation_metadata: {
                time_entries: [
                    { id: "te-1", punch_in_time: "2024-03-20T08:00:00", punch_out_time: "2024-03-20T10:00:00", entry_type: "work", duration_minutes: 120, note: null },
                    { id: "te-2", punch_in_time: "2024-03-20T10:30:00", punch_out_time: "2024-03-20T12:00:00", entry_type: "work", duration_minutes: 90, note: null },
                    { id: "te-3", punch_in_time: "2024-03-20T13:00:00", punch_out_time: "2024-03-20T17:00:00", entry_type: "work", duration_minutes: 240, note: null },
                ],
            },
        } as unknown as DailyAttendanceReport;
        render(<DailyReportTable {...defaultProps} data={[multiSessionAutoDeductRow]} />);
        expect(screen.getAllByText("Split").length).toBeGreaterThan(0);
    });

    it("does not show Split badge for a single time entry", () => {
        const singleSessionRow = {
            ...baseSegmentRow,
            generation_metadata: {
                time_entries: [
                    { id: "te-1", punch_in_time: "2024-03-20T08:00:00", punch_out_time: "2024-03-20T17:00:00", entry_type: "work", duration_minutes: 540, note: null },
                ],
            },
        } as unknown as DailyAttendanceReport;
        render(<DailyReportTable {...defaultProps} data={[singleSessionRow]} />);
        expect(screen.queryByText("Split")).not.toBeInTheDocument();
    });
});
