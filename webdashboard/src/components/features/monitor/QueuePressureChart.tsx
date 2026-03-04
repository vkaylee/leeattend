import { useTranslation } from "react-i18next";
import { BarChart, Bar, XAxis, YAxis, ResponsiveContainer, Tooltip, Cell } from "recharts";
import { Layers } from "lucide-react";
import type { QueuePressureData } from "@/hooks/use-monitor-charts";

const COLORS = { high: "#f97316", low: "#fbbf24" };

interface Props {
    data: QueuePressureData;
    isLoading: boolean;
}

const QueuePressureChart = ({ data, isLoading }: Props) => {
    const { t } = useTranslation();

    const chartData = [
        { name: t("monitor.pending_high"), value: data.high, color: COLORS.high },
        { name: t("monitor.pending_low"), value: data.low, color: COLORS.low },
    ];

    const hasData = data.total > 0;

    return (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] p-8 border border-slate-100 dark:border-slate-800 shadow-sm dark:shadow-black/10">
            <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 rounded-xl flex items-center justify-center border bg-orange-50 dark:bg-orange-950 border-orange-100 dark:border-orange-800 text-orange-500">
                    <Layers size={20} />
                </div>
                <div>
                    <h3 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                        {t("monitor.charts.queue_pressure")}
                    </h3>
                    <p className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5">
                        {t("monitor.charts.queue_pressure_desc")}
                    </p>
                </div>
            </div>

            {isLoading ? (
                <div className="h-[180px] flex items-center justify-center">
                    <div className="w-8 h-8 border-2 border-slate-200 dark:border-slate-700 border-t-slate-500 rounded-full animate-spin" />
                </div>
            ) : !hasData ? (
                <div className="h-[180px] flex flex-col items-center justify-center">
                    <Layers size={40} className="text-slate-200 dark:text-slate-700 mb-3" />
                    <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
                        {t("monitor.charts.queue_empty", { defaultValue: "Queue empty" })}
                    </p>
                </div>
            ) : (
                <>
                    <div className="text-center mb-4">
                        <span className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter">
                            {data.total}
                        </span>
                        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest ml-2">
                            {t("monitor.pending_tasks")}
                        </span>
                    </div>
                    <ResponsiveContainer width="100%" height={120}>
                        <BarChart data={chartData} layout="vertical" margin={{ top: 0, right: 10, left: 0, bottom: 0 }}>
                            <XAxis type="number" hide />
                            <YAxis
                                type="category"
                                dataKey="name"
                                width={90}
                                tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                                className="text-slate-400 dark:text-slate-500"
                                tickLine={false}
                                axisLine={false}
                            />
                            <Tooltip
                                contentStyle={{
                                    borderRadius: "1rem",
                                    border: "none",
                                    boxShadow: "0 8px 30px rgba(0,0,0,.1)",
                                    fontSize: "11px",
                                    fontWeight: 600,
                                }}
                            />
                            <Bar dataKey="value" radius={[0, 6, 6, 0]} barSize={20}>
                                {chartData.map((entry, index) => (
                                    <Cell key={index} fill={entry.color} />
                                ))}
                            </Bar>
                        </BarChart>
                    </ResponsiveContainer>
                </>
            )}
        </div>
    );
};

export default QueuePressureChart;
