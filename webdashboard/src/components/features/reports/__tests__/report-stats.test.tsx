import { render, screen } from "@testing-library/react";

import { describe, it, expect } from "vitest";

import type { AttendanceSummary, DailyAttendanceReport } from "@/lib/api";

import ReportStats from "../../reports/report-stats";

const mockT = (key: string) => key;

const mockSummaryData = [
    {
        employee_id: "emp-001",
        employee_name: "Alice",
        computed_std_hours: 40,
        computed_ot_hours: 8,
    },
    {
        employee_id: "emp-002",
        employee_name: "Bob",
        computed_std_hours: 35,
        computed_ot_hours: 12,
    },
] as unknown as AttendanceSummary[];

const mockDailyData = [
    {
        employee_id: "emp-001",
        employee_name: "Alice",
        date: "2026-06-01",
        computed_std_hours: 8,
        computed_ot_hours: 2,
        status: "present",
    },
    {
        employee_id: "emp-002",
        employee_name: "Bob",
        date: "2026-06-01",
        computed_std_hours: 7,
        computed_ot_hours: 3,
        status: "present",
    },
    {
        employee_id: "emp-001",
        employee_name: "Alice",
        date: "2026-06-02",
        computed_std_hours: 8,
        computed_ot_hours: 0,
        status: "present",
    },
] as unknown as DailyAttendanceReport[];

describe("ReportStats", () => {
    describe("summary report type", () => {
        it("renders standard hours card with computed total", () => {
            render(
                <ReportStats filteredData={mockSummaryData} reportType="summary" t={mockT} />
            );

            // 40 + 35 = 75 std hours
            expect(screen.getByText("75")).toBeInTheDocument();
            expect(screen.getByText("reports.quick_stats.std_hours")).toBeInTheDocument();
        });

        it("renders overtime hours card with computed total", () => {
            render(
                <ReportStats filteredData={mockSummaryData} reportType="summary" t={mockT} />
            );

            // 8 + 12 = 20 OT hours
            expect(screen.getByText("20")).toBeInTheDocument();
            expect(screen.getByText("reports.quick_stats.ot_hours")).toBeInTheDocument();
        });

        it("renders employee count card with unique employee count", () => {
            render(
                <ReportStats filteredData={mockSummaryData} reportType="summary" t={mockT} />
            );

            // 2 unique employees
            expect(screen.getByText("2")).toBeInTheDocument();
            expect(screen.getByText("common.employees")).toBeInTheDocument();
        });
    });

    describe("daily report type", () => {
        it("renders standard hours computed from daily records", () => {
            render(
                <ReportStats filteredData={mockDailyData} reportType="daily" t={mockT} />
            );

            // 8 + 7 + 8 = 23 std hours
            expect(screen.getByText("23")).toBeInTheDocument();
        });

        it("renders overtime hours computed from daily records", () => {
            render(
                <ReportStats filteredData={mockDailyData} reportType="daily" t={mockT} />
            );

            // 2 + 3 + 0 = 5 OT hours
            expect(screen.getByText("5")).toBeInTheDocument();
        });

        it("renders unique employee count from daily records", () => {
            render(
                <ReportStats filteredData={mockDailyData} reportType="daily" t={mockT} />
            );

            // 2 unique employees (emp-001 appears twice)
            expect(screen.getByText("2")).toBeInTheDocument();
        });
    });

    describe("edge cases", () => {
        it("handles empty data gracefully", () => {
            render(
                <ReportStats filteredData={[]} reportType="summary" t={mockT} />
            );

            // All three cards show 0 (std_hours, ot_hours, employees)
            const zeroElements = screen.getAllByText("0");
            expect(zeroElements.length).toBe(3);
        });

        it("handles null computed_hours gracefully (treats as 0)", () => {
            const dataWithNulls = [
                {
                    employee_id: "emp-001",
                    employee_name: "Alice",
                    computed_std_hours: undefined as unknown as number,
                    computed_ot_hours: undefined as unknown as number,
                },
            ] as unknown as AttendanceSummary[];

            render(
                <ReportStats filteredData={dataWithNulls} reportType="summary" t={mockT} />
            );

            // std_hours and ot_hours both resolve to 0, employee count = 1
            const zeroElements = screen.getAllByText("0");
            expect(zeroElements.length).toBe(2); // std_hours: 0, ot_hours: 0
            // Employee count is 1
            expect(screen.getByText("1")).toBeInTheDocument();
        });

        it("handles single employee data", () => {
            const singleEmployee = [
                {
                    employee_id: "emp-001",
                    employee_name: "Alice",
                    computed_std_hours: 42,
                    computed_ot_hours: 5,
                },
            ] as unknown as AttendanceSummary[];

            render(
                <ReportStats filteredData={singleEmployee} reportType="summary" t={mockT} />
            );

            expect(screen.getByText("42")).toBeInTheDocument();
            expect(screen.getByText("5")).toBeInTheDocument();
            expect(screen.getByText("1")).toBeInTheDocument(); // 1 unique employee
        });

        it("handles large datasets efficiently", () => {
            const largeData = Array.from({ length: 100 }, (_, i) => ({
                employee_id: `emp-${String(i).padStart(3, "0")}`,
                employee_name: `Employee ${i}`,
                computed_std_hours: 40,
                computed_ot_hours: 5,
            })) as unknown as AttendanceSummary[];

            render(
                <ReportStats filteredData={largeData} reportType="summary" t={mockT} />
            );

            // formatHours uses Intl.NumberFormat which may add locale-specific separators
            // 100 * 40 = 4000 std hours, 100 * 5 = 500 OT hours
            const formattedStd = new Intl.NumberFormat(undefined, {
                minimumFractionDigits: 0,
                maximumFractionDigits: 2,
            }).format(4000);
            const formattedOt = new Intl.NumberFormat(undefined, {
                minimumFractionDigits: 0,
                maximumFractionDigits: 2,
            }).format(500);

            expect(screen.getByText(formattedStd)).toBeInTheDocument();
            expect(screen.getByText(formattedOt)).toBeInTheDocument();
            // 100 unique employees (not formatted since it's Set.size, not a formatted number)
            // Actually totalEmployees is a number, not formatted via formatHours
            // It's rendered directly as {totalEmployees}, so no formatting
            const employeeCountStr = largeData.length.toString(); // "100"
            expect(screen.getByText(employeeCountStr)).toBeInTheDocument();
        });
    });
});
