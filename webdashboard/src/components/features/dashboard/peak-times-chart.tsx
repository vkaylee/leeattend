import { BarChart3 } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
    BarChart,
    Bar,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    Legend,
} from "recharts";

import { DashboardChartError } from "@/components/features/dashboard/dashboard-chart-state";
import { Typography } from "@/components/ui/typography";
import { usePeakTimes } from "@/hooks/use-dashboard";

const COLORS = {
    check_in: "#06b6d4",  // cyan-500
    check_out: "#10b981", // emerald-500
};

function ChartGradients() {
    return (
        <defs>
            <linearGradient id="gradCheckIn" x1="0" x2="0" y1="0" y2="1">
                <stop offset="5%" stopColor={COLORS.check_in} stopOpacity={0.8} />
                <stop offset="95%" stopColor={COLORS.check_in} stopOpacity={0.4} />
            </linearGradient>
            <linearGradient id="gradCheckOut" x1="0" x2="0" y1="0" y2="1">
                <stop offset="5%" stopColor={COLORS.check_out} stopOpacity={0.8} />
                <stop offset="95%" stopColor={COLORS.check_out} stopOpacity={0.4} />
            </linearGradient>
        </defs>
    );
}

export default function PeakTimesChart() {
    const { data: slots, error, isLoading } = usePeakTimes();

    const chartData = slots ?? [];

    const hasData =
        chartData.length > 0 &&
        chartData.some((d) => d.check_in > 0 || d.check_out > 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <ChartHeader />

            {isLoading ? (
                <ChartLoading />
            ) : error ? (
                <DashboardChartError icon={<BarChart3 size={48} />} />
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
        <div className="flex items-center justify-between mb-8">
            <div className="flex items-center gap-4">
                <div className="bg-cyan-50 dark:bg-cyan-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-cyan-600 dark:text-cyan-400 shadow-inner border border-cyan-100 dark:border-cyan-800">
                    <BarChart3 size={22} />
                </div>
                <div>
                    <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h2">
                        {t("dashboard.hr.peak_times")}
                    </Typography>
                    <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
                        {t("dashboard.hr.peak_times_subtitle")}
                    </Typography>
                </div>
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
            <BarChart3 className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
            <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
                {t("common.no_data")}
            </Typography>
        </div>
    );
}

function ChartMain({ chartData }: { chartData: { check_in: number; check_out: number; time_slot: string }[] }) {
    const { t } = useTranslation();
    return (
        <ResponsiveContainer height={280} width="100%">
            <BarChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                <ChartGradients />
                <CartesianGrid
                    className="text-slate-100 dark:text-slate-800"
                    stroke="currentColor"
                    strokeDasharray="3 3"
                />
                <XAxis
                    angle={-35}
                    axisLine={false}
                    className="text-slate-400 dark:text-slate-500"
                    dataKey="time_slot"
                    height={50}
                    interval={0}
                    textAnchor="end"
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
                    formatter={(value: unknown) => [
                        `${Number(value ?? 0)} ${t("dashboard.hr.people")}`,
                        undefined,
                    ]}
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
                    dataKey="check_in"
                    fill="url(#gradCheckIn)"
                    name={t("dashboard.hr.check_in_label")}
                    radius={[4, 4, 0, 0]}
                />
                <Bar
                    dataKey="check_out"
                    fill="url(#gradCheckOut)"
                    name={t("dashboard.hr.check_out_label")}
                    radius={[4, 4, 0, 0]}
                />
            </BarChart>
        </ResponsiveContainer>
    );
}
