import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
    BarChart,
    Bar,
    XAxis,
    YAxis,
    CartesianGrid,
    PieChart,
    Pie,
    Cell,
    ResponsiveContainer,
    Tooltip,
    Legend,
} from "recharts";
import { useTurnoverAnalytics } from "@/hooks/use-dashboard";

type TabKey = "tenure" | "source" | "department";

interface PeriodOption {
    label: string;
    value: string;
    startDate: string;
    endDate: string;
}

function formatDate(d: Date): string {
    return d.toISOString().slice(0, 10);
}

function getPeriodOptions(t: (key: string) => string): PeriodOption[] {
    const today = new Date();
    const endDate = formatDate(today);

    const months3 = new Date(today);
    months3.setMonth(months3.getMonth() - 3);

    const months6 = new Date(today);
    months6.setMonth(months6.getMonth() - 6);

    const year1 = new Date(today);
    year1.setFullYear(year1.getFullYear() - 1);

    const year2 = new Date(today);
    year2.setFullYear(year2.getFullYear() - 2);

    return [
        { label: t("dashboard.hr.turnover_period_3m"), value: "3m", startDate: formatDate(months3), endDate },
        { label: t("dashboard.hr.turnover_period_6m"), value: "6m", startDate: formatDate(months6), endDate },
        { label: t("dashboard.hr.turnover_period_1y"), value: "1y", startDate: formatDate(year1), endDate },
        { label: t("dashboard.hr.turnover_period_2y"), value: "2y", startDate: formatDate(year2), endDate },
    ];
}

const SOURCE_COLORS = [
    "#22c55e", "#3b82f6", "#f59e0b", "#06b6d4", "#8b5cf6",
    "#ec4899", "#14b8a6", "#f97316",
];

const TENURE_COLOR = "#6366f1";
const DEPT_COLOR = "#3b82f6";

export default function TurnoverAnalyticsChart() {
    const { t } = useTranslation();
    const [activeTab, setActiveTab] = useState<TabKey>("tenure");
    const [selectedPeriod, setSelectedPeriod] = useState("6m");

    const periods = useMemo(() => getPeriodOptions(t), [t]);
    const currentPeriod = periods.find((p) => p.value === selectedPeriod) ?? periods[1];

    const { data, isLoading } = useTurnoverAnalytics(
        currentPeriod.startDate,
        currentPeriod.endDate
    );

    const tabs: { key: TabKey; labelKey: string }[] = [
        { key: "tenure", labelKey: "dashboard.hr.turnover_tenure" },
        { key: "source", labelKey: "dashboard.hr.turnover_by_source" },
        { key: "department", labelKey: "dashboard.hr.turnover_by_dept" },
    ];

    if (isLoading) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm animate-pulse">
                <div className="h-4 w-48 bg-slate-200 dark:bg-slate-700 rounded-full mb-8" />
                <div className="h-64 bg-slate-100 dark:bg-slate-800 rounded-3xl" />
            </div>
        );
    }

    const isEmpty =
        !data ||
        (data.tenure_distribution.length === 0 &&
            data.by_source.length === 0 &&
            data.by_department.length === 0);

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            {/* Header with period selector */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
                <div>
                    <h3 className="text-lg font-semibold text-slate-800 dark:text-slate-200">
                        {t("dashboard.hr.turnover_title")}
                    </h3>
                    <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                        {t("dashboard.hr.turnover_subtitle")}
                    </p>
                </div>

                <div className="flex items-center gap-3">
                    {/* Total badge */}
                    {data && (
                        <span className="text-xs font-bold text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-950 px-4 py-1.5 rounded-full uppercase tracking-widest border border-rose-100 dark:border-rose-800 shadow-sm">
                            {t("dashboard.hr.turnover_total")}: {data.total_terminated}
                        </span>
                    )}

                    {/* Period selector */}
                    <select
                        value={selectedPeriod}
                        onChange={(e) => setSelectedPeriod(e.target.value)}
                        className="text-xs font-bold text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-800 px-4 py-2 rounded-xl border border-slate-200 dark:border-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 uppercase tracking-widest cursor-pointer"
                    >
                        {periods.map((p) => (
                            <option key={p.value} value={p.value}>
                                {p.label}
                            </option>
                        ))}
                    </select>
                </div>
            </div>

            {/* Tab navigation */}
            <div className="flex bg-slate-100 dark:bg-slate-800/50 p-1 rounded-2xl w-fit mb-6 border border-slate-200/50 dark:border-slate-700/50">
                {tabs.map((tab) => (
                    <button
                        key={tab.key}
                        onClick={() => setActiveTab(tab.key)}
                        className={`px-5 py-2 rounded-xl text-xs font-bold uppercase tracking-widest transition-all ${activeTab === tab.key
                                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                            }`}
                    >
                        {t(tab.labelKey)}
                    </button>
                ))}
            </div>

            {/* Chart content */}
            {isEmpty ? (
                <div className="flex items-center justify-center h-64 text-slate-400">
                    {t("common.no_data")}
                </div>
            ) : (
                <div style={{ height: 320 }}>
                    {activeTab === "tenure" && <TenureChart data={data!.tenure_distribution} />}
                    {activeTab === "source" && <SourceChart data={data!.by_source} t={t} />}
                    {activeTab === "department" && <DepartmentChart data={data!.by_department} />}
                </div>
            )}
        </div>
    );
}

// ── Sub-charts ──────────────────────────────────────────────────────

function TenureChart({ data }: { data: { bucket: string; count: number }[] }) {
    return (
        <ResponsiveContainer width="100%" height="100%">
            <BarChart data={data} margin={{ top: 10, right: 20, left: 0, bottom: 20 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
                <XAxis
                    dataKey="bucket"
                    tick={{ fontSize: 11, fill: "#94a3b8" }}
                    axisLine={false}
                    tickLine={false}
                />
                <YAxis
                    allowDecimals={false}
                    tick={{ fontSize: 11, fill: "#94a3b8" }}
                    axisLine={false}
                    tickLine={false}
                />
                <Tooltip
                    contentStyle={{
                        borderRadius: "12px",
                        border: "none",
                        boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
                        fontSize: 12,
                    }}
                />
                <Bar dataKey="count" fill={TENURE_COLOR} radius={[8, 8, 0, 0]} />
            </BarChart>
        </ResponsiveContainer>
    );
}

function SourceChart({
    data,
    t,
}: {
    data: { source_type: string; source_name: string; count: number }[];
    t: (key: string) => string;
}) {
    const chartData = data
        .filter((d) => d.count > 0)
        .map((d, i) => ({
            name: d.source_name,
            value: d.count,
            fill: SOURCE_COLORS[i % SOURCE_COLORS.length],
        }));

    const total = chartData.reduce((acc, d) => acc + d.value, 0);

    return (
        <div className="flex flex-col lg:flex-row items-center gap-4 h-full">
            <div className="w-full lg:w-1/2 h-full">
                <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                        <Pie
                            data={chartData}
                            cx="50%"
                            cy="50%"
                            innerRadius={55}
                            outerRadius={95}
                            paddingAngle={3}
                            dataKey="value"
                            nameKey="name"
                            stroke="none"
                        >
                            {chartData.map((entry, index) => (
                                <Cell key={`cell-${index}`} fill={entry.fill} />
                            ))}
                        </Pie>
                        <Tooltip
                            contentStyle={{
                                borderRadius: "12px",
                                border: "none",
                                boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
                            }}
                            formatter={(value: number | undefined) => [
                                `${value ?? 0} (${(((value ?? 0) / total) * 100).toFixed(1)}%)`,
                                t("dashboard.hr.people"),
                            ]}
                        />
                        <Legend verticalAlign="bottom" iconType="circle" iconSize={8} />
                    </PieChart>
                </ResponsiveContainer>
            </div>
            <div className="w-full lg:w-1/2 space-y-2">
                {chartData.map((entry, index) => (
                    <div
                        key={index}
                        className="flex items-center justify-between px-3 py-2 rounded-xl bg-slate-50 dark:bg-slate-800/50"
                    >
                        <div className="flex items-center gap-2">
                            <span
                                className="w-3 h-3 rounded-full inline-block"
                                style={{ backgroundColor: entry.fill }}
                            />
                            <span className="text-sm text-slate-700 dark:text-slate-300">
                                {entry.name}
                            </span>
                        </div>
                        <span className="text-sm font-medium text-slate-900 dark:text-slate-100">
                            {entry.value}
                        </span>
                    </div>
                ))}
            </div>
        </div>
    );
}

function DepartmentChart({ data }: { data: { department_name: string; count: number }[] }) {
    return (
        <ResponsiveContainer width="100%" height="100%">
            <BarChart
                data={data}
                layout="vertical"
                margin={{ top: 10, right: 20, left: 10, bottom: 10 }}
            >
                <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" horizontal={false} />
                <XAxis
                    type="number"
                    allowDecimals={false}
                    tick={{ fontSize: 11, fill: "#94a3b8" }}
                    axisLine={false}
                    tickLine={false}
                />
                <YAxis
                    dataKey="department_name"
                    type="category"
                    tick={{ fontSize: 11, fill: "#94a3b8" }}
                    axisLine={false}
                    tickLine={false}
                    width={120}
                />
                <Tooltip
                    contentStyle={{
                        borderRadius: "12px",
                        border: "none",
                        boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
                        fontSize: 12,
                    }}
                />
                <Bar dataKey="count" fill={DEPT_COLOR} radius={[0, 8, 8, 0]} />
            </BarChart>
        </ResponsiveContainer>
    );
}
