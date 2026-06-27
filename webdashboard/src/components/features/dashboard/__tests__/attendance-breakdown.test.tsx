import { render, screen } from "@testing-library/react";

import { describe, it, expect } from "vitest";

import { AttendanceBreakdown } from "../attendance-breakdown";

const mockDonutData = [
    { name: "Present", value: 42, fill: "#10b981" },
    { name: "Late", value: 5, fill: "#f59e0b" },
    { name: "Absent", value: 3, fill: "#ef4444" },
    { name: "On Leave", value: 2, fill: "#6366f1" },
];

const mockT = (key: string) => key;

describe("AttendanceBreakdown", () => {
    it("renders the donut chart when data is available", () => {
        render(
            <AttendanceBreakdown donutData={mockDonutData} hasDonutData={true} t={mockT} />
        );

        expect(screen.getByText("dashboard.hr.attendance_breakdown")).toBeInTheDocument();
        // Each legend item shows "name: value"
        expect(screen.getByText("Present: 42")).toBeInTheDocument();
        expect(screen.getByText("Late: 5")).toBeInTheDocument();
        expect(screen.getByText("Absent: 3")).toBeInTheDocument();
        expect(screen.getByText("On Leave: 2")).toBeInTheDocument();
    });

    it("renders empty state when hasDonutData is false", () => {
        render(
            <AttendanceBreakdown donutData={[]} hasDonutData={false} t={mockT} />
        );

        expect(screen.getByText("dashboard.hr.attendance_breakdown")).toBeInTheDocument();
        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders empty state when donutData is empty array with hasDonutData false", () => {
        render(
            <AttendanceBreakdown donutData={[]} hasDonutData={false} t={mockT} />
        );

        expect(screen.queryByText(/Present:/)).not.toBeInTheDocument();
        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("still renders chart when hasDonutData is true even with empty data", () => {
        // hasDonutData being true with empty data would be a caller bug,
        // but the component should still render the chart area rather than empty state
        render(
            <AttendanceBreakdown donutData={[]} hasDonutData={true} t={mockT} />
        );

        // Should NOT show "common.no_data" since hasDonutData=true
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
        // Title should still be there
        expect(screen.getByText("dashboard.hr.attendance_breakdown")).toBeInTheDocument();
    });

    it("renders donut chart with correct number of legend items", () => {
        const { container } = render(
            <AttendanceBreakdown donutData={mockDonutData} hasDonutData={true} t={mockT} />
        );

        // All 4 legend entries should be rendered
        const legendItems = container.querySelectorAll(".flex.items-center.gap-2");
        expect(legendItems.length).toBe(4);
    });

    it("renders legend with correct color indicators", () => {
        const { container } = render(
            <AttendanceBreakdown donutData={mockDonutData} hasDonutData={true} t={mockT} />
        );

        const colorDots = container.querySelectorAll(".w-3.h-3.rounded-full");
        expect(colorDots.length).toBe(4);

        // Verify colors match the data
        const expectedColors = ["#10b981", "#f59e0b", "#ef4444", "#6366f1"];
        colorDots.forEach((dot, i) => {
            expect((dot as HTMLElement).style.backgroundColor).toBe(expectedColors[i]);
        });
    });

    it("handles single data point correctly", () => {
        const singleData = [{ name: "Present", value: 50, fill: "#10b981" }];

        render(
            <AttendanceBreakdown donutData={singleData} hasDonutData={true} t={mockT} />
        );

        expect(screen.getByText("Present: 50")).toBeInTheDocument();
    });

    it("handles large values correctly", () => {
        const largeData = [
            { name: "Present", value: 999999, fill: "#10b981" },
        ];

        render(
            <AttendanceBreakdown donutData={largeData} hasDonutData={true} t={mockT} />
        );

        expect(screen.getByText("Present: 999999")).toBeInTheDocument();
    });
});
