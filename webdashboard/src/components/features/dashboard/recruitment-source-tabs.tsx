import { useMemo } from "react";

import {
    Bar,
    BarChart,
    CartesianGrid,
    Cell,
    Legend,
    Line,
    LineChart,
    Rectangle,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";

import { ChartTableTwin, type ChartTableColumn } from "@/components/features/dashboard/chart-table-twin";
import {
    HR_CHART_TOOLTIP_STYLE as TOOLTIP_STYLE,
    type TranslateFn,
} from "@/components/features/dashboard/dashboard-hr-shared";
import { Typography } from "@/components/ui/typography";
import type { RecruitmentEffectivenessAnalytics } from "@/hooks/use-dashboard";
import { colorByEntityWithKnown } from "@/lib/stable-colors";

/** Stable per-source-type colors — a source keeps the same hue across re-renders. */
const SOURCE_TYPE_COLORS: Record<string, string> = {
    direct: "#22c55e",   // green
    referral: "#3b82f6", // blue
    agency: "#f59e0b",   // amber
    unknown: "#94a3b8",  // slate
};

const HIRES_TREND_COLOR = "#6366f1";

export function RecruitmentOverviewTab({
    data,
    t,
    formatDate,
}: {
    data: RecruitmentEffectivenessAnalytics;
    formatDate: (d: string | Date | null | undefined) => string;
    t: TranslateFn;
}) {
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

export function RecruitmentFunnelTab({ data, t }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn }) {
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

export function RecruitmentRankingTab({ data, t }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn }) {
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
                        <Bar dataKey="value" radius={[0, 8, 8, 0]} shape={(props) => <Rectangle {...props} fill={chartData[props.index ?? 0]?.fill} />}>
                            {chartData.map((row) => <Cell fill={row.fill} key={row.name} />)}
                        </Bar>
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

export function RecruitmentTableTab({ data, t }: { data: RecruitmentEffectivenessAnalytics; t: TranslateFn }) {
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
