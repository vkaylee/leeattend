import { PieChart as PieChartIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
    PieChart,
    Pie,
    Cell,
    ResponsiveContainer,
    Tooltip,
    Legend,
} from "recharts";

import { Typography } from "@/components/ui/typography";
import { useLeaveTypesBreakdown } from "@/hooks/use-dashboard";

const LEAVE_COLORS = [
    "#ef4444", // red-500   — Sick
    "#3b82f6", // blue-500  — Annual
    "#ec4899", // pink-500  — Maternity
    "#f59e0b", // amber-500 — Unpaid
    "#10b981", // emerald-500
    "#06b6d4", // cyan-500
    "#8b5cf6", // violet-500
    "#f97316", // orange-500
];

export default function LeaveTypesChart() {
    const { t } = useTranslation();
    const { data: leaveTypes, isLoading } = useLeaveTypesBreakdown();

    const chartData = (leaveTypes ?? []).map((item, index) => ({
        ...item,
        color: LEAVE_COLORS[index % LEAVE_COLORS.length],
    }));

    const hasData = chartData.length > 0 && chartData.some((d) => d.count > 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="flex items-center gap-4 mb-8">
                <div className="bg-emerald-50 dark:bg-emerald-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-emerald-600 dark:text-emerald-400 shadow-inner border border-emerald-100 dark:border-emerald-800">
                    <PieChartIcon size={22} />
                </div>
                <div>
                    <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h2">
                        {t("dashboard.hr.leave_types")}
                    </Typography>
                    <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
                        {t("dashboard.hr.leave_types_subtitle")}
                    </Typography>
                </div>
            </div>

            {isLoading ? (
                <div className="h-[280px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <div className="h-[280px] flex flex-col items-center justify-center">
                    <PieChartIcon className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
                    <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
                        {t("common.no_data")}
                    </Typography>
                </div>
            ) : (
                <div className="flex flex-col items-center">
                    <ResponsiveContainer height={260} width="100%">
                        <PieChart>
                            <Pie
                                cx="50%"
                                cy="50%"
                                data={chartData}
                                dataKey="count"
                                innerRadius={65}
                                nameKey="leave_type"
                                outerRadius={105}
                                paddingAngle={3}
                                stroke="none"
                            >
                                {chartData.map((entry, index) => (
                                    // eslint-disable-next-line @typescript-eslint/no-deprecated
                                    <Cell fill={entry.color} key={`cell-${index}`} />
                                ))}
                            </Pie>
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
                        </PieChart>
                    </ResponsiveContainer>
                    <div className="flex flex-wrap justify-center gap-4 mt-2">
                        {chartData.map((entry, index) => (
                            <div className="flex items-center gap-2" key={index}>
                                <div
                                    className="w-3 h-3 rounded-full"
                                    style={{ backgroundColor: entry.color }}
                                />
                                <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                                    {entry.leave_type}: {entry.count}
                                </span>
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
