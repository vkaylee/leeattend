import { useTranslation } from "react-i18next";

import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";

import type { QueuePressureData, PressureLevel } from "@/hooks/use-monitor-charts";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface Props {
  data: QueuePressureData;
  isLoading: boolean;
}

const PRESSURE_CONFIG: Record<PressureLevel | "unavailable", { label: string; color: string; bg: string; border: string }> = {
  low: {
    label: "Low",
    color: "text-emerald-600 dark:text-emerald-300",
    bg: "bg-emerald-50 dark:bg-emerald-950/60",
    border: "border-emerald-100 dark:border-emerald-800/50",
  },
  medium: {
    label: "Medium",
    color: "text-amber-600 dark:text-amber-300",
    bg: "bg-amber-50 dark:bg-amber-950/60",
    border: "border-amber-100 dark:border-amber-800/50",
  },
  high: {
    label: "High",
    color: "text-orange-600 dark:text-orange-300",
    bg: "bg-orange-50 dark:bg-orange-950/60",
    border: "border-orange-100 dark:border-orange-800/50",
  },
  critical: {
    label: "Critical",
    color: "text-rose-600 dark:text-rose-300",
    bg: "bg-rose-50 dark:bg-rose-950/60",
    border: "border-rose-100 dark:border-rose-800/50",
  },
  unavailable: {
    label: "Unavailable",
    color: "text-slate-500 dark:text-slate-400",
    bg: "bg-slate-100 dark:bg-slate-800/60",
    border: "border-slate-200 dark:border-slate-700",
  },
};

export default function QueuePressureChart({ data, isLoading }: Props) {
  const { t } = useTranslation();
  const pressure = PRESSURE_CONFIG[data.pressureLevel];

  const chartData = [
    { name: t("monitor.pending_high"), value: data.high, fill: "#f97316" },
    { name: t("monitor.pending_normal", { defaultValue: "Normal priority" }), value: data.normal, fill: "#3b82f6" },
    { name: t("monitor.pending_low"), value: data.low, fill: "#fbbf24" },
  ];

  return (
    <div className="bg-white/70 dark:bg-slate-900/70 backdrop-blur-xl border border-slate-200/60 dark:border-slate-700/60 rounded-[2rem] p-6 shadow-lg dark:shadow-black/20 flex flex-col gap-4 relative overflow-hidden group">
      {/* Hover glow */}
      <div className="absolute inset-0 opacity-0 group-hover:opacity-100 motion-safe:transition-opacity duration-700 motion-reduce:transition-none pointer-events-none">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-40 h-40 rounded-full bg-gradient-to-br from-orange-400/15 to-amber-400/10 blur-2xl" />
      </div>

      <div className="flex items-center justify-between">
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h5">
          {t("monitor.queue_pressure")}
        </Typography>
        {/* Pressure level badge */}
        <span
          className={cn(
            "inline-flex items-center px-2.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider border",
            pressure.color,
            pressure.bg,
            pressure.border
          )}
          data-testid="pressure-badge"
        >
          {data.unavailable
            ? t("monitor.metric_unavailable", { defaultValue: "Unavailable" })
            : t(`monitor.pressure_levels.${data.pressureLevel}`, { defaultValue: pressure.label })}
        </span>
      </div>

      {isLoading ? (
        <div className="h-[160px] flex items-center justify-center">
          <div className="w-20 h-20 rounded-full border-4 border-slate-100 dark:border-slate-700 border-t-orange-400 motion-safe:animate-spin" />
        </div>
      ) : data.unavailable ? (
        <div className="h-[160px] flex flex-col items-center justify-center gap-2 text-center" role="status">
          <div className="w-12 h-12 rounded-2xl bg-slate-100 dark:bg-slate-800/60 flex items-center justify-center">
            <span aria-hidden="true" className="text-xl text-slate-400">—</span>
          </div>
          <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
            {t("monitor.metric_unavailable", { defaultValue: "Metric unavailable" })}
          </Typography>
        </div>
      ) : data.total === 0 ? (
        <div className="h-[160px] flex flex-col items-center justify-center gap-2">
          <div className="w-12 h-12 rounded-2xl bg-emerald-50 dark:bg-emerald-950/40 flex items-center justify-center">
            <span aria-hidden="true" className="text-xl">✓</span>
          </div>
          <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
            {t("monitor.queue_empty", { defaultValue: "Queue clear" })}
          </Typography>
        </div>
      ) : (
        <ResponsiveContainer height={160} width="100%">
          <BarChart barCategoryGap="30%" data={chartData} layout="vertical">
            <CartesianGrid
              horizontal={false}
              stroke="rgba(148,163,184,0.15)"
              strokeDasharray="3 3"
            />
            <XAxis hide type="number" />
            <YAxis
              axisLine={false}
              dataKey="name"
              tick={{ fontSize: 10, fontWeight: 700, fill: "#94a3b8" }}
              tickLine={false}
              type="category"
              width={80}
            />
            <Tooltip
              contentStyle={{
                borderRadius: "1rem",
                border: "1px solid rgba(148,163,184,0.2)",
                background: "rgba(255,255,255,0.95)",
                backdropFilter: "blur(8px)",
                fontSize: "12px",
                fontWeight: 600,
              }}
            />
            <Bar barSize={20} dataKey="value" radius={[0, 8, 8, 0]} />
          </BarChart>
        </ResponsiveContainer>
      )}

      <div className="flex items-baseline justify-center gap-2 text-center">
        <span className="text-2xl font-black text-slate-700 dark:text-slate-200 tracking-tighter tabular-nums">
          {data.total ?? "—"}
        </span>
        <span className="text-xs font-medium text-slate-500 dark:text-slate-400">
          {t("monitor.total_pending", { defaultValue: "tổng số đang chờ" })}
        </span>
      </div>

      <table className="sr-only">
        <caption>{t("monitor.queue_pressure_table", { defaultValue: "Queue pressure values" })}</caption>
        <thead>
          <tr>
            <th scope="col">{t("monitor.priority", { defaultValue: "Priority" })}</th>
            <th scope="col">{t("monitor.count", { defaultValue: "Count" })}</th>
          </tr>
        </thead>
        <tbody>
          {chartData.map((entry) => (
            <tr key={entry.name}>
              <th scope="row">{entry.name}</th>
              <td>{entry.value ?? "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
