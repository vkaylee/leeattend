import { useMemo } from "react";

import {
    Bar,
    BarChart,
    CartesianGrid,
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
import {
    HR_CHART_TOOLTIP_STYLE as TOOLTIP_STYLE,
    type TranslateFn,
} from "@/components/features/dashboard/dashboard-hr-shared";
import { Typography } from "@/components/ui/typography";
import type { TurnoverAnalytics } from "@/hooks/use-dashboard";
import { colorByEntity } from "@/lib/stable-colors";

export type TurnoverTabKey = "overview" | "tenure" | "source" | "department" | "reasons" | "breakdown";

const TREND_RATE_COLOR = "#6366f1";
const TREND_LEAVERS_COLOR = "#ef4444";
const CLASSIFICATION_COLORS: Record<string, string> = {
    voluntary: "#22c55e",
    involuntary: "#ef4444",
    unknown: "#94a3b8",
};

export function ActiveTurnoverTab({
    activeTab,
    data,
    formatDate,
    t,
}: {
    activeTab: TurnoverTabKey;
    data: TurnoverAnalytics;
    formatDate: (d: string | Date | null | undefined) => string;
    t: TranslateFn;
}) {
    switch (activeTab) {
        case "overview":
            return <OverviewTab data={data} formatDate={formatDate} t={t} />;
        case "tenure":
            return <TenureTab data={data} t={t} />;
        case "source":
            return <SourceTab data={data} t={t} />;
        case "department":
            return <DepartmentTab data={data} t={t} />;
        case "reasons":
            return <ReasonsTab data={data} t={t} />;
        case "breakdown":
            return <BreakdownTab data={data} t={t} />;
    }
}

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
                                <th className="px-4 py-3 font-bold whitespace-nowrap" key={col.key} scope="col">
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
                            chartData.map((row) => (
                                <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/40" key={`${row.cohort_month}-${row.months_since_hire}`}>
                                    {columns.map((col) => (
                                        <td className="px-4 py-2.5 whitespace-nowrap" key={col.key}>
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
    t: _t,
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
                        <Bar dataKey="value" radius={[0, 8, 8, 0]} />
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
                                <Bar dataKey="value" radius={[0, 8, 8, 0]} />
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
        <div className="grid gap-8 xl:grid-cols-2">
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
            <div className="xl:col-span-2">
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
