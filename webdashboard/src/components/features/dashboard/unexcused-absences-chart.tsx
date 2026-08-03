import { Building2, ShieldAlert, Users } from "lucide-react";
import { useMemo, useState } from "react";
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

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useUnexcusedAbsences } from "@/hooks/use-dashboard";
import { cn } from "@/lib/utils";

type RangePreset = "7d" | "30d" | "custom";
type GroupBy = "department" | "employee";

function toISODate(d: Date): string {
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
    if (!data) return null;

    return (
        <div className="bg-white dark:bg-slate-900 rounded-2xl p-4 border border-slate-100 dark:border-slate-800 shadow-xl min-w-[200px]">
            <Typography className="text-xs font-bold text-slate-700 dark:text-slate-200 mb-2" variant="p">
                {label}
            </Typography>
            <div className="space-y-1.5">
                <TooltipRow color="text-red-500" label={t("dashboard.hr.unexcused")} value={data.unexcused_count ?? 0} />
                <TooltipRow color="text-orange-500" label={t("dashboard.hr.consecutive_days")} value={data.consecutive_count ?? 0} />
                <div className="border-t border-slate-100 dark:border-slate-800 pt-1.5 mt-1.5">
                    <TooltipRow color="text-slate-500" label={t("dashboard.hr.total_absent")} value={data.total_absent_count ?? 0} />
                </div>
            </div>
        </div>
    );
}

function TooltipRow({ label, value, color }: { color: string; label: string; value: number }) {
    return (
        <div className="flex justify-between text-[11px]">
            <span className={cn(color, "font-semibold")}>{label}</span>
            <span className="font-bold text-slate-700 dark:text-slate-200">{value}</span>
        </div>
    );
}

interface TotalLabelProps {
    chartData: (ChartDataEntry & { name: string; remaining_unexcused: number; unexcused_count: number })[];
    height?: number;
    index?: number;
    width?: number;
    x?: number;
    y?: number;
}

function TotalLabel({ x = 0, y = 0, width = 0, height = 0, index = 0, chartData }: TotalLabelProps) {
    const row = chartData[index];
    const total = row.unexcused_count;
    if (!total || total === 0) return null;
    return (
        <text
            dominantBaseline="middle"
            fill="#64748b"
            fontSize={9}
            fontWeight={700}
            x={x + width + 4}
            y={y + height / 2}
        >
            {total}
        </text>
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
            return { startDate: toISODate(s), endDate: toISODate(now) };
        }
        if (preset === "30d") {
            const s = new Date(now);
            s.setDate(s.getDate() - 29);
            return { startDate: toISODate(s), endDate: toISODate(now) };
        }
        // custom — default to 30 days
        const s = new Date(now);
        s.setDate(s.getDate() - 29);
        return { startDate: toISODate(s), endDate: toISODate(now) };
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

    const chartHeight = Math.max(320, chartData.length * 40 + 80);

    const presets: { key: RangePreset; label: string }[] = [
        { key: "7d", label: t("dashboard.hr.last_7_days") },
        { key: "30d", label: t("dashboard.hr.last_30_days_range") },
    ];

    const groupByOptions: { key: GroupBy; label: string; icon: typeof Building2 }[] = [
        { key: "department", label: t("dashboard.hr.group_by_department"), icon: Building2 },
        { key: "employee", label: t("dashboard.hr.group_by_employee"), icon: Users },
    ];

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <ChartHeader
                groupBy={groupBy}
                groupByOptions={groupByOptions}
                preset={preset}
                presets={presets}
                setGroupBy={setGroupBy}
                setPreset={setPreset}
            />

            {isLoading ? (
                <ChartLoading />
            ) : !hasData ? (
                <ChartEmpty />
            ) : (
                <ChartMain chartData={chartData} chartHeight={chartHeight} />
            )}
        </div>
    );
}

interface ChartHeaderProps {
    groupBy: GroupBy;
    groupByOptions: { icon: typeof Building2; key: GroupBy; label: string }[];
    preset: RangePreset;
    presets: { key: RangePreset; label: string }[];
    setGroupBy: (g: GroupBy) => void;
    setPreset: (p: RangePreset) => void;
}

function ChartHeader({
    presets,
    preset,
    setPreset,
    groupByOptions,
    groupBy,
    setGroupBy,
}: ChartHeaderProps) {
    const { t } = useTranslation();
    return (
        <div className="flex items-center justify-between mb-8">
            <div className="flex items-center gap-4">
                <div className="bg-red-50 dark:bg-red-950 w-12 h-12 rounded-[1rem] flex items-center justify-center text-red-600 dark:text-red-400 shadow-inner border border-red-100 dark:border-red-800">
                    <ShieldAlert size={22} />
                </div>
                <div>
                    <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h2">
                        {t("dashboard.hr.unexcused_absences")}
                    </Typography>
                    <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
                        {t("dashboard.hr.unexcused_absences_subtitle")}
                    </Typography>
                </div>
            </div>

            <div className="flex items-center gap-3">
                <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1 rounded-[14px] border border-slate-200 dark:border-slate-700/50 shadow-inner">
                    {groupByOptions.map((g) => {
                        const Icon = g.icon;
                        return (
                            <Button
                                className={cn(
                                    "px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest transition-all flex items-center gap-1.5 h-auto shadow-none border border-transparent",
                                    groupBy === g.key
                                        ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900"
                                        : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent"
                                )}
                                key={g.key}
                                onClick={() => { setGroupBy(g.key); }}
                                type="button"
                                variant="ghost"
                            >
                                <Icon size={12} />
                                {g.label}
                            </Button>
                        );
                    })}
                </div>

                <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1 rounded-[14px] border border-slate-200 dark:border-slate-700/50 shadow-inner">
                    {presets.map((p) => (
                        <Button
                            className={cn(
                                "px-5 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest transition-all h-auto shadow-none border border-transparent",
                                preset === p.key
                                    ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900"
                                    : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent"
                            )}
                            key={p.key}
                            onClick={() => { setPreset(p.key); }}
                            type="button"
                            variant="ghost"
                        >
                            {p.label}
                        </Button>
                    ))}
                </div>
            </div>
        </div>
    );
}

function ChartLoading() {
    return (
        <div className="h-[320px] flex items-center justify-center">
            <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
        </div>
    );
}

function ChartEmpty() {
    const { t } = useTranslation();
    return (
        <div className="h-[320px] flex flex-col items-center justify-center">
            <ShieldAlert className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
            <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
                {t("common.no_data")}
            </Typography>
        </div>
    );
}

function ChartMain({ chartData, chartHeight }: { chartData: (ChartDataEntry & { name: string; remaining_unexcused: number; unexcused_count: number })[]; chartHeight: number }) {
    const { t } = useTranslation();
    return (
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
                    label={<TotalLabel chartData={chartData} />}
                    name={t("dashboard.hr.unexcused")}
                    radius={[0, 4, 4, 0]}
                    stackId="absences"
                />
            </BarChart>
        </ResponsiveContainer>
    );
}
