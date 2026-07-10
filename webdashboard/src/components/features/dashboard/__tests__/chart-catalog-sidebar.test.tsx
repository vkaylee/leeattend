import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import type { WidgetId } from "@/hooks/use-dashboard-layout";

import ChartCatalogSidebar from "../chart-catalog-sidebar";

// Vietnamese translations resolved by the global i18n mock (src/test/setup.tsx)
const LABELS = {
    title: "Biểu đồ",
    searchPlaceholder: "Tìm kiếm biểu đồ...",
    noResults: "Không tìm thấy biểu đồ",
    collapse: "Thu gọn bảng",
    expand: "Mở rộng bảng",
    attendanceCategory: "Chấm công",
    workforceCategory: "Nhân sự",
    performanceCategory: "Hiệu suất",
    summaryCategory: "Tổng quan",
    attendanceTrend: "Xu hướng chấm công 30 ngày",
    departmentChart: "Tỷ lệ chấm công theo phòng ban",
    peakTimes: "Phân bố giờ cao điểm check-in",
    overtimeTrend: "Xu hướng giờ tăng ca",
    latenessByDay: "Tần suất đi muộn theo ngày",
    leaveTypes: "Phân bố nghỉ phép theo loại",
    recruitmentSources: "Phân tích nguồn tuyển dụng",
    turnoverAnalytics: "Phân tích nghỉ việc",
    unexcusedAbsences: "Nghỉ không phép theo thời gian",
};

const allWidgets: { id: WidgetId; labelKey: string }[] = [
    { id: "attendance_trend", labelKey: "dashboard.meta.attendance_trend" },
    { id: "department_chart", labelKey: "dashboard.meta.department_chart" },
    { id: "peak_times", labelKey: "dashboard.meta.peak_times" },
    { id: "overtime_trend", labelKey: "dashboard.meta.overtime_trend" },
    { id: "lateness_by_day", labelKey: "dashboard.meta.lateness_by_day" },
    { id: "leave_types", labelKey: "dashboard.meta.leave_types" },
    { id: "recruitment_sources", labelKey: "dashboard.meta.recruitment_sources" },
    { id: "turnover_analytics", labelKey: "dashboard.meta.turnover_analytics" },
    { id: "unexcused_absences", labelKey: "dashboard.meta.unexcused_absences" },
];

const defaultVisibleWidgets: Record<WidgetId, boolean> = {
    kpi_cards: true,
    donut_chart: true,
    quick_stats: true,
    promo_cards: false,
    attendance_trend: true,
    department_chart: true,
    peak_times: true,
    overtime_trend: false,
    lateness_by_day: true,
    leave_types: false,
    recruitment_sources: true,
    turnover_analytics: true,
    unexcused_absences: false,
};

function setup(overrides?: {
    visibleWidgets?: Record<WidgetId, boolean>;
}) {
    const onToggleWidget = vi.fn();
    const visibleWidgets = overrides?.visibleWidgets ?? defaultVisibleWidgets;
    const utils = render(
        <ChartCatalogSidebar
            allWidgets={allWidgets}
            onToggleWidget={onToggleWidget}
            visibleWidgets={visibleWidgets}
        />
    );
    return { ...utils, onToggleWidget };
}

describe("ChartCatalogSidebar", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        localStorage.clear();
    });

    it("renders category headers in order", () => {
        setup();

        expect(screen.getByText(LABELS.attendanceCategory)).toBeInTheDocument();
        expect(screen.getByText(LABELS.workforceCategory)).toBeInTheDocument();
        expect(screen.getByText(LABELS.performanceCategory)).toBeInTheDocument();
    });

    it("renders widget toggle buttons with role=switch", () => {
        setup();

        const switches = screen.getAllByRole("switch");
        expect(switches.length).toBe(allWidgets.length);
    });

    it("marks visible widgets as checked (aria-checked=true)", () => {
        setup();

        const attendanceSwitch = screen.getByRole("switch", { name: LABELS.attendanceTrend });
        expect(attendanceSwitch).toHaveAttribute("aria-checked", "true");
    });

    it("marks hidden widgets as unchecked (aria-checked=false)", () => {
        setup();

        const otSwitch = screen.getByRole("switch", { name: LABELS.overtimeTrend });
        expect(otSwitch).toHaveAttribute("aria-checked", "false");
    });

    it("calls onToggleWidget when a widget switch is clicked", () => {
        const { onToggleWidget } = setup();

        fireEvent.click(screen.getByRole("switch", { name: LABELS.overtimeTrend }));
        expect(onToggleWidget).toHaveBeenCalledWith("overtime_trend");
    });

    it("renders collapse button", () => {
        setup();

        expect(screen.getByText(LABELS.collapse)).toBeInTheDocument();
    });

    it("renders search input", () => {
        setup();

        const searchInput = screen.getByPlaceholderText(LABELS.searchPlaceholder);
        expect(searchInput).toBeInTheDocument();
    });

    it("renders the catalog title", () => {
        setup();

        expect(screen.getByText(LABELS.title)).toBeInTheDocument();
    });

    it("filters widgets by search query matching Vietnamese label", () => {
        setup();

        const searchInput = screen.getByPlaceholderText(LABELS.searchPlaceholder);

        // Search for "chấm công" (attendance) should match attendance_trend and department_chart
        fireEvent.change(searchInput, { target: { value: "chấm công" } });

        expect(screen.getByRole("switch", { name: LABELS.attendanceTrend })).toBeInTheDocument();
        expect(screen.getByRole("switch", { name: LABELS.departmentChart })).toBeInTheDocument();
    });

    it("hides non-matching widgets when searching", () => {
        setup();

        const searchInput = screen.getByPlaceholderText(LABELS.searchPlaceholder);

        // Search for "tăng ca" should only match overtime_trend
        fireEvent.change(searchInput, { target: { value: "tăng ca" } });

        expect(screen.getByRole("switch", { name: LABELS.overtimeTrend })).toBeInTheDocument();
        // Other widgets should be hidden
        expect(screen.queryByRole("switch", { name: LABELS.attendanceTrend })).not.toBeInTheDocument();
    });

    it("shows no results message when search matches nothing", () => {
        setup();

        const searchInput = screen.getByPlaceholderText(LABELS.searchPlaceholder);

        fireEvent.change(searchInput, { target: { value: "xyznonexistent123" } });

        expect(screen.getByText(LABELS.noResults)).toBeInTheDocument();
    });

    it("clears search results when query is cleared", () => {
        setup();

        const searchInput = screen.getByPlaceholderText(LABELS.searchPlaceholder);

        // First filter to nothing
        fireEvent.change(searchInput, { target: { value: "xyznonexistent123" } });
        expect(screen.getByText(LABELS.noResults)).toBeInTheDocument();

        // Then clear
        fireEvent.change(searchInput, { target: { value: "" } });
        expect(screen.queryByText(LABELS.noResults)).not.toBeInTheDocument();
        // All widgets should be back
        const switches = screen.getAllByRole("switch");
        expect(switches.length).toBe(allWidgets.length);
    });

    it("shows checkmark SVG inside visible widget toggle buttons", () => {
        setup();

        // The checkmark is inside the circle span, not the top-level button
        const attendanceSwitch = screen.getByRole("switch", { name: LABELS.attendanceTrend });
        const circleSpan = attendanceSwitch.querySelector(".w-4.h-4.rounded-full");
        expect(circleSpan).not.toBeNull();
        const checkmark = circleSpan!.querySelector("svg");
        expect(checkmark).not.toBeNull();
    });

    it("does not show checkmark SVG inside hidden widget toggle buttons", () => {
        setup();

        // Hidden button still shows the icon SVG (lucide-timer) but NOT the checkmark
        const otSwitch = screen.getByRole("switch", { name: LABELS.overtimeTrend });
        const circleSpan = otSwitch.querySelector(".w-4.h-4.rounded-full");
        expect(circleSpan).not.toBeNull();
        const checkmark = circleSpan!.querySelector("svg");
        expect(checkmark).toBeNull();
    });

    it("handles empty widgets list gracefully", () => {
        render(
            <ChartCatalogSidebar
                allWidgets={[]}
                onToggleWidget={vi.fn()}
                visibleWidgets={{} as Record<WidgetId, boolean>}
            />
        );

        // Title should still render
        expect(screen.getByText(LABELS.title)).toBeInTheDocument();
        // No switch roles
        expect(screen.queryByRole("switch")).not.toBeInTheDocument();
        // No results message shown
        expect(screen.getByText(LABELS.noResults)).toBeInTheDocument();
    });

    it("calls onToggleWidget with correct widget ID for each widget", () => {
        const { onToggleWidget } = setup();

        fireEvent.click(screen.getByRole("switch", { name: LABELS.leaveTypes }));
        expect(onToggleWidget).toHaveBeenCalledWith("leave_types");
    });
});
