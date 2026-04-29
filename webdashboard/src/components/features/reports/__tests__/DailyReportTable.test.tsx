import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import { DailyAttendanceReport } from "@/lib/api";

import DailyReportTable from "../DailyReportTable";

describe("DailyReportTable", () => {
    const mockT = (key: string) => key;
    const mockFormatDate = (date: string) => date;
    const mockFormatTime = (time: string | null) => time || "---";

    const mockData: DailyAttendanceReport[] = [
        {
            date: "2024-03-20",
            employee_id: "EMP001",
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
        workUnitMinutes: 60,
        isDebugMode: false,
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
        expect(screen.getByText("common.date")).toBeInTheDocument();
        expect(screen.getByText("common.employee")).toBeInTheDocument();
        expect(screen.getByText("common.shift")).toBeInTheDocument();
        expect(screen.getByText("reports.table.std_hours")).toBeInTheDocument();
    });

    it("renders employee data rows with high-density spacing classes", () => {
        render(<DailyReportTable {...defaultProps} />);

        // Check if the data is rendered
        expect(screen.getByText("John Doe")).toBeInTheDocument();
        expect(screen.getByText("Jane Smith")).toBeInTheDocument();

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

        // The Confirm button should be present as a button with a title attribute for tooltip
        // We check if it has the title "common.attendance_confirm" (since we mocked t)
        const confirmButtons = screen.getAllByTitle("common.attendance_confirm");
        expect(confirmButtons.length).toBeGreaterThan(0);

        // The button should have padding classes p-1.5 for the compacted design
        expect(confirmButtons[0].className).toContain("p-1.5");
    });

    it("triggers callbacks when action buttons are clicked", () => {
        render(<DailyReportTable {...defaultProps} />);

        // Find the Confirm button for John Doe
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

        // Employee header should be gone
        expect(screen.queryByText("common.employee")).not.toBeInTheDocument();

        // Employee names should not be rendered in cells
        expect(screen.queryByText("John Doe")).not.toBeInTheDocument();
        expect(screen.queryByText("Jane Smith")).not.toBeInTheDocument();

        // But dates and other columns still work
        expect(screen.getByText("common.date")).toBeInTheDocument();
        expect(screen.getByText("common.shift")).toBeInTheDocument();
    });

    it("shows employee name column by default (hideEmployeeName=false)", () => {
        render(<DailyReportTable {...defaultProps} hideEmployeeName={false} />);

        expect(screen.getByText("common.employee")).toBeInTheDocument();
        expect(screen.getByText("John Doe")).toBeInTheDocument();
        expect(screen.getByText("Jane Smith")).toBeInTheDocument();
    });
    it("uses flex-1 min-h-0 for dynamic height instead of hardcoded max-h", () => {
        render(<DailyReportTable {...defaultProps} />);

        // The outer scrollable container should use flex-1 and min-h-0
        const table = screen.getByRole("table");
        const outerDiv = table.parentElement;
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
            } as DailyAttendanceReport,
        ];
        render(<DailyReportTable {...defaultProps} data={dataWithUnscheduled} />);

        const badge = screen.getByTitle("reports.status.unscheduled_tooltip");
        expect(badge).toBeInTheDocument();
        expect(badge.textContent).toContain("reports.status.unscheduled");
    });

    it("calls onResolveUnscheduled when unscheduled badge is clicked", () => {
        const onResolve = vi.fn();
        const dataWithUnscheduled: DailyAttendanceReport[] = [
            {
                ...mockData[0],
                status: "unscheduled",
            } as DailyAttendanceReport,
        ];
        render(
            <DailyReportTable
                {...defaultProps}
                data={dataWithUnscheduled}
                onResolveUnscheduled={onResolve}
            />
        );

        const badge = screen.getByTitle("reports.status.unscheduled_tooltip");
        fireEvent.click(badge);
        expect(onResolve).toHaveBeenCalledWith(dataWithUnscheduled[0]);
    });

    it("does not show unscheduled badge when status is not unscheduled", () => {
        render(<DailyReportTable {...defaultProps} />);

        // Default mockData has status "present" → should not render unscheduled badge
        const badge = screen.queryByTitle("reports.status.unscheduled_tooltip");
        expect(badge).not.toBeInTheDocument();
    });
});
