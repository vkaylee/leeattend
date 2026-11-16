import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
    Bar,
    BarChart,
    CartesianGrid,
    Legend,
    Line,
    LineChart,
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
    useRecruitmentEffectiveness,
    type RecruitmentEffectivenessAnalytics,
} from "@/hooks/use-dashboard";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDepartments } from "@/hooks/use-departments";
import { colorByEntityWithKnown } from "@/lib/stable-colors";
import { cn } from "@/lib/utils";

type TabKey = "overview" | "funnel" | "ranking" | "table";

type ComparisonKey = "none" | "previous_period" | "same_period_last_year";

/** Translation function accepting optional i18next interpolation options. */
type TranslateFn = (key: string, options?: Record<string, string | number>) => string;

interface PeriodOption {
    endDate: string;
    label: string;
    startDate: string;
    value: string;
}

const TOOLTIP_STYLE = {
    borderRadius: "12px",
    border: "none",
    boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
    fontSize: 12,
};

/** Stable per-source-type colors — a source keeps the same hue across re-renders. */
const SOURCE_TYPE_COLORS: Record<string, string> = {
    direct: "#22c55e",   // green
    referral: "#3b82f6", // blue
    agency: "#f59e0b",   // amber
    unknown: "#94a3b8",  // slate
};

const HIRES_TREND_COLOR = "#6366f1";

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

function WarningStrip({ warnings, t }: { warnings: RecruitmentEffectivenessAnalytics["warnings"]; t: TranslateFn }) {
    if (warnings.length === 0) return null;

    return (
        <div aria-live="polite" className="mb-5 space-y-2">
            {warnings.map((w) => {
                const label = w.message.startsWith("dashboard.") ? t(w.message) : w.message;
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

function ComparisonBadge({ data, t }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn }) {
    const { overview } = data;
    if (overview.comparison_hires == null) return null;

    const delta = overview.comparison_hires_delta ?? 0;
    const arrow = delta > 0 ? "▲" : delta < 0 ? "▼" : "●";
    const color = delta > 0 ? "text-emerald-500 dark:text-emerald-400" : "text-rose-500 dark:text-rose-400";

    return (
        <div className="flex items-center gap-3 flex-wrap">
            <span className={cn("text-[10px] font-bold", color)}>
                {arrow} {t("dashboard.hr.effective_comparison_delta", { delta: String(delta) })}
            </span>
            <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500">
                {t("dashboard.hr.effective_previous_hires", { count: String(overview.comparison_hires) })}
            </span>
        </div>
    );
}

function KpiHeader({ data, t }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn }) {
    const { overview } = data;
    const retention = overview.retention_90d != null ? `${overview.retention_90d}%` : t("dashboard.hr.rate_no_value");
    const avgCost = overview.average_cost_per_hire ?? t("dashboard.hr.rate_no_value");
    return (
        <div className="mb-6">
            <div className="flex flex-wrap gap-3">
                <KpiCard
                    accentClass="text-primary-600 dark:text-primary-400"
                    label={t("dashboard.hr.effective_total_hires")}
                    value={String(overview.hires)}
                />
                <KpiCard
                    label={t("dashboard.hr.effective_avg_cost_per_hire")}
                    sub={overview.cost_available ? undefined : t("dashboard.hr.effective_no_cost_data")}
                    value={avgCost}
                />
                <KpiCard
                    accentClass="text-emerald-500"
                    label={t("dashboard.hr.effective_retained_90d")}
                    value={String(overview.retained_90d)}
                />
                <KpiCard
                    label={t("dashboard.hr.effective_retention_90d")}
                    value={retention}
                />
                <KpiCard
                    accentClass="text-rose-500"
                    label={t("dashboard.hr.effective_turnover_in_range")}
                    value={String(overview.turnover_in_range)}
                />
            </div>
            <div className="mt-3">
                <ComparisonBadge data={data} t={t} />
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
    departmentIds,
    setDepartmentIds,
    sourceTypes,
    setSourceTypes,
    sourceNameFilter,
    setSourceNameFilter,
    departments,
}: {
    t: TranslateFn;
    periods: PeriodOption[];
    selectedPeriod: string;
    setSelectedPeriod: (v: string) => void;
    customRange: { startDate: string; endDate: string };
    setCustomRange: (r: { startDate: string; endDate: string }) => void;
    comparison: ComparisonKey;
    setComparison: (c: ComparisonKey) => void;
    departmentIds: string[];
    setDepartmentIds: (v: string[]) => void;
    sourceTypes: string[];
    setSourceTypes: (v: string[]) => void;
    sourceNameFilter: string;
    setSourceNameFilter: (v: string) => void;
    departments: { id: string; name: string }[];
}) {
    const comparisonOptions: { label: string; value: ComparisonKey }[] = [
        { label: t("dashboard.hr.previous_period"), value: "previous_period" },
        { label: t("dashboard.hr.same_period_last_year"), value: "same_period_last_year" },
        { label: t("dashboard.hr.no_comparison"), value: "none" },
    ];

    const sourceTypeOptions = [
        { label: t("dashboard.hr.effective_source_direct"), value: "direct" },
        { label: t("dashboard.hr.effective_source_referral"), value: "referral" },
        { label: t("dashboard.hr.effective_source_agency"), value: "agency" },
    ];

    return (
        <div className="flex flex-col gap-4 mb-6">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                <div>
                    <Typography className="text-lg font-semibold text-slate-800 dark:text-slate-200" variant="h3">
                        {t("dashboard.hr.effective_title")}
                    </Typography>
                    <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1" variant="p">
                        {t("dashboard.hr.effective_subtitle")}
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

            {/* Self-contained filter row — matches the turnover widget's decision to keep
                the executive global filter bar separate. */}
            <div className="flex flex-wrap items-center gap-3">
                <select
                    aria-label={t("dashboard.hr.effective_department_filter")}
                    className="min-w-[140px] max-w-full text-xs font-bold text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-800 px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 uppercase tracking-widest cursor-pointer"
                    onChange={(e) => {
                        const value = e.target.value;
                        if (value === "") {
                            setDepartmentIds([]);
                        } else {
                            setDepartmentIds([value]);
                        }
                    }}
                    value={departmentIds.length === 1 ? departmentIds[0] : ""}
                >
                    <option value="">{t("dashboard.hr.effective_all_departments")}</option>
                    {departments.map((d) => (
                        <option key={d.id} value={d.id}>{d.name}</option>
                    ))}
                </select>

                <select
                    aria-label={t("dashboard.hr.effective_source_type_filter")}
                    className="min-w-[140px] max-w-full text-xs font-bold text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-800 px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 uppercase tracking-widest cursor-pointer"
                    onChange={(e) => {
                        const value = e.target.value;
                        if (value === "") {
                            setSourceTypes([]);
                        } else {
                            setSourceTypes([value]);
                        }
                    }}
                    value={sourceTypes.length === 1 ? sourceTypes[0] : ""}
                >
                    <option value="">{t("dashboard.hr.effective_all_source_types")}</option>
                    {sourceTypeOptions.map((o) => (
                        <option key={o.value} value={o.value}>{o.label}</option>
                    ))}
                </select>

                <input
                    aria-label={t("dashboard.hr.effective_source_name_filter")}
                    className="min-w-[160px] max-w-full text-xs font-bold text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-800 px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 placeholder:text-slate-400"
                    onChange={(e) => { setSourceNameFilter(e.target.value); }}
                    placeholder={t("dashboard.hr.effective_source_name_placeholder")}
                    value={sourceNameFilter}
                />
            </div>
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

export default function RecruitmentSourceChart() {
    const { t } = useTranslation();
    const { formatDate } = useDateTimeFormatter();
    const { data: departments = [] } = useDepartments();

    const [activeTab, setActiveTab] = useState<TabKey>("overview");
    const [selectedPeriod, setSelectedPeriod] = useState("6m");
    const [customRange, setCustomRange] = useState<{ startDate: string; endDate: string }>({ startDate: "", endDate: "" });
    const [comparison, setComparison] = useState<ComparisonKey>("previous_period");
    const [departmentIds, setDepartmentIds] = useState<string[]>([]);
    const [sourceTypes, setSourceTypes] = useState<string[]>([]);
    const [sourceNameFilter, setSourceNameFilter] = useState("");

    const periods = useMemo(() => getPeriodOptions(t), [t]);
    const currentPeriod = periods.find((p) => p.value === selectedPeriod) ?? periods[1];

    const resolvedRange = selectedPeriod === "custom" ? customRange : currentPeriod;
    const effectiveComparison = comparison === "none" ? undefined : comparison;

    const { data, error, isLoading } = useRecruitmentEffectiveness({
        startDate: resolvedRange.startDate,
        endDate: resolvedRange.endDate,
        comparison: effectiveComparison,
        departmentIds: departmentIds.join(","),
        sourceTypes: sourceTypes.join(","),
    });

    const tabs: { key: TabKey; labelKey: string }[] = useMemo(() => [
        { key: "overview", labelKey: "dashboard.hr.effective_overview_tab" },
        { key: "funnel", labelKey: "dashboard.hr.effective_funnel_tab" },
        { key: "ranking", labelKey: "dashboard.hr.effective_ranking_tab" },
        { key: "table", labelKey: "dashboard.hr.effective_table_tab" },
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
                    departmentIds={departmentIds}
                    departments={departments}
                    periods={periods}
                    selectedPeriod={selectedPeriod}
                    setComparison={setComparison}
                    setCustomRange={setCustomRange}
                    setDepartmentIds={setDepartmentIds}
                    setSelectedPeriod={setSelectedPeriod}
                    setSourceNameFilter={setSourceNameFilter}
                    setSourceTypes={setSourceTypes}
                    sourceNameFilter={sourceNameFilter}
                    sourceTypes={sourceTypes}
                    t={t}
                />
                <DashboardChartError icon={<span aria-hidden="true" className="text-4xl">!</span>} />
            </div>
        );
    }

    const isEmpty = !data || data.total_hires === 0;

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <ChartHeader
                comparison={comparison}
                customRange={customRange}
                departmentIds={departmentIds}
                departments={departments}
                periods={periods}
                selectedPeriod={selectedPeriod}
                setComparison={setComparison}
                setCustomRange={setCustomRange}
                setDepartmentIds={setDepartmentIds}
                setSelectedPeriod={setSelectedPeriod}
                setSourceNameFilter={setSourceNameFilter}
                setSourceTypes={setSourceTypes}
                sourceNameFilter={sourceNameFilter}
                sourceTypes={sourceTypes}
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
                    <KpiHeader data={data} t={t} />

                    {activeTab === "overview" && <OverviewTab data={data} formatDate={formatDate} t={t} />}
                    {activeTab === "funnel" && <FunnelTab data={data} t={t} />}
                    {activeTab === "ranking" && <RankingTab data={filteredData(data, sourceNameFilter)} t={t} />}
                    {activeTab === "table" && <TableTab data={filteredData(data, sourceNameFilter)} t={t} />}
                </div>
            )}
        </div>
    );
}

// ── Tabs ──────────────────────────────────────────────────────────────

/** Client-side source-name filter (the backend has no free-text source name param). */
function filteredData(
    data: RecruitmentEffectivenessAnalytics,
    sourceNameFilter: string,
): RecruitmentEffectivenessAnalytics {
    const q = sourceNameFilter.trim().toLowerCase();
    if (!q) return data;
    return {
        ...data,
        by_source: data.by_source.filter((s) => s.source_name.toLowerCase().includes(q)),
    };
}

function OverviewTab({ data, t, formatDate }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn; formatDate: (d: string | Date | null | undefined) => string }) {
    return (
        <>
            {data.hires_trend.length > 0 ? <HiresTrendChart data={data.hires_trend} formatDate={formatDate} t={t} /> : null}
        </>
    );
}

function HiresTrendChart({ data, t, formatDate }: { data: RecruitmentEffectivenessAnalytics["hires_trend"]; t: TranslateFn; formatDate: (d: string | Date | null | undefined) => string }) {
    const chartData = useMemo(
        () => data.map((point) => ({
            ...point,
            label: formatDate(point.month),
        })),
        [data, formatDate],
    );

    const columns: ChartTableColumn<(typeof chartData)[number]>[] = [
        { header: t("dashboard.hr.effective_month"), key: "label" },
        { header: t("dashboard.hr.effective_hires"), key: "hires", numeric: true },
    ];

    return (
        <div>
            <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                {t("dashboard.hr.effective_hires_trend")}
            </Typography>
            <div aria-label={t("dashboard.hr.effective_hires_trend")} role="img">
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
                        />
                        <Tooltip contentStyle={TOOLTIP_STYLE} />
                        <Line
                            dataKey="hires"
                            dot={{ r: 3 }}
                            name={t("dashboard.hr.effective_hires")}
                            stroke={HIRES_TREND_COLOR}
                            strokeWidth={2.5}
                            type="monotone"
                        />
                    </LineChart>
                </ResponsiveContainer>
            </div>
            <ChartTableTwin
                caption={t("dashboard.hr.effective_hires_trend")}
                columns={columns}
                rows={chartData}
                searchableKeys={["label"]}
            />
        </div>
    );
}

function FunnelTab({ data, t }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn }) {
    const stages = data.funnel;
    const funnelRows = stages.map((stage) => ({
        ...stage,
        label: t(`dashboard.hr.effective_stage_${stage.stage}`),
        hasData: stage.count != null,
        countValue: stage.count ?? 0,
        conversionValue: stage.conversion_rate != null ? parseFloat(stage.conversion_rate) : null,
        fill: stage.count != null ? "#6366f1" : "#cbd5e1",
    }));

    const columns: ChartTableColumn<(typeof funnelRows)[number]>[] = [
        { header: t("dashboard.hr.effective_stage"), key: "label" },
        {
            header: t("dashboard.hr.effective_count"),
            key: "countValue",
            numeric: true,
            render: (r) => (r.hasData ? String(r.countValue) : t("dashboard.hr.effective_no_data_source")),
        },
        {
            header: t("dashboard.hr.effective_conversion"),
            key: "conversionValue",
            numeric: true,
            render: (r) => (r.conversionValue != null ? `${r.conversionValue}%` : t("dashboard.hr.rate_no_value")),
        },
    ];

    return (
        <div>
            <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                {t("dashboard.hr.effective_funnel_title")}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 mb-4" variant="p">
                {t("dashboard.hr.effective_funnel_honesty_note")}
            </Typography>

            <div aria-label={t("dashboard.hr.effective_funnel_title")} role="img">
                <ResponsiveContainer height={Math.max(220, funnelRows.length * 56)} width="100%">
                    <BarChart data={funnelRows} layout="vertical" margin={{ top: 10, right: 20, left: 10, bottom: 10 }}>
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
                            dataKey="label"
                            tick={{ fontSize: 11, fill: "#94a3b8" }}
                            tickLine={false}
                            type="category"
                            width={130}
                        />
                        <Tooltip
                            contentStyle={TOOLTIP_STYLE}
                            formatter={(value: unknown, name: unknown) => {
                                if (name === "countValue") {
                                    const row = funnelRows.find((r) => r.countValue === Number(value));
                                    if (row && !row.hasData) return [t("dashboard.hr.effective_no_data_source"), t("dashboard.hr.effective_count")];
                                }
                                const displayValue = typeof value === "number" || typeof value === "string" ? String(value) : "";
                                const displayName = typeof name === "string" || typeof name === "number" ? String(name) : "";
                                return [displayValue, displayName];
                            }}
                        />
                        <Bar dataKey="countValue" radius={[0, 8, 8, 0]} />
                    </BarChart>
                </ResponsiveContainer>
            </div>

            <ChartTableTwin
                caption={t("dashboard.hr.effective_funnel_title")}
                columns={columns}
                rows={funnelRows}
                searchableKeys={["label"]}
            />
        </div>
    );
}

function RankingTab({ data, t }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn }) {
    const rows = data.by_source;
    const chartData = useMemo(
        () => rows
            .filter((r) => r.hires > 0)
            .map((r) => ({
                name: r.source_name,
                value: r.hires,
                fill: colorByEntityWithKnown(r.source_type, SOURCE_TYPE_COLORS),
                sourceType: r.source_type,
                retained: r.retained_90d,
                retention: r.retention_90d,
                cost: r.cost_per_hire,
                costAvailable: r.cost_per_hire_available,
                turnover: r.turnover_in_range,
            })),
        [rows],
    );

    if (chartData.length === 0) {
        return (
            <div className="flex items-center justify-center h-48 text-slate-400">
                {t("common.no_data")}
            </div>
        );
    }

    const columns: ChartTableColumn<(typeof chartData)[number]>[] = [
        { header: t("dashboard.hr.effective_source"), key: "name" },
        { header: t("dashboard.hr.effective_hires"), key: "value", numeric: true },
        { header: t("dashboard.hr.effective_retained_90d"), key: "retained", numeric: true },
        {
            header: t("dashboard.hr.effective_retention_90d"),
            key: "retention",
            numeric: true,
            render: (r) => (r.retention != null ? `${r.retention}%` : t("dashboard.hr.rate_no_value")),
        },
        {
            header: t("dashboard.hr.effective_cost_per_hire"),
            key: "cost",
            numeric: true,
            render: (r) => (r.costAvailable && r.cost != null ? r.cost : t("dashboard.hr.rate_no_value")),
        },
        { header: t("dashboard.hr.effective_turnover"), key: "turnover", numeric: true },
    ];

    return (
        <div>
            <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                {t("dashboard.hr.effective_ranking_title")}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 mb-4" variant="p">
                {t("dashboard.hr.effective_ranking_honesty_note")}
            </Typography>

            <div aria-label={t("dashboard.hr.effective_ranking_title")} role="img">
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
                        <Legend
                            wrapperStyle={{ fontSize: "9px", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.15em" }}
                        />
                        <Bar dataKey="value" radius={[0, 8, 8, 0]} />
                    </BarChart>
                </ResponsiveContainer>
            </div>

            <ChartTableTwin
                caption={t("dashboard.hr.effective_ranking_title")}
                columns={columns}
                rows={chartData}
                searchableKeys={["name"]}
            />
        </div>
    );
}

function TableTab({ data, t }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn }) {
    const rows = data.by_source;

    if (rows.length === 0) {
        return (
            <div className="flex items-center justify-center h-48 text-slate-400">
                {t("common.no_data")}
            </div>
        );
    }

    const columns: ChartTableColumn<(typeof rows)[number]>[] = [
        { header: t("dashboard.hr.effective_source"), key: "source_name" },
        { header: t("dashboard.hr.effective_source_type"), key: "source_type" },
        { header: t("dashboard.hr.effective_hires"), key: "hires", numeric: true },
        {
            header: t("dashboard.hr.effective_hires_ratio"),
            key: "hires_ratio",
            numeric: true,
            render: (r) => (r.hires_ratio != null ? `${r.hires_ratio}%` : t("dashboard.hr.rate_no_value")),
        },
        { header: t("dashboard.hr.effective_retained_90d"), key: "retained_90d", numeric: true },
        {
            header: t("dashboard.hr.effective_retention_90d"),
            key: "retention_90d",
            numeric: true,
            render: (r) => (r.retention_90d != null ? `${r.retention_90d}%` : t("dashboard.hr.rate_no_value")),
        },
        {
            header: t("dashboard.hr.effective_cost_per_hire"),
            key: "cost_per_hire",
            numeric: true,
            render: (r) => (r.cost_per_hire_available && r.cost_per_hire != null ? r.cost_per_hire : t("dashboard.hr.rate_no_value")),
        },
        { header: t("dashboard.hr.effective_turnover"), key: "turnover_in_range", numeric: true },
    ];

    return (
        <div>
            <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3" variant="h4">
                {t("dashboard.hr.effective_source_table")}
            </Typography>
            <ChartTableTwin
                caption={t("dashboard.hr.effective_source_table")}
                columns={columns}
                rows={rows}
                searchableKeys={["source_name", "source_type"]}
            />
        </div>
    );
}
