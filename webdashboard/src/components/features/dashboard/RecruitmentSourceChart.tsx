import { useTranslation } from "react-i18next";
import {
    PieChart,
    Pie,
    Cell,
    ResponsiveContainer,
    Tooltip,
    Legend,
} from "recharts";

import { useRecruitmentSourceBreakdown } from "@/hooks/use-dashboard";

const SOURCE_COLORS: Record<string, string> = {
    direct: "#22c55e",   // green
    referral: "#3b82f6", // blue
    agency: "#f59e0b",   // amber
};

const FALLBACK_COLORS = [
    "#06b6d4", "#10b981", "#ec4899", "#14b8a6", "#f97316",
];

function getColor(sourceType: string, index: number): string {
    return SOURCE_COLORS[sourceType] || FALLBACK_COLORS[index % FALLBACK_COLORS.length];
}

export default function RecruitmentSourceChart() {
    const { t } = useTranslation();
    const { data, isLoading } = useRecruitmentSourceBreakdown();

    if (isLoading) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm animate-pulse">
                <div className="h-4 w-40 bg-slate-200 dark:bg-slate-700 rounded-full mb-8" />
                <div className="h-64 bg-slate-100 dark:bg-slate-800 rounded-3xl" />
            </div>
        );
    }

    const chartData = (data ?? [])
        .filter((d) => d.count > 0)
        .map((d, i) => ({
            name: d.source_name,
            value: d.count,
            sourceType: d.source_type,
            fill: getColor(d.source_type, i),
        }));

    const totalHires = chartData.reduce((acc, d) => acc + d.value, 0);

    if (chartData.length === 0) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
                <h3 className="text-lg font-semibold text-slate-800 dark:text-slate-200">
                    {t("dashboard.hr.recruitment_sources")}
                </h3>
                <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                    {t("dashboard.hr.recruitment_sources_subtitle")}
                </p>
                <div className="flex items-center justify-center h-64 text-slate-400">
                    {t("common.no_data")}
                </div>
            </div>
        );
    }

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <div className="mb-6">
                <h3 className="text-lg font-semibold text-slate-800 dark:text-slate-200">
                    {t("dashboard.hr.recruitment_sources")}
                </h3>
                <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                    {t("dashboard.hr.recruitment_sources_subtitle")}
                </p>
            </div>

            <div className="flex flex-col lg:flex-row items-center gap-4">
                <div className="w-full lg:w-1/2" style={{ height: 260 }}>
                    <ResponsiveContainer width="100%" height="100%">
                        <PieChart>
                            <Pie
                                data={chartData}
                                cx="50%"
                                cy="50%"
                                innerRadius={60}
                                outerRadius={100}
                                paddingAngle={3}
                                dataKey="value"
                                nameKey="name"
                                stroke="none"
                            >
                                {chartData.map((entry, index) => (
                                    <Cell
                                        key={`cell-${index}`}
                                        fill={entry.fill}
                                    />
                                ))}
                            </Pie>
                            <Tooltip
                                contentStyle={{
                                    borderRadius: "12px",
                                    border: "none",
                                    boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
                                }}
                                formatter={(value: number | undefined) => [
                                    `${value ?? 0} (${(((value ?? 0) / totalHires) * 100).toFixed(1)}%)`,
                                    t("dashboard.hr.people"),
                                ]}
                            />
                            <Legend
                                verticalAlign="bottom"
                                iconType="circle"
                                iconSize={8}
                            />
                        </PieChart>
                    </ResponsiveContainer>
                </div>

                {/* Inline legend with counts */}
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
                                {entry.name}: {entry.value}
                            </span>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
