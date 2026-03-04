import { useTranslation } from "react-i18next";
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from "recharts";
import { PieChartIcon } from "lucide-react";
import type { JobStatusCount } from "@/hooks/use-monitor-charts";

interface Props {
    data: JobStatusCount[];
    isLoading: boolean;
}

const JobStatusDistributionChart = ({ data, isLoading }: Props) => {
    const { t } = useTranslation();
    const totalJobs = data.reduce((sum, d) => sum + d.count, 0);
    const hasData = totalJobs > 0;

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] p-8 border border-slate-100 dark:border-slate-800 shadow-sm dark:shadow-black/10">
            <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 rounded-xl flex items-center justify-center border bg-violet-50 dark:bg-violet-950 border-violet-100 dark:border-violet-800 text-violet-500">
                    <PieChartIcon size={20} />
                </div>
                <div>
                    <h3 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                        {t("monitor.charts.job_distribution")}
                    </h3>
                    <p className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5">
                        {t("monitor.charts.job_distribution_desc")}
                    </p>
                </div>
            </div>

            {isLoading ? (
                <div className="h-[180px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-slate-200 dark:border-slate-700 border-t-slate-500 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <div className="h-[180px] flex flex-col items-center justify-center">
                    <PieChartIcon size={40} className="text-slate-200 dark:text-slate-700 mb-3" />
                    <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
                        {t("common.no_data")}
                    </p>
                </div>
            ) : (
                <div className="relative h-[180px]">
                    <ResponsiveContainer width="100%" height="100%">
                        <PieChart>
                            <Pie
                                data={data}
                                cx="50%"
                                cy="50%"
                                innerRadius={55}
                                outerRadius={80}
                                dataKey="count"
                                nameKey="status"
                                paddingAngle={3}
                                strokeWidth={0}
                            >
                                {data.map((entry, index) => (
                                    <Cell key={index} fill={entry.color} />
                                ))}
                            </Pie>
                            <Tooltip
                                contentStyle={{
                                    borderRadius: "1rem",
                                    border: "none",
                                    boxShadow: "0 8px 30px rgba(0,0,0,.1)",
                                    fontSize: "11px",
                                    fontWeight: 600,
                                }}
                                formatter={(value, name) => [value, name]}
                            />
                        </PieChart>
                    </ResponsiveContainer>
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                        <span className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter">
                            {totalJobs}
                        </span>
                        <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest">
                            {t("monitor.charts.total_jobs", { defaultValue: "jobs" })}
                        </span>
                    </div>
                </div>
            )}

            {/* Legend */}
            {hasData && !isLoading && (
                <div className="flex flex-wrap justify-center gap-3 mt-3">
                    {data.map((entry) => (
                        <div key={entry.status} className="flex items-center gap-1.5">
                            <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: entry.color }} />
                            <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                                {entry.status} ({entry.count})
                            </span>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default JobStatusDistributionChart;
