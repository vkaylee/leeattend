import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi } from "vitest";

import { DailyAttendanceReport } from "@/lib/api";

import { DailyReportTableRow, DailyReportTableRowProps } from "../daily-report-table-row";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

// Mock the ScheduleTracePopover since it's an internal dependency
vi.mock("../../../shifts/schedule-trace-popover", () => ({
    ScheduleTracePopover: () => <span data-testid="mock-trace-popover">TRACE</span>,
}));

// Mock lucide-react icons
vi.mock("lucide-react", () => ({
    RefreshCw: ({ className }: { className?: string }) => <span className={className}>⟳</span>,
    Bug: () => <span>🐛</span>,
    CheckCircle: () => <span>✓</span>,
    ThumbsUp: () => <span>👍</span>,
    ThumbsDown: () => <span>👎</span>,
    ChevronRight: ({ className }: { className?: string }) => <span className={className}>▶</span>,
}));

describe("DailyReportTableRow — Segment Cells", () => {
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

    const defaultProps: DailyReportTableRowProps = {
        row: baseSegmentRow,
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

    // ── SegmentTypeCell Tests ──

    it("renders segment type badge when segment_type is present", () => {
        render(
            <table><tbody><DailyReportTableRow {...defaultProps} /></tbody></table>
        );
        expect(screen.getByText("work_segments.segment_type.work")).toBeInTheDocument();
    });

    it("renders dash when segment_type is null", () => {
        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={baseNonSegmentRow} />
            </tbody></table>
        );
        // Should show "—" for type and counter cells (at least 2 dashes)
        const dashes = screen.getAllByText("—");
        expect(dashes.length).toBeGreaterThanOrEqual(2);
    });

    it("renders correct segment type colors for different types", () => {
        const standbyRow = {
            ...baseSegmentRow,
            segment_type: "standby",
            segment_id: "seg-standby",
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={standbyRow} />
            </tbody></table>
        );
        expect(screen.getByText("work_segments.segment_type.standby")).toBeInTheDocument();

        const badge = screen.getByText("work_segments.segment_type.standby");
        // Should have the standby color classes
        expect(badge.className).toContain("text-orange-700");
    });

    it("renders all known segment types with correct i18n keys", () => {
        const types = ["work", "standby", "on_call", "training", "travel"];
        types.forEach((type) => {
            const typeRow = {
                ...baseSegmentRow,
                segment_type: type,
                segment_id: `seg-${type}`,
                sort_order: 0,
                segment_count: 1,
            } as DailyAttendanceReport;

            const { unmount } = render(
                <table><tbody>
                    <DailyReportTableRow {...defaultProps} row={typeRow} />
                </tbody></table>
            );

            expect(screen.getByText(`work_segments.segment_type.${type}`)).toBeInTheDocument();
            unmount();
        });
    });

    // ── SegmentCounterCell Tests ──

    it("renders segment counter when multi-segment and segment_type is present", () => {
        render(
            <table><tbody><DailyReportTableRow {...defaultProps} /></tbody></table>
        );
        // Counter shows "1/2" (sort_order 0 + 1 = 1, segment_count = 2)
        expect(screen.getByText("1")).toBeInTheDocument();
        expect(screen.getByText("/2")).toBeInTheDocument();
    });

    it("renders dash for single segment rows", () => {
        const singleSegmentRow = {
            ...baseSegmentRow,
            segment_count: 1,
            segment_id: "seg-single",
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={singleSegmentRow} />
            </tbody></table>
        );
        // Should show dash for counter since count=1
        const dashes = screen.getAllByText("—");
        expect(dashes.length).toBeGreaterThanOrEqual(1);
    });

    it("renders correct counter for different sort positions", () => {
        const secondSegmentRow = {
            ...baseSegmentRow,
            sort_order: 1,
            segment_count: 3,
            segment_id: "seg-2",
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={secondSegmentRow} />
            </tbody></table>
        );
        // Counter shows "2/3"
        expect(screen.getByText("2")).toBeInTheDocument();
        expect(screen.getByText("/3")).toBeInTheDocument();
    });

    // ── ScheduledCell Tests ──

    it("renders scheduled start and end times", () => {
        render(
            <table><tbody><DailyReportTableRow {...defaultProps} /></tbody></table>
        );
        expect(screen.getByText(/08:00 — 12:00/)).toBeInTheDocument();
    });

    it("renders no-shift message when scheduled_start is null", () => {
        const noShiftRow = {
            ...baseSegmentRow,
            scheduled_start: null,
            scheduled_end: null,
            segment_id: "seg-noshift",
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={noShiftRow} />
            </tbody></table>
        );
        expect(screen.getByText("reports.no_shift")).toBeInTheDocument();
    });

    it("shows interval_name above scheduled time in segment mode", () => {
        render(
            <table><tbody><DailyReportTableRow {...defaultProps} /></tbody></table>
        );
        const intervalLabel = screen.getByText("Ca sáng");
        expect(intervalLabel).toBeInTheDocument();
    });

    it("shows break info for non-segment rows", () => {
        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={baseNonSegmentRow} />
            </tbody></table>
        );
        expect(screen.getByText(/common.break: 60m/)).toBeInTheDocument();
    });

    it("does not show break info for segment rows", () => {
        render(
            <table><tbody><DailyReportTableRow {...defaultProps} /></tbody></table>
        );
        expect(screen.queryByText(/common.break/)).not.toBeInTheDocument();
    });

    it("shows break info with start-end times when available", () => {
        const breakTimedRow = {
            ...baseNonSegmentRow,
            break_start: "12:00:00",
            break_end: "13:00:00",
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={breakTimedRow} />
            </tbody></table>
        );
        expect(screen.getByText(/common.break: 12:00-13:00/)).toBeInTheDocument();
    });

    // ── Visual Grouping Tests ──

    it("applies first-in-group border accent", () => {
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    isFirstInGroup
                    isMultiSegmentGroup
                />
            </tbody></table>
        );
        const row = screen.getByRole("row");
        expect(row.className).toContain("border-t-2");
        expect(row.className).toContain("border-l-2");
    });

    it("does not apply first-in-group accent when not first", () => {
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    isFirstInGroup={false}
                    isMultiSegmentGroup
                />
            </tbody></table>
        );
        const row = screen.getByRole("row");
        expect(row.className).not.toContain("border-t-2");
    });

    it("applies muted date color for non-first segments", () => {
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    isFirstInGroup={false}
                    isMultiSegmentGroup
                />
            </tbody></table>
        );
        // The date cell should have muted text class
        const dateText = screen.getByText("2024-03-20");
        expect(dateText.className).toContain("text-slate-300");
    });

    it("renders default date color for first segment in group", () => {
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    isFirstInGroup
                    isMultiSegmentGroup
                />
            </tbody></table>
        );
        const dateText = screen.getByText("2024-03-20");
        expect(dateText.className).not.toContain("text-slate-300");
    });

    it("does not apply multi-segment styling when not multi-segment", () => {
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    isFirstInGroup
                    isMultiSegmentGroup={false}
                    row={baseNonSegmentRow}
                />
            </tbody></table>
        );
        const row = screen.getByRole("row");
        expect(row.className).not.toContain("border-l-2");
        expect(row.className).not.toContain("border-t-2");
    });

    // ── Key Generation & Expansion Tests ──

    it("generates unique segmentKey with segment_id", () => {
        const toggleSpy = vi.fn();
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    onToggleExpandRow={toggleSpy}
                />
            </tbody></table>
        );

        // First button in the row is the expand/collapse chevron
        const buttons = screen.getAllByRole("button");
        fireEvent.click(buttons[0]);
        // Should be called with segmentKey containing seg-1
        expect(toggleSpy).toHaveBeenCalledWith(
            expect.stringContaining("seg-1")
        );
    });

    it("uses sort_order as fallback when segment_id is null", () => {
        const toggleSpy = vi.fn();
        const noSegIdRow = {
            ...baseSegmentRow,
            segment_id: null,
            sort_order: 2,
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    onToggleExpandRow={toggleSpy}
                    row={noSegIdRow}
                />
            </tbody></table>
        );

        const buttons = screen.getAllByRole("button");
        fireEvent.click(buttons[0]);
        // Key should contain sort_order as fallback
        expect(toggleSpy).toHaveBeenCalledWith(
            expect.stringContaining("-2")
        );
    });

    it("uses '0' as ultimate fallback in segmentKey", () => {
        const toggleSpy = vi.fn();
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    onToggleExpandRow={toggleSpy}
                    row={baseNonSegmentRow}
                />
            </tbody></table>
        );

        const buttons = screen.getAllByRole("button");
        fireEvent.click(buttons[0]);
        // Key should use '0' as fallback for non-segment rows
        expect(toggleSpy).toHaveBeenCalledWith(
            expect.stringContaining("-0")
        );
    });

    it("looks up adjustment by employment_id-based dayKey", () => {
        const adjustmentMap = {
            "EMP002-2024-03-21": {
                id: "adj-1",
                employment_id: "EMP002",
                requested_date: "2024-03-21",
            },
        } as unknown as DailyReportTableRowProps["adjustmentMap"];

        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    adjustmentMap={adjustmentMap}
                    row={baseNonSegmentRow}
                />
            </tbody></table>
        );
        // The confirm button should have the "pending" styling (orange) since an adjustment exists
        const confirmButton = screen.getByTitle("reports.confirm.pending_badge");
        expect(confirmButton.className).toContain("bg-orange-50");
    });

    it("compares reprocessing/confirming by employee_id-based dayKey", () => {
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    confirmingRow="EMP002-2024-03-21"
                    reprocessingRow="EMP002-2024-03-21"
                    row={baseNonSegmentRow}
                />
            </tbody></table>
        );
        // Both buttons should show pending/loading state
        const reprocessButton = screen.getByTitle("reports.reprocess.tooltip");
        expect(reprocessButton).toBeInTheDocument();
    });

    // ── Expanded Row Tests ──

    it("shows expanded timeline row when segmentKey is in expandedDebugRows", () => {
        const expandedSet = new Set<string>(["EMP001-2024-03-20-seg-1"]);
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    expandedDebugRows={expandedSet}
                />
            </tbody></table>
        );
        // Timeline title should be visible
        expect(screen.getByText("reports.timeline.title")).toBeInTheDocument();
    });

    it("does not show expanded row when segmentKey not in expandedDebugRows", () => {
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    expandedDebugRows={new Set<string>()}
                />
            </tbody></table>
        );
        expect(screen.queryByText("reports.timeline.title")).not.toBeInTheDocument();
    });

    it("expanded row uses correct colSpan for isDebugMode=true", () => {
        const expandedSet = new Set<string>(["EMP001-2024-03-20-seg-1"]);
        render(
            <table><tbody>
                <DailyReportTableRow
                    {...defaultProps}
                    expandedDebugRows={expandedSet}
                    isDebugMode
                />
            </tbody></table>
        );
        // The expanded td should span 12 columns (hideEmployeeName implicitly false → +1 debug column = 12)
        const expandedCell = screen.getByText("reports.timeline.title").closest("td");
        expect(expandedCell).toBeInTheDocument();
        expect(expandedCell?.getAttribute("colSpan")).toBe("12");
    });

    // ── Split Badge Tests ──

    it("shows Split badge when time_entries has multiple entries", () => {
        const multiSessionRow = {
            ...baseSegmentRow,
            generation_metadata: {
                time_entries: [
                    {
                        id: "te-1",
                        punch_in_time: "2024-03-20T07:55:00",
                        punch_out_time: "2024-03-20T10:00:00",
                        entry_type: "work",
                        duration_minutes: 125,
                        note: null,
                    },
                    {
                        id: "te-2",
                        punch_in_time: "2024-03-20T10:15:00",
                        punch_out_time: "2024-03-20T12:00:00",
                        entry_type: "work",
                        duration_minutes: 105,
                        note: null,
                    },
                ],
            },
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={multiSessionRow} />
            </tbody></table>
        );
        // Split badge should appear
        const splitBadges = screen.getAllByText("Split");
        expect(splitBadges.length).toBeGreaterThanOrEqual(1);
    });

    it("hides Split badge for auto-deduct break with 2 work blocks", () => {
        // Single interval with auto-deduct break: work → break → work
        // The employee only punched at shift start/end, break was auto-computed
        const autoDeductRow = {
            ...baseSegmentRow,
            is_break_auto_deduct: true,
            generation_metadata: {
                time_entries: [
                    {
                        id: "te-work1",
                        punch_in_time: "2024-03-20T07:19:00",
                        punch_out_time: "2024-03-20T11:30:00",
                        entry_type: "work",
                        duration_minutes: 251,
                        note: null,
                    },
                    {
                        id: "te-break",
                        punch_in_time: "2024-03-20T11:30:00",
                        punch_out_time: "2024-03-20T12:30:00",
                        entry_type: "break",
                        duration_minutes: 60,
                        note: null,
                        is_break_overtime: false,
                    },
                    {
                        id: "te-work2",
                        punch_in_time: "2024-03-20T12:30:00",
                        punch_out_time: "2024-03-20T17:42:00",
                        entry_type: "work",
                        duration_minutes: 312,
                        note: null,
                    },
                ],
            },
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={autoDeductRow} />
            </tbody></table>
        );
        // With is_break_auto_deduct=true and exactly 2 work entries,
        // the Split badge should NOT appear (single session with auto-deduct gap)
        expect(screen.queryByText("Split")).not.toBeInTheDocument();
    });

    it("shows Split badge for auto-deduct break with >2 work blocks (genuine multi-session)", () => {
        const multiSessionAutoDeductRow = {
            ...baseSegmentRow,
            is_break_auto_deduct: true,
            generation_metadata: {
                time_entries: [
                    {
                        id: "te-work1",
                        punch_in_time: "2024-03-20T07:00:00",
                        punch_out_time: "2024-03-20T09:00:00",
                        entry_type: "work",
                        duration_minutes: 120,
                        note: null,
                    },
                    {
                        id: "te-break",
                        punch_in_time: "2024-03-20T09:00:00",
                        punch_out_time: "2024-03-20T10:00:00",
                        entry_type: "break",
                        duration_minutes: 60,
                        note: null,
                        is_break_overtime: false,
                    },
                    {
                        id: "te-work2",
                        punch_in_time: "2024-03-20T10:00:00",
                        punch_out_time: "2024-03-20T12:00:00",
                        entry_type: "work",
                        duration_minutes: 120,
                        note: null,
                    },
                    {
                        id: "te-work3",
                        punch_in_time: "2024-03-20T13:00:00",
                        punch_out_time: "2024-03-20T17:00:00",
                        entry_type: "work",
                        duration_minutes: 240,
                        note: null,
                    },
                ],
            },
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={multiSessionAutoDeductRow} />
            </tbody></table>
        );
        // With 3 work entries (>2), Split badge SHOULD appear
        const badges = screen.getAllByText("Split");
        expect(badges.length).toBeGreaterThanOrEqual(1);
    });

    it("does not show Split badge for single time entries", () => {
        const singleSessionRow = {
            ...baseSegmentRow,
            generation_metadata: {
                time_entries: [
                    {
                        id: "te-1",
                        punch_in_time: "2024-03-20T07:55:00",
                        punch_out_time: "2024-03-20T12:00:00",
                        entry_type: "work",
                        duration_minutes: 245,
                        note: null,
                    },
                ],
            },
        } as DailyAttendanceReport;

        render(
            <table><tbody>
                <DailyReportTableRow {...defaultProps} row={singleSessionRow} />
            </tbody></table>
        );
        expect(screen.queryByText("Split")).not.toBeInTheDocument();
    });
});
