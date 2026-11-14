import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
    Bar,
    BarChart,
    CartesianGrid,
    Cell,
    Legend,
    Line,
    LineChart,
    ReferenceLine,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";

import { ChartTableTwin, type ChartTableColumn } from "@/components/features/dashboard/chart-table-twin";
import { DashboardChartError } from "@/components/features/dashboard/dashboard-chart-state";
import { DashboardDateRange } from "@/components/features/dashboard/dashboard-date-range";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import {
    useTurnoverAnalytics,
    type TurnoverAnalytics,
    type TurnoverRate,
} from "@/hooks/use-dashboard";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { colorByEntity } from "@/lib/stable-colors";
import { cn } from "@/lib/utils";

type TabKey = "overview" | "tenure" | "source" | "department" | "reasons" | "breakdown";

type ComparisonKey = "none" | "previous_period" | "same_period_last_year";

/** Translation function accepting optional i18next interpolation options. */
type TranslateFn = (key: string, options?: Record<string, string | number>) => string;

interface PeriodOption {
    endDate: string;
    label: string;
    startDate: string;
    value: string;
}

function toISODate(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function getPeriodOptions(t: TranslateFn): PeriodOption[] {
    const today = new Date();
    const endDate = toISODate(today);

    const months3 = new Date(today.getFullYear(), today.getMonth() - 3, 1);
    const months6 = new Date(today.getFullYear(), today.getMonth() - 6, 1);

    const year1 = new Date(today);
    year1.setFullYear(year1.getFullYear() - 1);

    const year2 = new Date(today);
    year2.setFullYear(year2.getFullYear() - 2);

    return [
        { label: t("dashboard.hr.turnover_period_3m"), value: "3m", startDate: toISODate(months3), endDate },
        { label: t("dashboard.hr.turnover_period_6m"), value: "6m", startDate: toISODate(months6), endDate },
        { label: t("dashboard.hr.turnover_period_1y"), value: "1y", startDate: toISODate(year1), endDate },
        { label: t("dashboard.hr.turnover_period_2y"), value: "2y", startDate: toISODate(year2), endDate },
        { label: t("dashboard.hr.custom_range"), value: "custom", startDate: "", endDate: "" },
    ];
}

const TOOLTIP_STYLE = {
    borderRadius: "12px",
    border: "none",
    boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
    fontSize: 12,
};

const TREND_RATE_COLOR = "#6366f1";
const TREND_LEAVERS_COLOR = "#ef4444";
const TENURE_COLOR = "#6366f1";
const DEPT_COLOR = "#3b82f6";
const CLASSIFICATION_COLORS: Record<string, string> = {
    voluntary: "#22c55e",
    involuntary: "#ef4444",
    unknown: "#94a3b8",
};

function rateDisplay(rate: TurnoverRate | undefined, t: TranslateFn): string {
    if (!rate) return t("dashboard.hr.rate_no_value");
    if (rate.value == null) return t("dashboard.hr.rate_no_value");
    return t("dashboard.hr.rate_with_denominator", {
        rate: `${rate.value}%`,
        numerator: String(rate.numerator),
        denominator: String(rate.denominator),
    });
}

function KpiCard({
    label,
    value,
    sub,
    accentClass,
}: {
    label: string;
    value: string;
    sub?: string;
    accentClass?: string;
}) {
    return (
        <div className="flex-1 min-w-[140px] rounded-2xl border border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-800/30 px-4 py-3">
            <Typography className="text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500" variant="p">
                {label}
            </Typography>
            <div className="mt-1 flex items-baseline gap-2 flex-wrap">
                <span className={cn("text-xl font-bold text-slate-900 dark:text-slate-100", accentClass)}>
                    {value}
                </span>
                {sub ? (
                    <span className="text-[10px] font-semibold text-slate-400 dark:text-slate-500">{sub}</span>
                ) : null}
            </div>
        </div>
    );
}

function ComparisonBadge({
    comparison,
    target,
    t,
}: {
    comparison: TurnoverAnalytics["comparison"];
    target: TurnoverAnalytics["target"];
    t: TranslateFn;
}) {
    const bits: React.ReactNode[] = [];

    if (comparison?.delta_percentage_points != null) {
        const delta = parseFloat(comparison.delta_percentage_points);
        const arrow = delta > 0 ? "▲" : delta < 0 ? "▼" : "●";
        const color = delta > 0 ? "text-rose-500 dark:text-rose-400" : "text-emerald-500 dark:text-emerald-400";
        bits.push(
            <span className={cn("text-[10px] font-bold", color)} key="delta">
                {arrow} {t("dashboard.hr.pp_delta", { delta: comparison.delta_percentage_points })}
            </span>,
        );
    }

    if (target) {
        const value = target.value != null ? `${target.value}%` : "—";
        const operator = target.operator === "lte" ? "≤" : target.operator === "gte" ? "≥" : "≈";
        bits.push(
            <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500" key="target">
                {t("dashboard.hr.target")} {operator} {value}
            </span>,
        );
    }

    if (bits.length === 0) return null;

    return (
        <div className="flex items-center gap-3 flex-wrap">
            {bits}
        </div>
    );
}

function WarningStrip({ warnings, t }: { warnings: TurnoverAnalytics["warnings"]; t: TranslateFn }) {
    if (!warnings || warnings.length === 0) return null;

    return (
        <div aria-live="polite" className="mb-5 space-y-2">
            {warnings.map((w) => {
                const labelKey =
                    w.code === "LOW_DENOMINATOR"
                        ? "dashboard.hr.turnover_warning_low_denominator"
                        : w.code === "NO_DATA_SOURCE"
                            ? "dashboard.hr.turnover_warning_no_data_source"
                            : "dashboard.hr.turnover_warning_incomplete_data";
                const label = w.message?.startsWith("dashboard.") ? t(w.message) : (w.message ?? t(labelKey));
                return (
                    <div
                        className="flex items-start gap-2 rounded-xl border border-amber-200 dark:border-amber-800 bg-amber-50 dark:bg-amber-950/40 px-4 py-2.5"
                        key={w.code}
                        role="note"
                    >
                        <span aria-hidden="true" className="text-amber-500">⚠</span>
                        <span className="text-[11px] font-semibold text-amber-700 dark:text-amber-400">
                            {label}
                        </span>
                    </div>
                );
            })}
        </div>
    );
}

function KpiHeader({ data, t }: { data: TurnoverAnalytics; t: TranslateFn }) {
    const retention = data.retention_rate != null ? `${data.retention_rate}%` : t("dashboard.hr.rate_no_value");
    return (
        <div className="mb-6">
            <div className="flex flex-wrap gap-3">
                <KpiCard
                    accentClass="text-primary-600 dark:text-primary-400"
                    label={t("dashboard.hr.turnover_rate")}
                    sub={t("dashboard.hr.turnover_rate_subtitle")}
                    value={rateDisplay(data.turnover_rate, t)}
                />
                <KpiCard
                    label={t("dashboard.hr.avg_headcount")}
                    value={String(data.average_headcount)}
                />
                <KpiCard
                    accentClass="text-emerald-500"
                    label={t("dashboard.hr.voluntary")}
                    value={String(data.voluntary_count)}
                />
                <KpiCard
                    accentClass="text-rose-500"
                    label={t("dashboard.hr.involuntary")}
                    value={String(data.involuntary_count)}
                />
                <KpiCard
                    label={t("dashboard.hr.retention_rate")}
                    value={retention}
                />
                <KpiCard
                    label={t("dashboard.hr.early_attrition")}
                    value={rateDisplay(data.early_attrition, t)}
                />
            </div>
            <div className="mt-3">
                <ComparisonBadge comparison={data.comparison} t={t} target={data.target} />
            </div>
        </div>
    );
}

function ChartHeader({
    t,
    periods,
    selectedPeriod,
    setSelectedPeriod,
    customRange,
    setCustomRange,
    comparison,
    setComparison,
}: {
    t: TranslateFn;
    periods: PeriodOption[];
    selectedPeriod: string;
    setSelectedPeriod: (v: string) => void;
    customRange: { startDate: string; endDate: string };
    setCustomRange: (r: { startDate: string; endDate: string }) => void;
    comparison: ComparisonKey;
    setComparison: (c: ComparisonKey) => void;
}) {
    const comparisonOptions: { label: string; value: ComparisonKey }[] = [
        { label: t("dashboard.hr.previous_period"), value: "previous_period" },
        { label: t("dashboard.hr.same_period_last_year"), value: "same_period_last_year" },
        { label: t("dashboard.hr.no_comparison"), value: "none" },
    ];

    return (
        <div className="flex flex-col gap-4 mb-6">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                <div>
                    <Typography className="text-lg font-semibold text-slate-800 dark:text-slate-200" variant="h3">
                        {t("dashboard.hr.turnover_title")}
                    </Typography>
                    <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1" variant="p">
                        {t("dashboard.hr.turnover_subtitle")}
                    </Typography>
                </div>

                <div className="flex min-w-0 max-w-full flex-wrap items-center gap-3 sm:justify-end">
                    <select
                        aria-label={t("dashboard.hr.comparison")}
                        className="min-w-0 max-w-full text-xs font-bold text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-800 px-4 py-2 rounded-xl border border-slate-200 dark:border-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 uppercase tracking-widest cursor-pointer"
                        onChange={(e) => { setComparison(e.target.value as ComparisonKey); }}
                        value={comparison}
                    >
                        {comparisonOptions.map((o) => (
                            <option key={o.value} value={o.value}>{o.label}</option>
                        ))}
                    </select>

                    <select
                        aria-label={t("dashboard.hr.turnover_period")}
                        className="min-w-0 max-w-full text-xs font-bold text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-800 px-4 py-2 rounded-xl border border-slate-200 dark:border-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 uppercase tracking-widest cursor-pointer"
                        onChange={(e) => { setSelectedPeriod(e.target.value); }}
                        value={selectedPeriod}
                    >
                        {periods.map((p) => (
                            <option key={p.value} value={p.value}>
                                {p.label}
                            </option>
                        ))}
                    </select>
                </div>
            </div>

            {selectedPeriod === "custom" ? (
                <DashboardDateRange
                    endDate={customRange.endDate}
                    onChange={setCustomRange}
                    startDate={customRange.startDate}
                />
            ) : null}
        </div>
    );
}

function ChartTabs({
    tabs,
    activeTab,
    setActiveTab,
    t,
}: {
    activeTab: TabKey;
    setActiveTab: (k: TabKey) => void;
    t: TranslateFn;
    tabs: { key: TabKey; labelKey: string }[];
}) {
    return (
        <div className="flex max-w-full overflow-x-auto scrollbar-hide bg-slate-100 dark:bg-slate-800/50 p-1 rounded-2xl w-fit mb-6 border border-slate-200/50 dark:border-slate-700/50">
            {tabs.map((tab) => (
                <Button
                    className={cn(
                        "px-5 py-2 rounded-xl text-xs font-bold uppercase tracking-widest transition-all h-auto shadow-none border border-transparent",
                        activeTab === tab.key
                            ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900"
                            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent",
                    )}
                    key={tab.key}
                    onClick={() => { setActiveTab(tab.key); }}
                    type="button"
                    variant="ghost"
                >
                    {t(tab.labelKey)}
                </Button>
            ))}
        </div>
    );
}

export default function TurnoverAnalyticsChart() {
    const { t } = useTranslation();
    const { formatDate } = useDateTimeFormatter();
    const [activeTab, setActiveTab] = useState<TabKey>("overview");
    const [selectedPeriod, setSelectedPeriod] = useState("6m");
    const [customRange, setCustomRange] = useState<{ startDate: string; endDate: string }>({ startDate: "", endDate: "" });
    const [comparison, setComparison] = useState<ComparisonKey>("previous_period");

    const periods = useMemo(() => getPeriodOptions(t), [t]);
    const currentPeriod = periods.find((p) => p.value === selectedPeriod) ?? periods[1];

    const resolvedRange = selectedPeriod === "custom" ? customRange : currentPeriod;
    const effectiveComparison = comparison === "none" ? undefined : comparison;

    const { data, error, isLoading } = useTurnoverAnalytics({
        startDate: resolvedRange.startDate,
        endDate: resolvedRange.endDate,
        comparison: effectiveComparison,
    });

    const tabs: { key: TabKey; labelKey: string }[] = useMemo(() => [
        { key: "overview", labelKey: "dashboard.hr.turnover_overview_tab" },
        { key: "tenure", labelKey: "dashboard.hr.turnover_tenure" },
        { key: "source", labelKey: "dashboard.hr.turnover_by_source" },
        { key: "department", labelKey: "dashboard.hr.turnover_by_dept" },
        { key: "reasons", labelKey: "dashboard.hr.turnover_reasons_tab" },
        { key: "breakdown", labelKey: "dashboard.hr.turnover_breakdown_tab" },
    ], []);

    if (isLoading) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm animate-pulse">
                <div className="h-4 w-48 bg-slate-200 dark:bg-slate-700 rounded-full mb-8" />
                <div className="h-64 bg-slate-100 dark:bg-slate-800 rounded-3xl" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
                <ChartHeader
                    comparison={comparison}
                    customRange={customRange}
                    periods={periods}
                    selectedPeriod={selectedPeriod}
                    setComparison={setComparison}
                    setCustomRange={setCustomRange}
                    setSelectedPeriod={setSelectedPeriod}
                    t={t}
                />
                <DashboardChartError icon={<span aria-hidden="true" className="text-4xl">!</span>} />
            </div>
        );
    }

    const isEmpty =
        !data ||
        (data.tenure_distribution.length === 0 &&
            data.by_source.length === 0 &&
            data.by_department.length === 0 &&
            data.termination_reasons.length === 0 &&
            data.monthly_trend.length === 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <ChartHeader
                comparison={comparison}
                customRange={customRange}
                periods={periods}
                selectedPeriod={selectedPeriod}
                setComparison={setComparison}
                setCustomRange={setCustomRange}
                setSelectedPeriod={setSelectedPeriod}
                t={t}
            />

            {data ? <WarningStrip t={t} warnings={data.warnings} /> : null}

            <ChartTabs
                activeTab={activeTab}
                setActiveTab={setActiveTab}
                t={t}
                tabs={tabs}
            />

            {isEmpty ? (
                <div className="flex items-center justify-center h-64 text-slate-400">
                    {t("common.no_data")}
                </div>
            ) : (
                <div className="space-y-6">
                    {data ? <KpiHeader data={data} t={t} /> : null}

                    {activeTab === "overview" && <OverviewTab data={data} formatDate={formatDate} t={t} />}
                    {activeTab === "tenure" && <TenureTab data={data} t={t} />}
                    {activeTab === "source" && <SourceTab data={data} t={t} />}
                    {activeTab === "department" && <DepartmentTab data={data} t={t} />}
                    {activeTab === "reasons" && <ReasonsTab data={data} t={t} />}
                    {activeTab === "breakdown" && <BreakdownTab data={data} t={t} />}
                </div>
            )}
        </div>
    );
}

// ── Tabs ──────────────────────────────────────────────────────────────

function OverviewTab({ data, t, formatDate }: { data: TurnoverAnalytics; t: TranslateFn; formatDate: (d: string | Date | null | undefined) => string }) {
    return (
        <>
            {data.monthly_trend.length > 0 ? <TrendChart data={data.monthly_trend} formatDate={formatDate} t={t} /> : null}
            {data.cohort_retention.length > 0 ? <CohortChart data={data.cohort_retention} t={t} /> : null}
        </>
    );
}

function TrendChart({ data, t, formatDate }: { data: TurnoverAnalytics["monthly_trend"]; t: TranslateFn; formatDate: (d: string | Date | null | undefined) => string }) {
    const chartData = useMemo(
        () => data.map((point) => ({
            ...point,
            label: formatDate(point.month),
            turnover_rate_value: point.turnover_rate.value != null ? parseFloat(point.turnover_rate.value) : null,
        })),
        [data, formatDate],
    );

    const columns: ChartTableColumn<(typeof chartData)[number]>[] = [
        { header: t("common.date"), key: "label" },
        { header: t("dashboard.hr.headcount"), key: "headcount", numeric: true },
        { header: t("dashboard.hr.leavers"), key: "leavers", numeric: true },
        {
            header: t("dashboard.hr.turnover_rate"),
            key: "turnover_rate",
            numeric: true,
            render: (r) => r.turnover_rate.value ?? t("dashboard.hr.rate_no_value"),
        },
    ];

    return (
        <div>
            <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                {t("dashboard.hr.monthly_trend")}
            </Typography>
            <div aria-label={t("dashboard.hr.monthly_trend")} role="img">
                <ResponsiveContainer height={260} width="100%">
                    <LineChart data={chartData} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
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
                            yAxisId="headcount"
                        />
                        <YAxis
                            allowDecimals
                            axisLine={false}
                            className="text-slate-400 dark:text-slate-500"
                            orientation="right"
                            tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                            tickLine={false}
                            width={40}
                            yAxisId="rate"
                        />
                        <Tooltip contentStyle={TOOLTIP_STYLE} />
                        <Legend
                            wrapperStyle={{ fontSize: "9px", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.15em" }}
                        />
                        <Line
                            dataKey="headcount"
                            dot={{ r: 3 }}
                            name={t("dashboard.hr.headcount")}
                            stroke={TREND_RATE_COLOR}
                            strokeWidth={2.5}
                            type="monotone"
                            yAxisId="headcount"
                        />
                        <Line
                            dataKey="leavers"
                            dot={{ r: 3 }}
                            name={t("dashboard.hr.leavers")}
                            stroke={TREND_LEAVERS_COLOR}
                            strokeWidth={2.5}
                            type="monotone"
                            yAxisId="headcount"
                        />
                        <Line
                            dataKey="turnover_rate_value"
                            dot={false}
                            name={t("dashboard.hr.turnover_rate")}
                            stroke="#8b5cf6"
                            strokeDasharray="5 4"
                            strokeWidth={2}
                            type="monotone"
                            yAxisId="rate"
                        />
                        <ReferenceLine
                            stroke="#f59e0b"
                            strokeDasharray="4 4"
                            strokeWidth={1}
                            y={data[0]?.turnover_rate.value != null ? parseFloat(data[0].turnover_rate.value) : undefined}
                            yAxisId="rate"
                        />
                    </LineChart>
                </ResponsiveContainer>
            </div>
            <ChartTableTwin
                caption={t("dashboard.hr.monthly_trend")}
                columns={columns}
                rows={chartData}
                searchableKeys={["label"]}
            />
        </div>
    );
}

function CohortChart({ data, t }: { data: TurnoverAnalytics["cohort_retention"]; t: TranslateFn }) {
    const chartData = useMemo(
        () => data.map((row) => ({
            ...row,
            retention_value: row.retention_rate != null ? parseFloat(row.retention_rate) : null,
        })),
        [data],
    );

    const columns: ChartTableColumn<(typeof chartData)[number]>[] = [
        { header: t("dashboard.hr.cohort_month"), key: "cohort_month" },
        { header: t("dashboard.hr.months_since_hire"), key: "months_since_hire", numeric: true },
        { header: t("dashboard.hr.cohort_size"), key: "cohort_size", numeric: true },
        { header: t("dashboard.hr.active_count"), key: "active_count", numeric: true },
        {
            header: t("dashboard.hr.retention_rate"),
            key: "retention_rate",
            numeric: true,
            render: (r) => r.retention_rate ?? t("dashboard.hr.rate_no_value"),
        },
    ];

    return (
        <div>
            <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                {t("dashboard.hr.cohort_retention")}
            </Typography>
            <div className="overflow-x-auto">
                <table className="w-full text-left text-[11px]">
                    <caption className="sr-only">{t("dashboard.hr.cohort_retention")}</caption>
                    <thead className="sticky top-0 bg-slate-50 dark:bg-slate-800/80 text-slate-500 dark:text-slate-400 uppercase text-[9px] tracking-widest">
                        <tr>
                            {columns.map((col) => (
                                <th className="px-4 py-3 font-bold whitespace-nowrap" key={String(col.key)} scope="col">
                                    {t(col.header)}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                        {chartData.length === 0 ? (
                            <tr>
                                <td className="px-4 py-8 text-center text-slate-400" colSpan={columns.length}>
                                    {t("common.no_data")}
                                </td>
                            </tr>
                        ) : (
                            chartData.map((row, index) => (
                                <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/40" key={`${row.cohort_month}-${row.months_since_hire}`}>
                                    {columns.map((col) => (
                                        <td className="px-4 py-2.5 whitespace-nowrap" key={String(col.key)}>
                                            {col.render ? col.render(row) : String(row[col.key] ?? "")}
                                        </td>
                                    ))}
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>
            <ChartTableTwin
                caption={t("dashboard.hr.cohort_retention")}
                columns={columns}
                rows={chartData}
                searchableKeys={["cohort_month"]}
            />
        </div>
    );
}

function HorizontalBarChart<T extends { count: number }>({
    rows,
    nameKey,
    colorKey,
    chartKey,
    emptyMessage,
    t,
}: {
    rows: T[];
    nameKey: (row: T) => string;
    colorKey: (row: T) => string;
    chartKey: string;
    emptyMessage: string;
    t: TranslateFn;
}) {
    const chartData = useMemo(
        () => rows.filter((r) => r.count > 0).map((r) => ({
            name: nameKey(r),
            value: r.count,
            fill: colorByEntity(colorKey(r)),
        })),
        [rows, nameKey, colorKey],
    );

    if (chartData.length === 0) {
        return (
            <div className="flex items-center justify-center h-48 text-slate-400">
                {emptyMessage}
            </div>
        );
    }

    const columns: ChartTableColumn<(typeof chartData)[number]>[] = [
        { header: "common.name", key: "name" },
        { header: "dashboard.hr.people", key: "value", numeric: true },
    ];

    return (
        <div>
            <div aria-label={chartKey} role="img">
                <ResponsiveContainer height={Math.max(200, chartData.length * 44)} width="100%">
                    <BarChart data={chartData} layout="vertical" margin={{ top: 10, right: 20, left: 10, bottom: 10 }}>
                        <CartesianGrid horizontal={false} stroke="#e2e8f0" strokeDasharray="3 3" />
                        <XAxis
                            allowDecimals={false}
                            axisLine={false}
                            tick={{ fontSize: 11, fill: "#94a3b8" }}
                            tickLine={false}
                            type="number"
                        />
                        <YAxis
                            axisLine={false}
                            dataKey="name"
                            tick={{ fontSize: 11, fill: "#94a3b8" }}
                            tickLine={false}
                            type="category"
                            width={120}
                        />
                        <Tooltip contentStyle={TOOLTIP_STYLE} />
                        <Bar dataKey="value" radius={[0, 8, 8, 0]}>
                            {chartData.map((entry) => (
                                // eslint-disable-next-line @typescript-eslint/no-deprecated
                                <Cell fill={entry.fill} key={entry.name} />
                            ))}
                        </Bar>
                    </BarChart>
                </ResponsiveContainer>
            </div>
            <ChartTableTwin
                caption={chartKey}
                columns={columns}
                rows={chartData}
                searchableKeys={["name"]}
            />
        </div>
    );
}

function TenureTab({ data, t }: { data: TurnoverAnalytics; t: TranslateFn }) {
    return (
        <HorizontalBarChart
            chartKey={t("dashboard.hr.turnover_tenure")}
            colorKey={(r) => r.bucket}
            emptyMessage={t("common.no_data")}
            nameKey={(r) => r.bucket}
            rows={data.tenure_distribution}
            t={t}
        />
    );
}

function SourceTab({ data, t }: { data: TurnoverAnalytics; t: TranslateFn }) {
    const known: Record<string, string> = {
        direct: "#22c55e",
        referral: "#3b82f6",
        agency: "#f59e0b",
    };
    return (
        <HorizontalBarChart
            chartKey={t("dashboard.hr.turnover_by_source")}
            colorKey={(r) => known[r.source_type] ?? r.source_type}
            emptyMessage={t("common.no_data")}
            nameKey={(r) => r.source_name}
            rows={data.by_source}
            t={t}
        />
    );
}

function DepartmentTab({ data, t }: { data: TurnoverAnalytics; t: TranslateFn }) {
    return (
        <HorizontalBarChart
            chartKey={t("dashboard.hr.turnover_by_dept")}
            colorKey={(r) => r.department_name}
            emptyMessage={t("common.no_data")}
            nameKey={(r) => r.department_name}
            rows={data.by_department}
            t={t}
        />
    );
}

function ReasonsTab({ data, t }: { data: TurnoverAnalytics; t: TranslateFn }) {
    const classificationRows = data.classification.length > 0
        ? data.classification.map((c) => ({
            name: t(`dashboard.hr.${c.classification}`),
            value: c.count,
            fill: CLASSIFICATION_COLORS[c.classification] ?? "#94a3b8",
        }))
        : [];

    const columns: ChartTableColumn<(typeof classificationRows)[number]>[] = [
        { header: t("dashboard.hr.classification"), key: "name" },
        { header: t("dashboard.hr.people"), key: "value", numeric: true },
    ];

    return (
        <div className="space-y-8">
            <div>
                <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                    {t("dashboard.hr.termination_reasons")}
                </Typography>
                <HorizontalBarChart
                    chartKey={t("dashboard.hr.termination_reasons")}
                    colorKey={(r) => r.reason}
                    emptyMessage={t("common.no_data")}
                    nameKey={(r) => r.reason}
                    rows={data.termination_reasons}
                    t={t}
                />
            </div>

            {classificationRows.length > 0 ? (
                <div>
                    <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                        {t("dashboard.hr.classification")}
                    </Typography>
                    <div aria-label={t("dashboard.hr.classification")} role="img">
                        <ResponsiveContainer height={160} width="100%">
                            <BarChart data={classificationRows} layout="vertical" margin={{ top: 10, right: 20, left: 10, bottom: 10 }}>
                                <CartesianGrid horizontal={false} stroke="#e2e8f0" strokeDasharray="3 3" />
                                <XAxis allowDecimals={false} axisLine={false} tick={{ fontSize: 11, fill: "#94a3b8" }} tickLine={false} type="number" />
                                <YAxis axisLine={false} dataKey="name" tick={{ fontSize: 11, fill: "#94a3b8" }} tickLine={false} type="category" width={110} />
                                <Tooltip contentStyle={TOOLTIP_STYLE} />
                                <Bar dataKey="value" radius={[0, 8, 8, 0]}>
                                    {classificationRows.map((entry) => (
                                        // eslint-disable-next-line @typescript-eslint/no-deprecated
                                        <Cell fill={entry.fill} key={entry.name} />
                                    ))}
                                </Bar>
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                    <ChartTableTwin
                        caption={t("dashboard.hr.classification")}
                        columns={columns}
                        rows={classificationRows}
                        searchableKeys={["name"]}
                    />
                </div>
            ) : null}
        </div>
    );
}

function BreakdownTab({ data, t }: { data: TurnoverAnalytics; t: TranslateFn }) {
    return (
        <div className="grid gap-8 lg:grid-cols-2">
            <div>
                <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                    {t("dashboard.hr.by_employment_type")}
                </Typography>
                <HorizontalBarChart
                    chartKey={t("dashboard.hr.by_employment_type")}
                    colorKey={(r) => r.employment_type}
                    emptyMessage={t("common.no_data")}
                    nameKey={(r) => r.employment_type}
                    rows={data.by_employment_type}
                    t={t}
                />
            </div>
            <div>
                <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                    {t("dashboard.hr.by_location")}
                </Typography>
                <HorizontalBarChart
                    chartKey={t("dashboard.hr.by_location")}
                    colorKey={(r) => r.location_name}
                    emptyMessage={t("common.no_data")}
                    nameKey={(r) => r.location_name}
                    rows={data.by_location}
                    t={t}
                />
            </div>
            <div className="lg:col-span-2">
                <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                    {t("dashboard.hr.by_manager")}
                </Typography>
                <HorizontalBarChart
                    chartKey={t("dashboard.hr.by_manager")}
                    colorKey={(r) => r.manager_name}
                    emptyMessage={t("common.no_data")}
                    nameKey={(r) => r.manager_name}
                    rows={data.by_manager}
                    t={t}
                />
            </div>
        </div>
    );
}
