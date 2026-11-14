import { TrendingUp } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
    CartesianGrid,
    Legend,
    Line,
    LineChart,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";

import { ChartSummary } from "@/components/features/dashboard/chart-summary";
import { ChartTableTwin, type ChartTableColumn } from "@/components/features/dashboard/chart-table-twin";
import { DashboardChartError } from "@/components/features/dashboard/dashboard-chart-state";
import { MetricSelector } from "@/components/features/dashboard/metric-selector";
import { Typography } from "@/components/ui/typography";
import { useAttendanceTrend, type AttendanceTrendPoint } from "@/hooks/use-dashboard";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

type MetricKey = "present" | "late" | "absent" | "on_leave";

const COLORS: Record<MetricKey, string> = {
    present: "#10b981",
    late: "#f59e0b",
    absent: "#ef4444",
    on_leave: "#6366f1",
};

interface Row extends AttendanceTrendPoint {
    label: string;
}

function rateValue(rate: { value: string | null }): string {
    return rate.value == null ? "—" : `${rate.value}%`;
}

export default function AttendanceTrendChart() {
    const { t } = useTranslation();
    const { formatDate } = useDateTimeFormatter();
    const { data: trend, error, isLoading } = useAttendanceTrend();
    const [metric, setMetric] = useState<MetricKey>("present");

    const chartData: Row[] = useMemo(
        () => (trend ?? []).map((point) => ({ ...point, label: formatDate(point.date) })),
        [trend, formatDate],
    );

    const hasData = chartData.length > 0 && chartData.some((d) => d.eligible > 0);

    const options: { label: string; value: MetricKey }[] = [
        { label: t("dashboard.hr.present_today"), value: "present" },
        { label: t("dashboard.hr.late_today"), value: "late" },
        { label: t("dashboard.hr.absent_today"), value: "absent" },
        { label: t("dashboard.hr.on_leave_today"), value: "on_leave" },
    ];

    const columns: ChartTableColumn<Row>[] = [
        { header: t("common.date"), key: "label" },
        { header: t("dashboard.hr.present_today"), key: "present", numeric: true },
        { header: t("dashboard.hr.late_today"), key: "late", numeric: true },
        { header: t("dashboard.hr.absent_today"), key: "absent", numeric: true },
        { header: t("dashboard.hr.on_leave_today"), key: "on_leave", numeric: true },
        { header: t("dashboard.hr.eligible"), key: "eligible", numeric: true },
        { header: t("dashboard.hr.attendance_rate"), key: "attendance_rate", numeric: true, render: (r) => rateValue(r.attendance_rate) },
        { header: t("dashboard.hr.absence_rate"), key: "absence_rate", numeric: true, render: (r) => rateValue(r.absence_rate) },
        { header: t("dashboard.hr.late_rate"), key: "late_rate", numeric: true, render: (r) => rateValue(r.late_rate) },
    ];

    const summary = t("dashboard.hr.attendance_trend_summary", {
        metric: options.find((o) => o.value === metric)?.label ?? metric,
    });

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="flex items-center justify-between mb-8">
                <div className="flex items-center gap-4">
                    <div className="bg-emerald-50 dark:bg-emerald-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-emerald-600 dark:text-emerald-400 shadow-inner border border-emerald-100 dark:border-emerald-800">
                        <TrendingUp size={22} />
                    </div>
                    <div>
                        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h2">
                            {t("dashboard.hr.attendance_trend")}
                        </Typography>
                        <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
                            {t("dashboard.hr.last_30_days")}
                        </Typography>
                    </div>
                </div>
                <MetricSelector
                    aria-label={t("dashboard.hr.attendance_trend")}
                    onChange={setMetric}
                    options={options}
                    value={metric}
                />
            </div>

            {isLoading ? (
                <div className="h-[280px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : error ? (
                <DashboardChartError icon={<TrendingUp size={48} />} />
            ) : !hasData ? (
                <div className="h-[280px] flex flex-col items-center justify-center">
                    <TrendingUp className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
                    <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
                        {t("common.no_data")}
                    </Typography>
                </div>
            ) : (
                <>
                    <ChartSummary text={summary} />
                    <div aria-label={summary} role="img">
                        <ResponsiveContainer height={280} width="100%">
                            <LineChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                                <CartesianGrid className="text-slate-100 dark:text-slate-800" stroke="currentColor" strokeDasharray="3 3" />
                                <XAxis
                                    axisLine={false}
                                    className="text-slate-400 dark:text-slate-500"
                                    dataKey="label"
                                    interval="preserveStartEnd"
                                    tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                                    tickLine={false}
                                />
                                <YAxis
                                    allowDecimals={false}
                                    axisLine={false}
                                    className="text-slate-400 dark:text-slate-500"
                                    tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                                    tickLine={false}
                                />
                                <Tooltip
                                    contentStyle={{
                                        borderRadius: "1rem",
                                        border: "none",
                                        boxShadow: "0 8px 30px rgba(0,0,0,.1)",
                                        fontSize: "11px",
                                        fontWeight: 600,
                                    }}
                                />
                                <Legend
                                    wrapperStyle={{ fontSize: "9px", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.15em" }}
                                />
                                <Line
                                    dataKey={metric}
                                    dot={{ r: 3 }}
                                    name={options.find((o) => o.value === metric)?.label}
                                    stroke={COLORS[metric]}
                                    strokeWidth={2.5}
                                    type="monotone"
                                />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                    <ChartTableTwin
                        caption={t("dashboard.hr.attendance_trend")}
                        columns={columns}
                        rows={chartData}
                        searchableKeys={["label"]}
                    />
                </>
            )}
        </div>
    );
}
