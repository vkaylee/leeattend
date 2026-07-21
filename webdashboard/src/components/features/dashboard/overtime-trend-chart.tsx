import { Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
    AreaChart,
    Area,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    Legend,
} from "recharts";

import { Typography } from "@/components/ui/typography";
import { useOvertimeTrend } from "@/hooks/use-dashboard";

const COLORS = {
    system: "#f97316",   // orange-500
    approved: "#10b981", // emerald-500
};

function getOvertimeChartData(trend: { approved_ot_hours: number; date: string; system_ot_hours: number }[] | undefined) {
    const chartData = (trend ?? []).map((point) => ({
        ...point,
        label: new Date(point.date).toLocaleDateString(undefined, {
            month: "short",
            day: "numeric",
        }),
    }));

    const hasData =
        chartData.length > 0 &&
        chartData.some((d) => d.system_ot_hours > 0 || d.approved_ot_hours > 0);

    return { chartData, hasData };
}

function OvertimeHeader({ t }: { t: (k: string) => string }) {
    return (
        <div className="flex items-center justify-between mb-8">
            <div className="flex items-center gap-4">
                <div className="bg-orange-50 dark:bg-orange-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-orange-600 dark:text-orange-400 shadow-inner border border-orange-100 dark:border-orange-800">
                    <Clock size={22} />
                </div>
                <div>
                    <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h2">
                        {t("dashboard.hr.overtime_trend")}
                    </Typography>
                    <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
                        {t("dashboard.hr.overtime_trend_subtitle")}
                    </Typography>
                </div>
            </div>
        </div>
    );
}

function OvertimeEmptyState({ t }: { t: (k: string) => string }) {
    return (
        <div className="h-[280px] flex flex-col items-center justify-center">
            <Clock className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
            <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
                {t("common.no_data")}
            </Typography>
        </div>
    );
}

export default function OvertimeTrendChart() {
    const { t } = useTranslation();
    const { data: trend, isLoading } = useOvertimeTrend();

    const { chartData, hasData } = getOvertimeChartData(trend);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <OvertimeHeader t={t} />

            {isLoading ? (
                <div className="h-[280px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <OvertimeEmptyState t={t} />
            ) : (
                <ResponsiveContainer height={280} width="100%">
                    <AreaChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                        <defs>
                            <linearGradient id="gradSystemOt" x1="0" x2="0" y1="0" y2="1">
                                <stop offset="5%" stopColor={COLORS.system} stopOpacity={0.3} />
                                <stop offset="95%" stopColor={COLORS.system} stopOpacity={0} />
                            </linearGradient>
                            <linearGradient id="gradApprovedOt" x1="0" x2="0" y1="0" y2="1">
                                <stop offset="5%" stopColor={COLORS.approved} stopOpacity={0.3} />
                                <stop offset="95%" stopColor={COLORS.approved} stopOpacity={0} />
                            </linearGradient>
                        </defs>
                        <CartesianGrid
                            className="text-slate-100 dark:text-slate-800"
                            stroke="currentColor"
                            strokeDasharray="3 3"
                        />
                        <XAxis
                            axisLine={false}
                            className="text-slate-400 dark:text-slate-500"
                            dataKey="label"
                            interval="preserveStartEnd"
                            tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                            tickLine={false}
                        />
                        <YAxis
                            axisLine={false}
                            className="text-slate-400 dark:text-slate-500"
                            tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                            tickLine={false}
                            unit="h"
                        />
                        <Tooltip
                            contentStyle={{
                                borderRadius: "1rem",
                                border: "none",
                                boxShadow: "0 8px 30px rgba(0,0,0,.1)",
                                fontSize: "11px",
                                fontWeight: 600,
                            }}
                            formatter={(value: unknown) => [`${Number(value ?? 0).toFixed(1)}h`, undefined]}
                        />
                        <Legend
                            wrapperStyle={{
                                fontSize: "9px",
                                fontWeight: 700,
                                textTransform: "uppercase",
                                letterSpacing: "0.15em",
                            }}
                        />
                        <Area
                            dataKey="system_ot_hours"
                            fill="url(#gradSystemOt)"
                            name={t("dashboard.hr.system_ot")}
                            stroke={COLORS.system}
                            strokeWidth={2}
                            type="monotone"
                        />
                        <Area
                            dataKey="approved_ot_hours"
                            fill="url(#gradApprovedOt)"
                            name={t("dashboard.hr.approved_ot")}
                            stroke={COLORS.approved}
                            strokeWidth={2}
                            type="monotone"
                        />
                    </AreaChart>
                </ResponsiveContainer>
            )}
        </div>
    );
}
