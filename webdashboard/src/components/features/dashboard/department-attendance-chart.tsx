import { Building2 } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
    Bar,
    BarChart,
    CartesianGrid,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";

import { ChartSummary } from "@/components/features/dashboard/chart-summary";
import { ChartTableTwin, type ChartTableColumn } from "@/components/features/dashboard/chart-table-twin";
import { DashboardChartError } from "@/components/features/dashboard/dashboard-chart-state";
import { DashboardDateRange } from "@/components/features/dashboard/dashboard-date-range";
import { useDashboardChartError } from "@/components/features/dashboard/use-dashboard-chart-error";
import { Typography } from "@/components/ui/typography";
import { useDepartmentAttendance, type DepartmentAttendancePoint } from "@/hooks/use-dashboard";
import { useOrganization } from "@/hooks/use-organization";

const TOP_N = 8;

function rateValue(rate: { value: string | null }): string {
    return rate.value == null ? "—" : `${rate.value}%`;
}

function defaultRange(): { startDate: string; endDate: string } {
    const now = new Date();
    const end = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    const s = new Date(now);
    s.setDate(s.getDate() - 29);
    const start = `${s.getFullYear()}-${String(s.getMonth() + 1).padStart(2, "0")}-${String(s.getDate()).padStart(2, "0")}`;
    return { startDate: start, endDate: end };
}

interface Row extends DepartmentAttendancePoint {
    attendance: number;
}

export default function DepartmentAttendanceChart() {
    const { t } = useTranslation();
    const { organization } = useOrganization();
    const [range, setRange] = useState(defaultRange);

    const timezone = organization?.timezone ?? undefined;
    const { data, error, isLoading } = useDepartmentAttendance({
        topN: TOP_N,
        rangeParams: { startDate: range.startDate, endDate: range.endDate, timezone },
    });
    useDashboardChartError(Boolean(error));

    // Backend already sorts descending by attendance rate; slice client-side for Top-N.
    const allRows: Row[] = useMemo(
        () =>
            (data ?? []).map((d) => ({
                ...d,
                attendance: d.attendance_rate.value == null ? 0 : Number(d.attendance_rate.value),
            })),
        [data],
    );

    const chartData = useMemo(
        () => allRows.slice(0, TOP_N).sort((a, b) => b.attendance - a.attendance),
        [allRows],
    );

    const hasData = allRows.length > 0 && allRows.some((d) => d.total_employees > 0);

    const columns: ChartTableColumn<Row>[] = [
        { header: t("dashboard.filters.department.label"), key: "department_name" },
        { header: t("dashboard.hr.attendance_rate"), key: "attendance_rate", numeric: true, render: (r) => rateValue(r.attendance_rate) },
        { header: t("dashboard.hr.late_rate"), key: "late_rate", numeric: true, render: (r) => rateValue(r.late_rate) },
        { header: t("dashboard.hr.absence_rate"), key: "absence_rate", numeric: true, render: (r) => rateValue(r.absence_rate) },
        { header: t("dashboard.hr.on_time"), key: "on_time", numeric: true },
        { header: t("dashboard.hr.late_today"), key: "late", numeric: true },
        { header: t("dashboard.hr.absent_today"), key: "absent", numeric: true },
        { header: t("dashboard.hr.on_leave_today"), key: "on_leave", numeric: true },
        { header: t("dashboard.hr.total_employees"), key: "total_employees", numeric: true },
    ];

    const summary = t("dashboard.hr.dept_attendance_summary");

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="flex items-center justify-between mb-8">
                <div className="flex items-center gap-4">
                    <div className="bg-blue-50 dark:bg-blue-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-blue-600 dark:text-blue-400 shadow-inner border border-blue-100 dark:border-blue-800">
                        <Building2 size={22} />
                    </div>
                    <div>
                        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h2">
                            {t("dashboard.hr.dept_attendance")}
                        </Typography>
                        <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
                            {t("dashboard.hr.dept_attendance_subtitle")}
                        </Typography>
                    </div>
                </div>
                <div className="flex min-w-0 max-w-full flex-wrap items-center justify-end gap-3">
                    <span className="text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">
                        {t("dashboard.hr.top_n", { count: TOP_N })}
                    </span>
                    <DashboardDateRange
                        endDate={range.endDate}
                        onChange={setRange}
                        startDate={range.startDate}
                        timezone={timezone}
                    />
                </div>
            </div>

            {isLoading ? (
                <div className="h-[320px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : error ? (
                <DashboardChartError icon={<Building2 size={48} />} />
            ) : !hasData ? (
                <div className="h-[320px] flex flex-col items-center justify-center">
                    <Building2 className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
                    <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
                        {t("common.no_data")}
                    </Typography>
                </div>
            ) : (
                <>
                    <ChartSummary text={summary} />
                    <div aria-label={summary} role="img">
                        <ResponsiveContainer height={320} width="100%">
                            <BarChart data={chartData} layout="vertical" margin={{ top: 5, right: 30, left: 10, bottom: 5 }}>
                                <CartesianGrid
                                    className="text-slate-100 dark:text-slate-800"
                                    horizontal={false}
                                    stroke="currentColor"
                                    strokeDasharray="3 3"
                                />
                                <XAxis
                                    axisLine={false}
                                    className="text-slate-400 dark:text-slate-500"
                                    tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                                    tickLine={false}
                                    type="number"
                                    unit="%"
                                />
                                <YAxis
                                    axisLine={false}
                                    className="text-slate-400 dark:text-slate-500"
                                    dataKey="department_name"
                                    tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                                    tickLine={false}
                                    type="category"
                                    width={120}
                                />
                                <Tooltip
                                    contentStyle={{
                                        borderRadius: "1rem",
                                        border: "none",
                                        boxShadow: "0 8px 30px rgba(0,0,0,.1)",
                                        fontSize: "11px",
                                        fontWeight: 600,
                                    }}
                                    formatter={(value: unknown) => [`${Number(value ?? 0)}%`, t("dashboard.hr.attendance_rate")]}
                                />
                                <Bar
                                    dataKey="attendance"
                                    fill="#10b981"
                                    name={t("dashboard.hr.attendance_rate")}
                                    radius={[0, 4, 4, 0]}
                                />
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                    <ChartTableTwin
                        caption={t("dashboard.hr.dept_attendance")}
                        columns={columns}
                        rows={allRows}
                        searchableKeys={["department_name"]}
                    />
                </>
            )}
        </div>
    );
}
