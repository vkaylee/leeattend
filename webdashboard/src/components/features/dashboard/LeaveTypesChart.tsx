import { useTranslation } from "react-i18next";
import {
    PieChart,
    Pie,
    Cell,
    ResponsiveContainer,
    Tooltip,
    Legend,
} from "recharts";
import { PieChart as PieChartIcon } from "lucide-react";
import { useLeaveTypesBreakdown } from "@/hooks/use-dashboard";

const LEAVE_COLORS = [
    "#ef4444", // red-500   — Sick
    "#3b82f6", // blue-500  — Annual
    "#ec4899", // pink-500  — Maternity
    "#f59e0b", // amber-500 — Unpaid
    "#10b981", // emerald-500
    "#10b981", // emerald-500
    "#06b6d4", // cyan-500
    "#f97316", // orange-500
];

export default function LeaveTypesChart() {
    const { t } = useTranslation();
    const { data: leaveTypes, isLoading } = useLeaveTypesBreakdown();

    const chartData = (leaveTypes ?? []).map((item, i) => ({
        ...item,
        color: LEAVE_COLORS[i % LEAVE_COLORS.length],
    }));

    const hasData = chartData.length > 0 && chartData.some((d) => d.count > 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="flex items-center gap-4 mb-8">
                <div className="bg-emerald-50 dark:bg-emerald-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-emerald-600 dark:text-emerald-400 shadow-inner border border-emerald-100 dark:border-emerald-800">
                    <PieChartIcon size={22} />
                </div>
                <div>
                    <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                        {t("dashboard.hr.leave_types")}
                    </h2>
                    <p className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5">
                        {t("dashboard.hr.leave_types_subtitle")}
                    </p>
                </div>
            </div>

            {isLoading ? (
                <div className="h-[280px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <div className="h-[280px] flex flex-col items-center justify-center">
                    <PieChartIcon size={48} className="text-slate-200 dark:text-slate-700 mb-4" />
                    <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
                        {t("common.no_data")}
                    </p>
                </div>
            ) : (
                <div className="flex flex-col items-center">
                    <ResponsiveContainer width="100%" height={260}>
                        <PieChart>
                            <Pie
                                data={chartData}
                                cx="50%"
                                cy="50%"
                                innerRadius={65}
                                outerRadius={105}
                                paddingAngle={3}
                                dataKey="count"
                                nameKey="leave_type"
                                stroke="none"
                            >
                                {chartData.map((entry, index) => (
                                    <Cell key={`cell-${index}`} fill={entry.color} />
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
                        {chartData.map((entry, i) => (
                            <div key={i} className="flex items-center gap-2">
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
