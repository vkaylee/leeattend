import { CalendarClock } from "lucide-react";
import { useState } from "react";
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
import {
    DashboardChartEmpty,
    DashboardChartError,
    DashboardChartLoading,
} from "@/components/features/dashboard/dashboard-chart-state";
import { MetricSelector } from "@/components/features/dashboard/metric-selector";
import { useDashboardChartError } from "@/components/features/dashboard/use-dashboard-chart-error";
import { Typography } from "@/components/ui/typography";
import { useLatenessByDay, type LatenessByDay } from "@/hooks/use-dashboard";

type MetricKey = "late_count" | "avg_late_minutes";

const COLORS: Record<MetricKey, string> = {
    late_count: "#f59e0b",
    avg_late_minutes: "#ef4444",
};

export default function LatenessByDayChart() {
    const { t } = useTranslation();
    const { data: days, error, isLoading } = useLatenessByDay();
    useDashboardChartError(Boolean(error));
    const [metric, setMetric] = useState<MetricKey>("late_count");

    const chartData: LatenessByDay[] = days ?? [];

    const hasData = chartData.length > 0 && chartData.some((d) => d.late_count > 0 || d.avg_late_minutes > 0);

    const options: { label: string; value: MetricKey }[] = [
        { label: t("dashboard.hr.late_count"), value: "late_count" },
        { label: t("dashboard.hr.avg_late_minutes"), value: "avg_late_minutes" },
    ];

    const columns: ChartTableColumn<LatenessByDay>[] = [
        { header: t("dashboard.hr.day_of_week"), key: "day" },
        { header: t("dashboard.hr.late_count"), key: "late_count", numeric: true },
        { header: t("dashboard.hr.avg_late_minutes"), key: "avg_late_minutes", numeric: true },
    ];

    const metricLabel = options.find((o) => o.value === metric)?.label ?? metric;
    const summary = t("dashboard.hr.lateness_by_day_summary", { metric: metricLabel });

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="flex min-w-0 flex-wrap items-center justify-between gap-4 mb-8">
                <ChartHeader />
                <MetricSelector
                    aria-label={t("dashboard.hr.lateness_by_day")}
                    onChange={setMetric}
                    options={options}
                    value={metric}
                />
            </div>

            {isLoading ? (
                <DashboardChartLoading />
            ) : error ? (
                <DashboardChartError icon={<CalendarClock size={48} />} />
            ) : !hasData ? (
                <DashboardChartEmpty icon={<CalendarClock size={48} />} />
            ) : (
                <>
                    <ChartSummary text={summary} />
                    <div aria-label={summary} role="img">
                        <ResponsiveContainer height={280} width="100%">
                            <BarChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                                <CartesianGrid
                                    className="text-slate-100 dark:text-slate-800"
                                    stroke="currentColor"
                                    strokeDasharray="3 3"
                                />
                                <XAxis
                                    axisLine={false}
                                    className="text-slate-400 dark:text-slate-500"
                                    dataKey="day"
                                    tick={{ fontSize: 10, fontWeight: 700, fill: "currentColor" }}
                                    tickLine={false}
                                />
                                <YAxis
                                    allowDecimals={metric === "avg_late_minutes"}
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
                                <Bar
                                    dataKey={metric}
                                    fill={COLORS[metric]}
                                    name={metricLabel}
                                    radius={[6, 6, 0, 0]}
                                />
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                    <ChartTableTwin
                        caption={t("dashboard.hr.lateness_by_day")}
                        columns={columns}
                        rows={chartData}
                        searchableKeys={["day"]}
                    />
                </>
            )}
        </div>
    );
}

function ChartHeader() {
    const { t } = useTranslation();
    return (
        <div className="flex items-center gap-4">
            <div className="bg-amber-50 dark:bg-amber-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-amber-600 dark:text-amber-400 shadow-inner border border-amber-100 dark:border-amber-800">
                <CalendarClock size={22} />
            </div>
            <div>
                <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h2">
                    {t("dashboard.hr.lateness_by_day")}
                </Typography>
                <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
                    {t("dashboard.hr.lateness_by_day_subtitle")}
                </Typography>
            </div>
        </div>
    );
}

