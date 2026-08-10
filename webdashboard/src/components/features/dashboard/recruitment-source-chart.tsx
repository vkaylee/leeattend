import { useTranslation } from "react-i18next";

import {
    PieChart,
    Pie,
    Cell,
    ResponsiveContainer,
    Tooltip,
    Legend,
} from "recharts";

import { Typography } from "@/components/ui/typography";
import { DashboardChartError } from "@/components/features/dashboard/dashboard-chart-state";
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

function getRecruitmentChartData(data: { source_type: string; source_name: string; count: number }[] | undefined) {
    const chartData = (data ?? [])
        .filter((d) => d.count > 0)
        .map((d, index) => ({
            name: d.source_name,
            value: d.count,
            sourceType: d.source_type,
            fill: getColor(d.source_type, index),
        }));

    const totalHires = chartData.reduce((accumulator, d) => accumulator + d.value, 0);

    return { chartData, totalHires };
}

function RecruitmentHeader({ t }: { t: (k: string) => string }) {
    return (
        <div className="mb-6">
            <Typography className="text-lg font-semibold text-slate-800 dark:text-slate-200" variant="h3">
                {t("dashboard.hr.recruitment_sources")}
            </Typography>
            <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1" variant="p">
                {t("dashboard.hr.recruitment_sources_subtitle")}
            </Typography>
        </div>
    );
}

interface RecruitmentChartEntry {
    fill: string;
    name: string;
    sourceType: string;
    value: number;
}

function RecruitmentLegend({ chartData }: { chartData: RecruitmentChartEntry[] }) {
    return (
        <div className="w-full lg:w-1/2 space-y-2">
            {chartData.map((entry, index) => (
                <div
                    className="flex items-center justify-between px-3 py-2 rounded-xl bg-slate-50 dark:bg-slate-800/50"
                    key={index}
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
    );
}


export default function RecruitmentSourceChart() {
    const { t } = useTranslation();
    const { data, error, isLoading } = useRecruitmentSourceBreakdown();

    if (isLoading) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm animate-pulse">
                <div className="h-4 w-40 bg-slate-200 dark:bg-slate-700 rounded-full mb-8" />
                <div className="h-64 bg-slate-100 dark:bg-slate-800 rounded-3xl" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
                <RecruitmentHeader t={t} />
                <DashboardChartError icon={<span className="text-4xl" aria-hidden="true">!</span>} />
            </div>
        );
    }

    const { chartData, totalHires } = getRecruitmentChartData(data);

    if (chartData.length === 0) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
                <RecruitmentHeader t={t} />
                <div className="flex items-center justify-center h-64 text-slate-400">
                    {t("common.no_data")}
                </div>
            </div>
        );
    }

    return (
        <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
            <RecruitmentHeader t={t} />

            <div className="flex flex-col lg:flex-row items-center gap-4">
                <div className="w-full lg:w-1/2 h-[260px]">
                    <ResponsiveContainer height="100%" width="100%">
                        <PieChart>
                            <Pie
                                cx="50%"
                                cy="50%"
                                data={chartData}
                                dataKey="value"
                                innerRadius={60}
                                nameKey="name"
                                outerRadius={100}
                                paddingAngle={3}
                                stroke="none"
                            >
                                {chartData.map((entry, index) => (
                                    // eslint-disable-next-line @typescript-eslint/no-deprecated
                                    <Cell
                                        fill={entry.fill}
                                        key={`cell-${index}`}
                                    />
                                ))}
                            </Pie>
                            <Tooltip
                                contentStyle={{
                                    borderRadius: "12px",
                                    border: "none",
                                    boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
                                }}
                                formatter={(value: unknown) => [
                                    `${Number(value ?? 0)} (${((Number(value ?? 0) / (totalHires || 1)) * 100).toFixed(1)}%)`,
                                    t("dashboard.hr.people"),
                                ]}
                            />
                            <Legend
                                iconSize={8}
                                iconType="circle"
                                verticalAlign="bottom"
                            />
                        </PieChart>
                    </ResponsiveContainer>
                </div>

                <RecruitmentLegend chartData={chartData} />
            </div>
        </div>
    );
}


