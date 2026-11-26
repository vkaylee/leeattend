import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { DashboardChartError } from "@/components/features/dashboard/dashboard-chart-state";
import { DashboardDateRange } from "@/components/features/dashboard/dashboard-date-range";
import {
    getHrPeriodOptions as getPeriodOptions,
    HrAnalyticsKpiCard as KpiCard,
    HrAnalyticsTabs as ChartTabs,
    HrAnalyticsWarningStrip as WarningStrip,
    type PeriodOption,
    type TranslateFn,
} from "@/components/features/dashboard/dashboard-hr-shared";
import {
    RecruitmentFunnelTab as FunnelTab,
    RecruitmentOverviewTab as OverviewTab,
    RecruitmentRankingTab as RankingTab,
    RecruitmentTableTab as TableTab,
} from "@/components/features/dashboard/recruitment-source-tabs";
import { Typography } from "@/components/ui/typography";
import {
    useRecruitmentEffectiveness,
    type RecruitmentEffectivenessAnalytics,
} from "@/hooks/use-dashboard";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDepartments } from "@/hooks/use-departments";
import { cn } from "@/lib/utils";

type TabKey = "overview" | "funnel" | "ranking" | "table";

type ComparisonKey = "none" | "previous_period" | "same_period_last_year";

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
