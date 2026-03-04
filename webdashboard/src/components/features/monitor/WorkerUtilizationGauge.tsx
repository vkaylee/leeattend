import { useTranslation } from "react-i18next";
import { RadialBarChart, RadialBar, ResponsiveContainer, PolarAngleAxis } from "recharts";
import { Gauge } from "lucide-react";
import type { UtilizationData } from "@/hooks/use-monitor-charts";

const getUtilizationColor = (percent: number): string => {
    if (percent > 80) return "#ef4444";
    if (percent >= 50) return "#f59e0b";
    return "#10b981";
};

const getUtilizationBg = (percent: number): string => {
    if (percent > 80) return "#fef2f2";
    if (percent >= 50) return "#fffbeb";
    return "#f0fdf4";
};

interface Props {
    data: UtilizationData;
    isLoading: boolean;
}

const WorkerUtilizationGauge = ({ data, isLoading }: Props) => {
    const { t } = useTranslation();
    const color = getUtilizationColor(data.percent);
    const bgColor = getUtilizationBg(data.percent);
    const chartData = [{ name: "utilization", value: data.percent, fill: color }];

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] p-8 border border-slate-100 dark:border-slate-800 shadow-sm dark:shadow-black/10">
            <div className="flex items-center gap-3 mb-6">
                <div
                    className="w-10 h-10 rounded-xl flex items-center justify-center border"
                    style={{ backgroundColor: bgColor, borderColor: `${color}30`, color }}
                >
                    <Gauge size={20} />
                </div>
                <div>
                    <h3 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                        {t("monitor.charts.utilization")}
                    </h3>
                    <p className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5">
                        {t("monitor.charts.utilization_desc")}
                    </p>
                </div>
            </div>

            {isLoading ? (
                <div className="h-[180px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-slate-200 dark:border-slate-700 border-t-slate-500 rounded-full animate-spin" />
                </div>
            ) : (
                <div className="relative h-[180px]">
                    <ResponsiveContainer width="100%" height="100%">
                        <RadialBarChart
                            cx="50%"
                            cy="50%"
                            innerRadius="70%"
                            outerRadius="100%"
                            startAngle={180}
                            endAngle={0}
                            data={chartData}
                            barSize={14}
                        >
                            <PolarAngleAxis type="number" domain={[0, 100]} angleAxisId={0} tick={false} />
                            <RadialBar
                                dataKey="value"
                                cornerRadius={8}
                                background={{ fill: "var(--chart-bg, #f1f5f9)" }}
                                angleAxisId={0}
                            />
                        </RadialBarChart>
                    </ResponsiveContainer>
                    <div className="absolute inset-0 flex flex-col items-center justify-center -mt-4">
                        <span className="text-3xl font-bold tracking-tighter" style={{ color }}>
                            {data.percent}%
                        </span>
                        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-1">
                            {data.active}/{data.total} {t("monitor.charts.active_label", { defaultValue: "active" })}
                        </span>
                    </div>
                </div>
            )}

            {!isLoading && data.percent > 80 && (
                <p className="text-[10px] font-bold text-rose-500 uppercase tracking-widest text-center mt-2">
                    ⚠ {t("monitor.charts.scale_up_warning")}
                </p>
            )}
            {!isLoading && data.total > 0 && data.percent < 30 && (
                <p className="text-[10px] font-bold text-emerald-500 uppercase tracking-widest text-center mt-2">
                    ↓ {t("monitor.charts.scale_down_hint")}
                </p>
            )}
        </div>
    );
};

export default WorkerUtilizationGauge;
