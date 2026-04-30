import { CalendarClock } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
    _BarChart,
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

import { useLatenessByDay } from "@/hooks/use-dashboard";

const COLORS = {
    late_count: "#f59e0b",       // amber-500
    avg_minutes: "#ef4444",      // rose-500
};

export default function LatenessByDayChart() {
    const { t } = useTranslation();
    const { data: days, isLoading } = useLatenessByDay();

    const chartData = days ?? [];

    const hasData =
        chartData.length > 0 &&
        chartData.some((d) => d.late_count > 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="flex items-center gap-4 mb-8">
                <div className="bg-amber-50 dark:bg-amber-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-amber-600 dark:text-amber-400 shadow-inner border border-amber-100 dark:border-amber-800">
                    <CalendarClock size={22} />
                </div>
                <div>
                    <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                        {t("dashboard.hr.lateness_by_day")}
                    </h2>
                    <p className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5">
                        {t("dashboard.hr.lateness_by_day_subtitle")}
                    </p>
                </div>
            </div>

            {isLoading ? (
                <div className="h-[280px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <div className="h-[280px] flex flex-col items-center justify-center">
                    <CalendarClock size={48} className="text-slate-200 dark:text-slate-700 mb-4" />
                    <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
                        {t("common.no_data")}
                    </p>
                </div>
            ) : (
                <ResponsiveContainer width="100%" height={280}>
                    <ComposedChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                        <defs>
                            <linearGradient id="gradLateCount" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor={COLORS.late_count} stopOpacity={0.8} />
                                <stop offset="95%" stopColor={COLORS.late_count} stopOpacity={0.4} />
                            </linearGradient>
                        </defs>
                        <CartesianGrid
                            strokeDasharray="3 3"
                            stroke="currentColor"
                            className="text-slate-100 dark:text-slate-800"
                        />
                        <XAxis
                            dataKey="day"
                            tick={{ fontSize: 10, fontWeight: 700, fill: "currentColor" }}
                            className="text-slate-400 dark:text-slate-500"
                            tickLine={false}
                            axisLine={false}
                        />
                        <YAxis
                            yAxisId="left"
                            tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                            className="text-slate-400 dark:text-slate-500"
                            tickLine={false}
                            axisLine={false}
                            allowDecimals={false}
                        />
                        <YAxis
                            yAxisId="right"
                            orientation="right"
                            tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                            className="text-slate-400 dark:text-slate-500"
                            tickLine={false}
                            axisLine={false}
                            unit="m"
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
                            yAxisId="left"
                            dataKey="late_count"
                            name={t("dashboard.hr.late_count")}
                            fill="url(#gradLateCount)"
                            radius={[6, 6, 0, 0]}
                        />
                        <Line
                            yAxisId="right"
                            type="monotone"
                            dataKey="avg_late_minutes"
                            name={t("dashboard.hr.avg_late_minutes")}
                            stroke={COLORS.avg_minutes}
                            strokeWidth={2.5}
                            dot={{ r: 4, fill: COLORS.avg_minutes }}
                            activeDot={{ r: 6 }}
                        />
                    </ComposedChart>
                </ResponsiveContainer>
            )}
        </div>
    );
}
