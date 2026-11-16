import { render, screen, fireEvent } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";

import { DailyAttendanceReport } from "@/lib/api";

import DailyReportTable from "../daily-report-table";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

// Mock useWorkspace — DailyReportTimeline (rendered inside the table row)
// now reads workspace.timezone for correct punch positioning.
vi.mock("@/hooks/use-workspace", () => ({
    useWorkspace: () => ({
        organization: { timezone: "UTC" },
    }),
}));

describe("DailyReportTable", () => {
    const mockT = (key: string) => key;
    const mockFormatDate = (date: string) => date;
    const mockFormatTime = (time: string | null) => time ?? "---";

    const mockData: DailyAttendanceReport[] = [
        {
            date: "2024-03-20",
            employee_id: "EMP001",
            employment_id: "EMP001",
            employee_name: "John Doe",
            scheduled_start: "08:00:00",
            scheduled_end: "17:00:00",
            break_duration_minutes: 60,
            break_start: null,
            break_end: null,
            actual_punch_in: "07:55:00",
            actual_punch_out: "17:05:00",
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
            employee_identifier: "E01",
            is_break_auto_deduct: false,
            is_holiday: false
        } as DailyAttendanceReport,
        {
            date: "2024-03-21",
            employee_id: "EMP002",
            employment_id: "EMP002",
            employee_name: "Jane Smith",
            scheduled_start: "08:00:00",
            scheduled_end: "17:00:00",
            break_duration_minutes: 60,
            break_start: null,
            break_end: null,
            actual_punch_in: "08:15:00",
            actual_punch_out: "18:00:00",
            std_work_units: 465,
            ot_work_units: 60,
            computed_std_hours: 7.75,
            computed_ot_hours: 1.0,
            unit_type: "DAY",
            unit_coefficient_hours: 8.0,
            status: "late",
            ot_claim_status: "pending_manager",
            generation_metadata: null,
            workspace_id: "WS1",
            employee_identifier: "E02",
            is_break_auto_deduct: false,
            is_holiday: false
        } as DailyAttendanceReport
    ];

    const defaultProps = {
        data: mockData,
        isLoading: false,
        expandedDebugRows: new Set<string>(),
        reprocessingRow: null,
        confirmingRow: null,
        adjustmentMap: {},
        t: mockT,
        formatDate: mockFormatDate,
        formatTime: mockFormatTime,
        onToggleExpandRow: vi.fn(),
        onReprocessClick: vi.fn(),
        onConfirmClick: vi.fn(),
        onApproveOt: vi.fn(),
    };

    it("renders the table headers correctly", () => {
        render(<DailyReportTable {...defaultProps} />);
        // Desktop + mobile both render → use getAllByText for data in both views
        expect(screen.getAllByText("common.date")[0]).toBeInTheDocument();
        expect(screen.getAllByText("common.employee")[0]).toBeInTheDocument();
        expect(screen.getAllByText("reports.table.scheduled")[0]).toBeInTheDocument();
        expect(screen.getAllByText("reports.table.std_hours")[0]).toBeInTheDocument();
    });

    it("renders employee data rows with high-density spacing classes", () => {
        render(<DailyReportTable {...defaultProps} />);

        // Check if the data is rendered (desktop + mobile both render these)
        expect(screen.getAllByText("John Doe")[0]).toBeInTheDocument();
        expect(screen.getAllByText("Jane Smith")[0]).toBeInTheDocument();

        // The high-density UI classes applied in Phase 1 should group rows tightly
        const cells = screen.getAllByRole("cell");

        // Simply asserting that some cells contain the new spacing classes (px-4 py-2.5)
        // Testing library gets the raw element so we can check className
        const hasCompactPadding = cells.some(cell =>
            cell.className.includes("px-4") && cell.className.includes("py-2.5")
        );
        expect(hasCompactPadding).toBe(true);
    });

    it("applies font-mono to numeric time and hour representations", () => {
        render(<DailyReportTable {...defaultProps} />);

        // Verify that the punch in time is wrapped in a container that has font-mono
        const timeCells = screen.getAllByText("07:55:00");
        const containerCell = timeCells[0].closest("td");
        expect(containerCell?.className).toContain("font-mono");
    });

    it("renders action buttons in icon-only compact layout", () => {
        render(<DailyReportTable {...defaultProps} />);

        // Desktop confirm buttons with title
        const confirmButtons = screen.getAllByTitle("common.attendance_confirm");
        expect(confirmButtons.length).toBeGreaterThan(0);

        // The button should have padding classes p-1.5 for the compacted design
        expect(confirmButtons[0].className).toContain("p-1.5");
    });

    it("triggers callbacks when action buttons are clicked", () => {
        render(<DailyReportTable {...defaultProps} />);

        // Find the Confirm button for John Doe (desktop view)
        const confirmButtons = screen.getAllByTitle("common.attendance_confirm");
        fireEvent.click(confirmButtons[0]);
        expect(defaultProps.onConfirmClick).toHaveBeenCalledWith(mockData[0], undefined);

        // Find the Reprocess button for John Doe
        const reprocessButtons = screen.getAllByTitle("reports.reprocess.tooltip");
        fireEvent.click(reprocessButtons[0]);
        expect(defaultProps.onReprocessClick).toHaveBeenCalledWith(mockData[0]);
    });

    it("hides employee name column when hideEmployeeName is true", () => {
        render(<DailyReportTable {...defaultProps} hideEmployeeName />);

        // Employee header should be gone from both views
        expect(screen.queryByText("common.employee")).not.toBeInTheDocument();

        // Employee names should not be rendered in any cell
        expect(screen.queryByText("John Doe")).not.toBeInTheDocument();
        expect(screen.queryByText("Jane Smith")).not.toBeInTheDocument();

        // But dates and other columns still work
        expect(screen.getAllByText("common.date")[0]).toBeInTheDocument();
        expect(screen.getAllByText("reports.table.scheduled")[0]).toBeInTheDocument();
    });

    it("shows employee name column by default (hideEmployeeName=false)", () => {
        render(<DailyReportTable {...defaultProps} hideEmployeeName={false} />);

        expect(screen.getAllByText("common.employee")[0]).toBeInTheDocument();
        expect(screen.getAllByText("John Doe")[0]).toBeInTheDocument();
        expect(screen.getAllByText("Jane Smith")[0]).toBeInTheDocument();
    });
    it("uses flex-1 min-h-0 for dynamic height instead of hardcoded max-h", () => {
        render(<DailyReportTable {...defaultProps} />);

        // The outer scrollable container should use flex-1 and min-h-0 on the outer div
        const tables = screen.getAllByRole("table");
        // The desktop table is inside ResponsiveTable wrapper → its grandparent is the outer scroll container
        const outerDiv = tables[0].closest(".overflow-auto.flex-1.min-h-0");
        expect(outerDiv).not.toBeNull();
        expect(outerDiv!.className).toContain("flex-1");
        expect(outerDiv!.className).toContain("min-h-0");
        expect(outerDiv!.className).toContain("overflow-auto");
        // Should NOT have hardcoded max-h
        expect(outerDiv!.className).not.toContain("max-h-[calc");
    });

    it("shows unscheduled badge when status is unscheduled", () => {
        const dataWithUnscheduled: DailyAttendanceReport[] = [
            {
                ...mockData[0],
                status: "unscheduled",
                scheduled_start: null,
                scheduled_end: null,
            },
        ];
        render(<DailyReportTable {...defaultProps} data={dataWithUnscheduled} />);

        const badges = screen.getAllByTitle("reports.status.unscheduled_tooltip");
        expect(badges[0]).toBeInTheDocument();
        expect(badges[0].textContent).toContain("reports.status.unscheduled");
    });

    it("calls onResolveUnscheduled when unscheduled badge is clicked", () => {
        const onResolve = vi.fn();
        const dataWithUnscheduled: DailyAttendanceReport[] = [
            {
                ...mockData[0],
                status: "unscheduled",
            },
        ];
        render(
            <DailyReportTable
                {...defaultProps}
                data={dataWithUnscheduled}
                onResolveUnscheduled={onResolve}
            />
        );

        const badges = screen.getAllByTitle("reports.status.unscheduled_tooltip");
        fireEvent.click(badges[0]);
        expect(onResolve).toHaveBeenCalledWith(dataWithUnscheduled[0]);
    });

    it("does not show unscheduled badge when status is not unscheduled", () => {
        render(<DailyReportTable {...defaultProps} />);

        // Default mockData has status "present" → should not render unscheduled badge
        const badge = screen.queryByTitle("reports.status.unscheduled_tooltip");
        expect(badge).not.toBeInTheDocument();
    });

    it("displays correct shift name in trace popover when shift_name is provided", async () => {
        const user = userEvent.setup();
        const dataWithTraces: DailyAttendanceReport[] = [
            {
                ...mockData[0],
                shift_name: "Mocked Special Shift",
                generation_metadata: {
                    schedule_traces: [
                        {
                            rule_name: "Override Rule",
                            priority: 100,
                            status: "applied",
                            reason: "Manual assignment"
                        }
                    ]
                }
            }
        ];
        render(<DailyReportTable {...defaultProps} data={dataWithTraces} />);

        const trigger = screen.getByTestId("schedule-trace-trigger");
        expect(trigger).toBeInTheDocument();
        await user.click(trigger);

        expect(screen.getByText("shifts.traceability.applied_shift")).toBeInTheDocument();
        expect(screen.getByText("Mocked Special Shift")).toBeInTheDocument();
        expect(screen.getByText("Override Rule")).toBeInTheDocument();
    });

    // ── Per-Segment (Option A) Tests ──

    const multiSegmentData: DailyAttendanceReport[] = [
        {
            date: "2024-03-20",
            employee_id: "EMP001",
            employment_id: "EMP001",
            employee_name: "John Doe",
            segment_id: "seg-morning",
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
            is_holiday: false
        } as DailyAttendanceReport,
        {
            date: "2024-03-20",
            employee_id: "EMP001",
            employment_id: "EMP001",
            employee_name: "John Doe",
            segment_id: "seg-afternoon",
            segment_type: "standby",
            sort_order: 1,
            segment_count: 2,
            interval_name: "Ca chiều",
            scheduled_start: "13:00:00",
            scheduled_end: "17:00:00",
            break_duration_minutes: null,
            break_start: null,
            break_end: null,
            actual_punch_in: "13:05:00",
            actual_punch_out: "17:02:00",
            std_work_units: 237,
            ot_work_units: 2,
            computed_std_hours: 3.95,
            computed_ot_hours: 0.5,
            unit_type: "HOUR",
            unit_coefficient_hours: 1.0,
            status: "late",
            ot_claim_status: "none",
            generation_metadata: null,
            workspace_id: "WS1",
            employee_identifier: "E01",
            is_break_auto_deduct: false,
            is_holiday: false
        } as DailyAttendanceReport,
    ];

    const mixedSegmentAndRegularData: DailyAttendanceReport[] = [
        // Multi-segment employee
        ...multiSegmentData,
        // Regular (non-segment) employee
        {
            date: "2024-03-21",
            employee_id: "EMP002",
            employment_id: "EMP002",
            employee_name: "Jane Smith",
            segment_type: null,
            segment_count: null,
            sort_order: null,
            segment_id: null,
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
            is_holiday: false
        } as DailyAttendanceReport,
    ];

    it("renders segment type and segment number columns in header", () => {
        render(<DailyReportTable {...defaultProps} />);
        expect(screen.getAllByText("reports.table.segment_type")[0]).toBeInTheDocument();
        expect(screen.getAllByText("reports.table.segment_number")[0]).toBeInTheDocument();
    });

    it("renders Scheduled header instead of Shift", () => {
        render(<DailyReportTable {...defaultProps} />);
        expect(screen.getAllByText("reports.table.scheduled")[0]).toBeInTheDocument();
        // "Shift" header should no longer exist
        expect(screen.queryByText("common.shift")).not.toBeInTheDocument();
    });

    it("renders multi-segment data as separate rows without React key warnings", () => {
        const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);
        render(
            <DailyReportTable
                {...defaultProps}
                data={multiSegmentData}
                hideEmployeeName
            />
        );
        // Both segment rows should be rendered (desktop + mobile both render)
        expect(screen.getAllByText(/08:00 — 12:00/)[0]).toBeInTheDocument();
        expect(screen.getAllByText(/13:00 — 17:00/)[0]).toBeInTheDocument();
        // No React duplicate key warnings
        const keyWarnings = consoleSpy.mock.calls.filter(
            call => typeof call[0] === "string" && call[0].includes("same key")
        );
        expect(keyWarnings).toHaveLength(0);
        consoleSpy.mockRestore();
    });

    it("renders segment type badges for multi-segment data", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                data={multiSegmentData}
                hideEmployeeName
            />
        );
        // work_segments.segment_type.work and work_segments.segment_type.standby
        expect(screen.getAllByText("work_segments.segment_type.work")[0]).toBeInTheDocument();
        expect(screen.getAllByText("work_segments.segment_type.standby")[0]).toBeInTheDocument();
    });

    it("renders segment counter for multi-segment rows", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                data={multiSegmentData}
                hideEmployeeName
            />
        );
        // Both "1" and "2" appear in desktop table counter and may appear in mobile too
        expect(screen.getAllByText("1")[0]).toBeInTheDocument();
        expect(screen.getAllByText("2")[0]).toBeInTheDocument();
        // Denominator should appear in desktop rows
        const denominators = screen.getAllByText(/\/2/);
        expect(denominators.length).toBeGreaterThanOrEqual(2);
    });

    it("shows muted dash for segment type and counter on non-segment data", () => {
        const nonSegmentData: DailyAttendanceReport[] = [
            {
                date: "2024-03-20",
                employee_id: "EMP001",
                employment_id: "EMP001",
                employee_name: "John Doe",
                segment_type: null,
                segment_count: null,
                sort_order: null,
                scheduled_start: "08:00:00",
                scheduled_end: "17:00:00",
                actual_punch_in: "08:00:00",
                actual_punch_out: "17:00:00",
                computed_std_hours: 8.0,
                computed_ot_hours: 0,
                std_work_units: 480,
                ot_work_units: 0,
                break_duration_minutes: 60,
                break_start: null,
                break_end: null,
                unit_type: "DAY",
                unit_coefficient_hours: 8.0,
                status: "present",
                ot_claim_status: "none",
                generation_metadata: null,
                workspace_id: "WS1",
                employee_identifier: "E01",
                is_break_auto_deduct: false,
                is_holiday: false,
            } as DailyAttendanceReport,
        ];
        render(
            <DailyReportTable
                {...defaultProps}
                data={nonSegmentData}
                hideEmployeeName
            />
        );
        // Dash indicators appear for segment type + counter in desktop table
        const dashes = screen.getAllByText("—");
        expect(dashes.length).toBeGreaterThanOrEqual(2);
    });

    it("shows break info for non-segment rows", () => {
        const nonSegmentData: DailyAttendanceReport[] = [
            {
                date: "2024-03-20",
                employee_id: "EMP001",
                employment_id: "EMP001",
                employee_name: "John Doe",
                segment_type: null,
                segment_count: null,
                sort_order: null,
                scheduled_start: "08:00:00",
                scheduled_end: "17:00:00",
                break_duration_minutes: 60,
                break_start: null,
                break_end: null,
                actual_punch_in: "08:00:00",
                actual_punch_out: "17:00:00",
                computed_std_hours: 8.0,
                computed_ot_hours: 0,
                std_work_units: 480,
                ot_work_units: 0,
                unit_type: "DAY",
                unit_coefficient_hours: 8.0,
                status: "present",
                ot_claim_status: "none",
                generation_metadata: null,
                workspace_id: "WS1",
                employee_identifier: "E01",
                is_break_auto_deduct: false,
                is_holiday: false,
            } as DailyAttendanceReport,
        ];
        render(
            <DailyReportTable
                {...defaultProps}
                data={nonSegmentData}
                hideEmployeeName
            />
        );
        // Break info should appear: "common.break: 60m" on desktop table
        expect(screen.getByText(/common.break: 60m/)).toBeInTheDocument();
    });

    it("hides break info for segment rows", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                data={multiSegmentData}
                hideEmployeeName
            />
        );
        // Break info should NOT appear in segment mode
        expect(screen.queryByText(/common.break/)).not.toBeInTheDocument();
    });

    it("applies visual grouping borders for multi-segment rows", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                data={multiSegmentData}
                hideEmployeeName
            />
        );
        const rows = screen.getAllByRole("row");
        // Desktop data rows — SmartTable tags each with the `group/row` hover scope
        const dataRows = rows.filter(r => r.className.includes("group/row"));
        // First segment row should have top border accent class
        expect(dataRows[0].className).toContain("border-t-2");
        expect(dataRows[0].className).toContain("border-l-2");
        // Second segment row should NOT have top border accent (inner divider instead)
        expect(dataRows[1].className).not.toContain("border-t-2");
        expect(dataRows[1].className).toContain("border-l-2");
    });

    it("mutes date and employee text for non-first segments in a group", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                data={multiSegmentData}
                hideEmployeeName={false}
            />
        );
        // Both rows show employee_name "John Doe" (desktop + mobile)
        const nameCells = screen.getAllByText("John Doe");
        expect(nameCells.length).toBeGreaterThanOrEqual(2);
        // First occurrence in desktop should have the default bold color
        expect(nameCells[0].className).toContain("text-slate-700");
        // Second occurrence (non-first in group) in desktop should be muted
        expect(nameCells[1].className).toContain("text-slate-300");
    });

    it("handles mixed segment and non-segment data without errors", () => {
        const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);
        render(
            <DailyReportTable
                {...defaultProps}
                data={mixedSegmentAndRegularData}
            />
        );
        // Segment rows (desktop + mobile both render)
        expect(screen.getAllByText(/08:00 — 12:00/)[0]).toBeInTheDocument();
        expect(screen.getAllByText(/13:00 — 17:00/)[0]).toBeInTheDocument();
        // Non-segment row
        expect(screen.getAllByText(/08:00 — 17:00/)[0]).toBeInTheDocument();
        // Should have data rows (desktop only)
        const rows = screen.getAllByRole("row");
        expect(rows.length).toBeGreaterThanOrEqual(4);

        const keyWarnings = consoleSpy.mock.calls.filter(
            call => typeof call[0] === "string" && call[0].includes("same key")
        );
        expect(keyWarnings).toHaveLength(0);
        consoleSpy.mockRestore();
    });

    it("renders correct colSpan for skeleton loading with new columns", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                hideEmployeeName={false}
                isLoading
            />
        );
        // Skeleton rows in desktop table should have 13 <td> elements
        const rows = screen.getAllByRole("row");
        const tableHeaderRows = rows.filter(r => r.tagName === "TH" || r.closest("thead"));
        const dataRows = rows.filter(r => !tableHeaderRows.includes(r) && r.querySelector("td"));
        dataRows.slice(0, 5).forEach(row => {
            expect(row.children.length).toBe(12);
        });
    });

    it("renders correct colSpan for hideEmployeeName skeleton loading", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                hideEmployeeName
                isLoading
            />
        );
        // Skeleton rows with hideEmployeeName=true, isDebugMode=false → 11 columns
        const rows = screen.getAllByRole("row");
        const tableHeaderRows = rows.filter(r => r.tagName === "TH" || r.closest("thead"));
        const dataRows = rows.filter(r => !tableHeaderRows.includes(r) && r.querySelector("td"));
        dataRows.slice(0, 5).forEach(row => {
            expect(row.children.length).toBe(11);
        });
    });

    it("segment row expansion uses unique segmentKey per row", () => {
        const toggleSpy = vi.fn();
        render(
            <DailyReportTable
                {...defaultProps}
                data={multiSegmentData}
                hideEmployeeName
                onToggleExpandRow={toggleSpy}
            />
        );
        // Click first desktop expand button (buttons within table rows)
        const buttons = screen.getAllByRole("button");
        // Find a chevron button in the desktop view
        const chevronButtons = buttons.filter(b => b.querySelector(".lucide-chevron-right"));
        expect(chevronButtons.length).toBeGreaterThan(0);
        fireEvent.click(chevronButtons[0]);
        // Should be called with segmentKey containing segment_id
        expect(toggleSpy).toHaveBeenCalledWith(
            expect.stringContaining("seg-morning")
        );
    });

    it("uses interval_name as label in scheduled cell for segment rows", () => {
        render(
            <DailyReportTable
                {...defaultProps}
                data={multiSegmentData}
                hideEmployeeName
            />
        );
        expect(screen.getAllByText("Ca sáng")[0]).toBeInTheDocument();
        expect(screen.getAllByText("Ca chiều")[0]).toBeInTheDocument();
    });
});

