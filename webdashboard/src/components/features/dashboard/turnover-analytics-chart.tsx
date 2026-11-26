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
    ActiveTurnoverTab,
    type TurnoverTabKey as TabKey,
} from "@/components/features/dashboard/turnover-analytics-tabs";
import { Typography } from "@/components/ui/typography";
import {
    useTurnoverAnalytics,
    type TurnoverAnalytics,
    type TurnoverRate,
} from "@/hooks/use-dashboard";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { cn } from "@/lib/utils";

type ComparisonKey = "none" | "previous_period" | "same_period_last_year";

function rateDisplay(rate: TurnoverRate | undefined, t: TranslateFn): string {
    if (!rate) return t("dashboard.hr.rate_no_value");
    if (rate.value == null) return t("dashboard.hr.rate_no_value");
    return t("dashboard.hr.rate_with_denominator", {
        rate: `${rate.value}%`,
        numerator: String(rate.numerator),
        denominator: String(rate.denominator),
    });
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

    {
        const value = `${target.value}%`;
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
            <TurnoverContent
                activeTab={activeTab}
                data={data}
                formatDate={formatDate}
                setActiveTab={setActiveTab}
                t={t}
                tabs={tabs}
            />
        </div>
    );
}

// ── Tabs ──────────────────────────────────────────────────────────────

function isEmptyTurnoverData(data: TurnoverAnalytics): boolean {
    return data.tenure_distribution.length === 0 &&
        data.by_source.length === 0 &&
        data.by_department.length === 0 &&
        data.termination_reasons.length === 0 &&
        data.monthly_trend.length === 0;
}

function TurnoverContent({ activeTab, data, formatDate, t, tabs, setActiveTab }: {
    activeTab: TabKey;
    data: TurnoverAnalytics | undefined;
    formatDate: (d: string | Date | null | undefined) => string;
    setActiveTab: (tab: TabKey) => void;
    t: TranslateFn;
    tabs: { key: TabKey; labelKey: string }[];
}) {
    if (data) {
        return (
            <>
                <WarningStrip t={t} warnings={data.warnings} />
                <ChartTabs activeTab={activeTab} setActiveTab={setActiveTab} t={t} tabs={tabs} />
                {isEmptyTurnoverData(data) ? (
                    <div className="flex items-center justify-center h-64 text-slate-400">
                        {t("common.no_data")}
                    </div>
                ) : (
                    <div className="space-y-6">
                        <KpiHeader data={data} t={t} />
                        <ActiveTurnoverTab activeTab={activeTab} data={data} formatDate={formatDate} t={t} />
                    </div>
                )}
            </>
        );
    }

    return (
        <>
            <ChartTabs activeTab={activeTab} setActiveTab={setActiveTab} t={t} tabs={tabs} />
            <div className="flex items-center justify-center h-64 text-slate-400">
                {t("common.no_data")}
            </div>
        </>
    );
}
