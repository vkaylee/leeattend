import { ShieldAlert, Building2, Users } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
    BarChart,
    Bar,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
} from "recharts";

import { useUnexcusedAbsences } from "@/hooks/use-dashboard";
import { cn } from "@/lib/utils";

type RangePreset = "7d" | "30d" | "custom";
type GroupBy = "department" | "employee";

function formatDate(d: Date): string {
    return d.toISOString().slice(0, 10);
}

const COLORS = {
    unexcused: "#ef4444",
    consecutive: "#f97316",
};

 
interface ChartDataEntry {
    consecutive_count?: number;
    total_absent_count?: number;
    unexcused_count?: number;
}

interface CustomTooltipProps {
    active?: boolean;
    label?: string;
    payload?: { payload: ChartDataEntry | null }[];
}

function CustomTooltip({ active, payload, label }: CustomTooltipProps) {
    const { t } = useTranslation();
    if (!active || !payload?.length) return null;

    const data = payload[0]?.payload;
    return (
        <div className="bg-white dark:bg-slate-900 rounded-2xl p-4 border border-slate-100 dark:border-slate-800 shadow-xl min-w-[200px]">
            <p className="text-xs font-bold text-slate-700 dark:text-slate-200 mb-2">
                {label}
            </p>
            <div className="space-y-1.5">
                <div className="flex justify-between text-[11px]">
                    <span className="text-red-500 font-semibold">{t("dashboard.hr.unexcused")}</span>
                    <span className="font-bold text-slate-700 dark:text-slate-200">{data?.unexcused_count ?? 0}</span>
                </div>

                <div className="flex justify-between text-[11px]">
                    <span className="text-orange-500 font-semibold">{t("dashboard.hr.consecutive_days")}</span>
                    <span className="font-bold text-slate-700 dark:text-slate-200">{data?.consecutive_count ?? 0}</span>
                </div>
                <div className="border-t border-slate-100 dark:border-slate-800 pt-1.5 mt-1.5 flex justify-between text-[11px]">
                    <span className="text-slate-500 font-semibold">{t("dashboard.hr.total_absent")}</span>
                    <span className="font-bold text-slate-700 dark:text-slate-200">{data?.total_absent_count ?? 0}</span>
                </div>
            </div>
        </div>
    );
}

export default function UnexcusedAbsencesChart() {
    const { t } = useTranslation();
    const [preset, setPreset] = useState<RangePreset>("7d");
    const [groupBy, setGroupBy] = useState<GroupBy>("department");

    const { startDate, endDate } = useMemo(() => {
        const now = new Date();
        if (preset === "7d") {
            const s = new Date(now);
            s.setDate(s.getDate() - 6);
            return { startDate: formatDate(s), endDate: formatDate(now) };
        }
        if (preset === "30d") {
            const s = new Date(now);
            s.setDate(s.getDate() - 29);
            return { startDate: formatDate(s), endDate: formatDate(now) };
        }
        // custom — default to 30 days
        const s = new Date(now);
        s.setDate(s.getDate() - 29);
        return { startDate: formatDate(s), endDate: formatDate(now) };
    }, [preset]);

    const { data, isLoading } = useUnexcusedAbsences(startDate, endDate, groupBy);

    const chartData = (data ?? [])
        .map((d) => ({
            ...d,
            name: groupBy === "employee" ? (d.employee_name ?? d.department_name) : d.department_name,
            remaining_unexcused: Math.max(0, d.unexcused_count - d.consecutive_count),
        }))
        .filter((d) => d.unexcused_count >= 1);

    const hasData = chartData.length > 0 && chartData.some((d) => d.unexcused_count > 0);

    const presets: { key: RangePreset; label: string }[] = [
        { key: "7d", label: t("dashboard.hr.last_7_days") },
        { key: "30d", label: t("dashboard.hr.last_30_days_range") },
    ];

    const groupByOptions: { key: GroupBy; label: string; icon: typeof Building2 }[] = [
        { key: "department", label: t("dashboard.hr.group_by_department"), icon: Building2 },
        { key: "employee", label: t("dashboard.hr.group_by_employee"), icon: Users },
    ];

    // Dynamic chart height based on number of entries for horizontal layout
    const chartHeight = Math.max(320, chartData.length * 40 + 80);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="flex items-center justify-between mb-8">
                <div className="flex items-center gap-4">
                    <div className="bg-red-50 dark:bg-red-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-red-600 dark:text-red-400 shadow-inner border border-red-100 dark:border-red-800">
                        <ShieldAlert size={22} />
                    </div>
                    <div>
                        <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                            {t("dashboard.hr.unexcused_absences")}
                        </h2>
                        <p className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5">
                            {t("dashboard.hr.unexcused_absences_subtitle")}
                        </p>
                    </div>
                </div>

                <div className="flex items-center gap-3">
                    {/* Group-by toggle */}
                    <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1 rounded-[14px] border border-slate-200 dark:border-slate-700/50 shadow-inner">
                        {groupByOptions.map((g) => {
                            const Icon = g.icon;
                            return (
                                <button
                                    className={cn(
                                        "px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest transition-all flex items-center gap-1.5",
                                        groupBy === g.key
                                            ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                                            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                                    )}
                                    key={g.key}
                                    onClick={() => { setGroupBy(g.key); }}
                                >
                                    <Icon size={12} />
                                    {g.label}
                                </button>
                            );
                        })}
                    </div>

                    {/* Time-range toggle */}
                    <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1 rounded-[14px] border border-slate-200 dark:border-slate-700/50 shadow-inner">
                        {presets.map((p) => (
                            <button
                                className={cn(
                                    "px-5 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest transition-all",
                                    preset === p.key
                                        ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                                        : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                                )}
                                key={p.key}
                                onClick={() => { setPreset(p.key); }}
                            >
                                {p.label}
                            </button>
                        ))}
                    </div>
                </div>
            </div>


            {isLoading ? (
                <div className="h-[320px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <div className="h-[320px] flex flex-col items-center justify-center">
                    <ShieldAlert className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
                    <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
                        {t("common.no_data")}
                    </p>
                </div>
            ) : (
                <ResponsiveContainer height={chartHeight} width="100%">
                    <BarChart
                        data={chartData}
                        layout="vertical"
                        margin={{ top: 5, right: 30, left: 10, bottom: 5 }}
                    >
                        <CartesianGrid
                            className="text-slate-100 dark:text-slate-800"
                            horizontal={false}
                            stroke="currentColor"
                            strokeDasharray="3 3"
                        />
                        <XAxis
                            allowDecimals={false}
                            axisLine={false}
                            className="text-slate-400 dark:text-slate-500"
                            tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                            tickLine={false}
                            type="number"
                        />
                        <YAxis
                            axisLine={false}
                            className="text-slate-400 dark:text-slate-500"
                            dataKey="name"
                            tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                            tickLine={false}
                            type="category"
                            width={120}
                        />
                        <Tooltip content={<CustomTooltip />} cursor={{ fill: "rgba(0,0,0,0.04)" }} />
                        <Bar
                            dataKey="consecutive_count"
 
                            fill={COLORS.consecutive}
                            label={false}
                            name={t("dashboard.hr.consecutive_days")}
                            radius={[0, 0, 0, 0]}
                            stackId="absences"
                        />
                        <Bar
                            dataKey="remaining_unexcused"
                            fill={COLORS.unexcused}
                            label={(props: unknown) => {
                                const { x, y, width, height, index } = props as { x: number; y: number; width: number; height: number; index: number };
                                const row = chartData[index];
                                const total = row?.unexcused_count ?? 0;
                                if (!total || total === 0) return null;
                                return (
                                    <text dominantBaseline="middle" fill="#64748b" fontSize={9} fontWeight={700} x={x + width + 4} y={y + height / 2}>
                                        {total}
                                    </text>
                                );
                            }}
                            name={t("dashboard.hr.unexcused")}
                            radius={[0, 4, 4, 0]}
                            stackId="absences"
                        />
                    </BarChart>
                </ResponsiveContainer>
            )}
        </div>
    );
}
