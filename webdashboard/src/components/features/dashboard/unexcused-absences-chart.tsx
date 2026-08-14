import { Building2, ShieldAlert, Users } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
    Bar,
    BarChart,
    CartesianGrid,
    Legend,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";

import Button from "@/components/ui/button";
import PermissionGate from "@/components/common/permission-gate";
import { Typography } from "@/components/ui/typography";
import { DashboardChartError, useDashboardChartError } from "@/components/features/dashboard/dashboard-chart-state";
import { ChartSummary } from "@/components/features/dashboard/chart-summary";
import { ChartTableTwin, type ChartTableColumn } from "@/components/features/dashboard/chart-table-twin";
import { DashboardDateRange } from "@/components/features/dashboard/dashboard-date-range";
import { useUnexcusedAbsences, type UnexcusedAbsencesPoint } from "@/hooks/use-dashboard";
import { useWorkspace } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";

const CHART_HEIGHT = 320;
const TOP_N = 10;
const EMPLOYEE_ABSENCE_READ = "dashboard:employee_absence:read";

type GroupBy = "department" | "employee";

function defaultRange(): { startDate: string; endDate: string } {
    const now = new Date();
    const end = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    const s = new Date(now);
    s.setDate(s.getDate() - 29);
    const start = `${s.getFullYear()}-${String(s.getMonth() + 1).padStart(2, "0")}-${String(s.getDate()).padStart(2, "0")}`;
    return { startDate: start, endDate: end };
}

const COLORS = {
    unexcused: "#ef4444",
    consecutive: "#f97316",
};

interface Row extends UnexcusedAbsencesPoint {
    name: string;
    remaining_unexcused: number;
}

export default function UnexcusedAbsencesChart() {
    const { t } = useTranslation();
    const { workspace } = useWorkspace();
    const [groupBy, setGroupBy] = useState<GroupBy>("department");
    const [range, setRange] = useState(defaultRange);

    const timezone = workspace?.timezone ?? undefined;
    const { data, error, isLoading } = useUnexcusedAbsences(
        range.startDate,
        range.endDate,
        groupBy,
        timezone,
    );
    useDashboardChartError(Boolean(error));

    const allRows: Row[] = useMemo(
        () =>
            (data ?? [])
                .map((d) => ({
                    ...d,
                    name: groupBy === "employee" ? (d.employee_name ?? d.department_name) : d.department_name,
                    remaining_unexcused: Math.max(0, d.unexcused_count - d.consecutive_count),
                }))
                .filter((d) => d.unexcused_count >= 1),
        [data, groupBy],
    );

    // Top-N for the chart; full result remains available in the table twin.
    const chartData = useMemo(() => allRows.slice(0, TOP_N), [allRows]);

    const hasData = allRows.length > 0;

    const columns: ChartTableColumn<Row>[] = [
        { header: groupBy === "employee" ? t("dashboard.hr.group_by_employee") : t("dashboard.filters.department.label"), key: "name" },
        { header: t("dashboard.hr.unexcused"), key: "unexcused_count", numeric: true },
        { header: t("dashboard.hr.consecutive_days"), key: "consecutive_count", numeric: true },
        { header: t("dashboard.hr.total_absent"), key: "total_absent_count", numeric: true },
        { header: t("dashboard.hr.excused_leave"), key: "total_leave_count", numeric: true },
    ];

    const groupLabel = groupBy === "employee" ? t("dashboard.hr.group_by_employee") : t("dashboard.hr.group_by_department");
    const summary = t("dashboard.hr.unexcused_absences_summary", { group: groupLabel });

    const groupByOptions: { key: GroupBy; label: string; icon: typeof Building2 }[] = [
        { key: "department", label: t("dashboard.hr.group_by_department"), icon: Building2 },
        { key: "employee", label: t("dashboard.hr.group_by_employee"), icon: Users },
    ];

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <ChartHeader
                groupBy={groupBy}
                groupByOptions={groupByOptions}
                range={range}
                setGroupBy={setGroupBy}
                setRange={setRange}
                timezone={timezone}
            />

            {isLoading ? (
                <ChartLoading />
            ) : error ? (
                <DashboardChartError icon={<ShieldAlert size={48} />} />
            ) : !hasData ? (
                <ChartEmpty />
            ) : (
                <>
                    <ChartSummary text={summary} />
                    <div role="img" aria-label={summary}>
                        <ResponsiveContainer height={CHART_HEIGHT} width="100%">
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
                                <Legend
                                    wrapperStyle={{ fontSize: "9px", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.15em" }}
                                />
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
                                    label={false}
                                    name={t("dashboard.hr.unexcused")}
                                    radius={[0, 4, 4, 0]}
                                    stackId="absences"
                                />
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                    <ChartTableTwin
                        caption={t("dashboard.hr.unexcused_absences")}
                        columns={columns}
                        rows={allRows}
                        searchableKeys={["name"]}
                    />
                </>
            )}
        </div>
    );
}

interface CustomTooltipProps {
    active?: boolean;
    label?: string;
    payload?: { payload: Row }[];
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
                <TooltipRow color="text-red-500" label={t("dashboard.hr.unexcused")} value={data.unexcused_count} />
                <TooltipRow color="text-orange-500" label={t("dashboard.hr.consecutive_days")} value={data.consecutive_count} />
                <div className="border-t border-slate-100 dark:border-slate-800 pt-1.5 mt-1.5">
                    <TooltipRow color="text-slate-500" label={t("dashboard.hr.total_absent")} value={data.total_absent_count} />
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

interface ChartHeaderProps {
    groupBy: GroupBy;
    groupByOptions: { icon: typeof Building2; key: GroupBy; label: string }[];
    range: { startDate: string; endDate: string };
    setGroupBy: (g: GroupBy) => void;
    setRange: (r: { startDate: string; endDate: string }) => void;
    timezone?: string;
}

function ChartHeader({
    groupByOptions,
    groupBy,
    setGroupBy,
    range,
    setRange,
    timezone,
}: ChartHeaderProps) {
    const { t } = useTranslation();
    return (
        <div className="flex min-w-0 flex-wrap items-center justify-between gap-4 mb-8">
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

            <div className="flex min-w-0 max-w-full flex-wrap items-center justify-end gap-3">
                <div className="flex shrink-0 max-w-full overflow-x-auto scrollbar-hide bg-slate-100 dark:bg-slate-700/50 p-1 rounded-[14px] border border-slate-200 dark:border-slate-700/50 shadow-inner">
                    {groupByOptions.map((g) => {
                        const Icon = g.icon;
                        const button = (
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
                        // Employee mode surfaces per-employee PII — gate the toggle.
                        return g.key === "employee" ? (
                            <PermissionGate
                                fallback={null}
                                key={g.key}
                                permission={EMPLOYEE_ABSENCE_READ}
                            >
                                {button}
                            </PermissionGate>
                        ) : (
                            button
                        );
                    })}
                </div>

                <DashboardDateRange
                    endDate={range.endDate}
                    onChange={setRange}
                    startDate={range.startDate}
                    timezone={timezone}
                />
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
