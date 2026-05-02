import { Building2 } from "lucide-react";
import { useState } from "react";
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

import { useDepartmentAttendance } from "@/hooks/use-dashboard";
import { cn } from "@/lib/utils";

const COLORS = {
    on_time: "#10b981",
    late: "#f59e0b",
    absent: "#ef4444",
    on_leave: "#6366f1",
};

export default function DepartmentAttendanceChart() {
    const { t } = useTranslation();
    const [range, setRange] = useState<"today" | "week">("today");
    const { data, isLoading } = useDepartmentAttendance(range);

    const chartData = (data ?? []).map((d) => ({
        ...d,
        name: d.department_name,
    }));

    const hasData =
        chartData.length > 0 &&
        chartData.some((d) => d.on_time > 0 || d.late > 0 || d.absent > 0 || d.on_leave > 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="flex items-center justify-between mb-8">
                <div className="flex items-center gap-4">
                    <div className="bg-indigo-50 dark:bg-indigo-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-indigo-600 dark:text-indigo-400 shadow-inner border border-indigo-100 dark:border-indigo-800">
                        <Building2 size={22} />
                    </div>
                    <div>
                        <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                            {t("dashboard.hr.dept_attendance")}
                        </h2>
                        <p className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5">
                            {t("dashboard.hr.dept_attendance_subtitle")}
                        </p>
                    </div>
                </div>

                <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1 rounded-[14px] border border-slate-200 dark:border-slate-700/50 shadow-inner">
                    <button
                        className={cn(
                            "px-5 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest transition-all",
                            range === "today"
                                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                        )}
                        onClick={() => { setRange("today"); }}
                    >
                        {t("dashboard.hr.today")}
                    </button>
                    <button
                        className={cn(
                            "px-5 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest transition-all",
                            range === "week"
                                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                        )}
                        onClick={() => { setRange("week"); }}
                    >
                        {t("dashboard.hr.this_week")}
                    </button>
                </div>
            </div>

            {isLoading ? (
                <div className="h-[320px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <div className="h-[320px] flex flex-col items-center justify-center">
                    <Building2 className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
                    <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
                        {t("common.no_data")}
                    </p>
                </div>
            ) : (
                <ResponsiveContainer height={320} width="100%">
                    <BarChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                        <CartesianGrid
                            className="text-slate-100 dark:text-slate-800"
                            stroke="currentColor"
                            strokeDasharray="3 3"
                        />
                        <XAxis
                            axisLine={false}
                            className="text-slate-400 dark:text-slate-500"
                            dataKey="name"
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
                            wrapperStyle={{
                                fontSize: "9px",
                                fontWeight: 700,
                                textTransform: "uppercase",
                                letterSpacing: "0.15em",
                            }}
                        />
                        <Bar
                            dataKey="on_time"
                            fill={COLORS.on_time}
                            name={t("dashboard.hr.on_time")}
                            radius={[4, 4, 0, 0]}
                        />
                        <Bar
                            dataKey="late"
                            fill={COLORS.late}
                            name={t("dashboard.hr.late_today")}
                            radius={[4, 4, 0, 0]}
                        />
                        <Bar
                            dataKey="absent"
                            fill={COLORS.absent}
                            name={t("dashboard.hr.absent_today")}
                            radius={[4, 4, 0, 0]}
                        />
                        <Bar
                            dataKey="on_leave"
                            fill={COLORS.on_leave}
                            name={t("dashboard.hr.on_leave_today")}
                            radius={[4, 4, 0, 0]}
                        />
                    </BarChart>
                </ResponsiveContainer>
            )}
        </div>
    );
}
