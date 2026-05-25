import { TrendingUp } from "lucide-react";
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
import { useAttendanceTrend } from "@/hooks/use-dashboard";

const COLORS = {
    present: "#10b981",
    late: "#f59e0b",
    absent: "#ef4444",
    on_leave: "#6366f1",
};

interface AttendanceTrendChartAreaProps {
    chartData: {
        present: number;
        late: number;
        absent: number;
        on_leave: number;
        label: string;
    }[];
    t: (k: string) => string;
}

function AttendanceTrendChartArea({ chartData, t }: AttendanceTrendChartAreaProps) {
    return (
        <ResponsiveContainer height={280} width="100%">
            <AreaChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                <defs>
                    <linearGradient id="gradPresent" x1="0" x2="0" y1="0" y2="1">
                        <stop offset="5%" stopColor={COLORS.present} stopOpacity={0.3} />
                        <stop offset="95%" stopColor={COLORS.present} stopOpacity={0} />
                    </linearGradient>
                    <linearGradient id="gradLate" x1="0" x2="0" y1="0" y2="1">
                        <stop offset="5%" stopColor={COLORS.late} stopOpacity={0.3} />
                        <stop offset="95%" stopColor={COLORS.late} stopOpacity={0} />
                    </linearGradient>
                    <linearGradient id="gradAbsent" x1="0" x2="0" y1="0" y2="1">
                        <stop offset="5%" stopColor={COLORS.absent} stopOpacity={0.3} />
                        <stop offset="95%" stopColor={COLORS.absent} stopOpacity={0} />
                    </linearGradient>
                    <linearGradient id="gradLeave" x1="0" x2="0" y1="0" y2="1">
                        <stop offset="5%" stopColor={COLORS.on_leave} stopOpacity={0.3} />
                        <stop offset="95%" stopColor={COLORS.on_leave} stopOpacity={0} />
                    </linearGradient>
                </defs>
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
                <Area
                    dataKey="present"
                    fill="url(#gradPresent)"
                    name={t("dashboard.hr.present_today")}
                    stroke={COLORS.present}
                    strokeWidth={2}
                    type="monotone"
                />
                <Area
                    dataKey="late"
                    fill="url(#gradLate)"
                    name={t("dashboard.hr.late_today")}
                    stroke={COLORS.late}
                    strokeWidth={2}
                    type="monotone"
                />
                <Area
                    dataKey="absent"
                    fill="url(#gradAbsent)"
                    name={t("dashboard.hr.absent_today")}
                    stroke={COLORS.absent}
                    strokeWidth={2}
                    type="monotone"
                />
                <Area
                    dataKey="on_leave"
                    fill="url(#gradLeave)"
                    name={t("dashboard.hr.on_leave_today")}
                    stroke={COLORS.on_leave}
                    strokeWidth={2}
                    type="monotone"
                />
            </AreaChart>
        </ResponsiveContainer>
    );
}

export default function AttendanceTrendChart() {
    const { t } = useTranslation();
    const { data: trend, isLoading } = useAttendanceTrend();

    const chartData = (trend ?? []).map((point) => ({
        ...point,
        // Short date label: "02/01" or "01 Feb"
        label: new Date(point.date).toLocaleDateString(undefined, {
            month: "short",
            day: "numeric",
        }),
    }));

    const hasData = chartData.length > 0 && chartData.some((d) => d.present > 0 || d.late > 0 || d.absent > 0 || d.on_leave > 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
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
            </div>

            {isLoading ? (
                <div className="h-[280px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <div className="h-[280px] flex flex-col items-center justify-center">
                    <TrendingUp className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
                    <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
                        {t("common.no_data")}
                    </Typography>
                </div>
            ) : (
                <AttendanceTrendChartArea chartData={chartData} t={t} />
            )}
        </div>
    );
}
