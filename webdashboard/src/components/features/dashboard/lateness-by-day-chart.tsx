import { CalendarClock } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
    Bar,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    Legend,
    Line,
    ComposedChart,
} from "recharts";

import { Typography } from "@/components/ui/typography";
import { useLatenessByDay } from "@/hooks/use-dashboard";

const COLORS = {
    late_count: "#f59e0b",       // amber-500
    avg_minutes: "#ef4444",      // rose-500
};

function ChartGradients() {
    return (
        <defs>
            <linearGradient id="gradLateCount" x1="0" x2="0" y1="0" y2="1">
                <stop offset="5%" stopColor={COLORS.late_count} stopOpacity={0.8} />
                <stop offset="95%" stopColor={COLORS.late_count} stopOpacity={0.4} />
            </linearGradient>
        </defs>
    );
}

export default function LatenessByDayChart() {
    const { data: days, isLoading } = useLatenessByDay();

    const chartData = days ?? [];

    const hasData =
        chartData.length > 0 &&
        chartData.some((d) => d.late_count > 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <ChartHeader />

            {isLoading ? (
                <ChartLoading />
            ) : !hasData ? (
                <ChartEmpty />
            ) : (
                <ChartMain chartData={chartData} />
            )}
        </div>
    );
}

function ChartHeader() {
    const { t } = useTranslation();
    return (
        <div className="flex items-center gap-4 mb-8">
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

function ChartLoading() {
    return (
        <div className="h-[280px] flex items-center justify-center">
            <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
        </div>
    );
}

function ChartEmpty() {
    const { t } = useTranslation();
    return (
        <div className="h-[280px] flex flex-col items-center justify-center">
            <CalendarClock className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
            <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
                {t("common.no_data")}
            </Typography>
        </div>
    );
}

function ChartMain({ chartData }: { chartData: { avg_late_minutes: number; day: string; late_count: number }[] }) {
    const { t } = useTranslation();
    return (
        <ResponsiveContainer height={280} width="100%">
            <ComposedChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                <ChartGradients />
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
                    allowDecimals={false}
                    axisLine={false}
                    className="text-slate-400 dark:text-slate-500"
                    tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                    tickLine={false}
                    yAxisId="left"
                />
                <YAxis
                    axisLine={false}
                    className="text-slate-400 dark:text-slate-500"
                    orientation="right"
                    tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                    tickLine={false}
                    unit="m"
                    yAxisId="right"
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
                    wrapperStyle={{
                        fontSize: "9px",
                        fontWeight: 700,
                        textTransform: "uppercase",
                        letterSpacing: "0.15em",
                    }}
                />
                <Bar
                    dataKey="late_count"
                    fill="url(#gradLateCount)"
                    name={t("dashboard.hr.late_count")}
                    radius={[6, 6, 0, 0]}
                    yAxisId="left"
                />
                <Line
                    activeDot={{ r: 6 }}
                    dataKey="avg_late_minutes"
                    dot={{ r: 4, fill: COLORS.avg_minutes }}
                    name={t("dashboard.hr.avg_late_minutes")}
                    stroke={COLORS.avg_minutes}
                    strokeWidth={2.5}
                    type="monotone"
                    yAxisId="right"
                />
            </ComposedChart>
        </ResponsiveContainer>
    );
}
