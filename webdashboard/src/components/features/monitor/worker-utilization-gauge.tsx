import { Zap, Coffee } from "lucide-react";
import { useTranslation } from "react-i18next";

import { RadialBarChart, RadialBar, PolarAngleAxis } from "recharts";

import type { UtilizationData } from "@/hooks/use-monitor-charts";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface Props {
  data: UtilizationData;
  isLoading: boolean;
}

export default function WorkerUtilizationGauge({ data, isLoading }: Props) {
  const { t } = useTranslation();

  const chartData = [{ value: data.percent ?? 0, fill: "url(#gaugeGradient)" }];
  const percentColor =
    data.percent === null || !data.thresholdsAvailable
      ? "text-slate-400 dark:text-slate-500"
      : data.percent >= (data.criticalAtPercent ?? Number.POSITIVE_INFINITY)
        ? "text-rose-500"
        : data.percent >= (data.warningAtPercent ?? Number.POSITIVE_INFINITY)
          ? "text-amber-500"
          : "text-emerald-500";

  return (
    <div className="bg-white/70 dark:bg-slate-900/70 backdrop-blur-xl border border-slate-200/60 dark:border-slate-700/60 rounded-[2rem] p-6 shadow-lg dark:shadow-black/20 flex flex-col items-center gap-4 relative overflow-hidden group">
      {/* Decorative ring glow */}
      <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-40 h-40 rounded-full bg-gradient-to-br from-emerald-400/20 to-teal-400/10 blur-2xl" />
      </div>

      <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] w-full text-center" variant="h5">
        {t("monitor.charts.utilization")}
      </Typography>

      {isLoading ? (
        <div className="w-[160px] h-[160px] flex items-center justify-center">
          <div className="w-20 h-20 rounded-full border-4 border-slate-100 dark:border-slate-700 border-t-emerald-400 animate-spin" />
        </div>
      ) : data.unavailable ? (
        <div className="w-[160px] h-[160px] flex flex-col items-center justify-center gap-2 text-center" role="status">
          <span className="text-4xl font-black tracking-tighter text-slate-400 dark:text-slate-500" aria-hidden="true">—</span>
          <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
            {t("monitor.metric_unavailable", { defaultValue: "Metric unavailable" })}
          </Typography>
        </div>
      ) : (
        <div className="relative">
          <RadialBarChart
            barSize={12}
            data={chartData}
            endAngle={-270}
            height={160}
            innerRadius={60}
            outerRadius={75}
            startAngle={90}
            width={160}
          >
            <defs>
              <linearGradient id="gaugeGradient" x1="0" x2="1" y1="0" y2="1">
                <stop offset="0%" stopColor="#10b981" />
                <stop offset="100%" stopColor="#14b8a6" />
              </linearGradient>
            </defs>
            <PolarAngleAxis
              angleAxisId={0}
              domain={[0, 100]}
              tick={false}
              type="number"
            />
            <RadialBar
              background={{ fill: "rgba(148,163,184,0.12)" }}
              cornerRadius={8}
              dataKey="value"
            />
          </RadialBarChart>

          {/* Center overlay */}
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <span
              className={cn(
                "text-3xl font-black tracking-tighter tabular-nums transition-colors duration-500",
                percentColor
              )}
            >
              {data.percent === null ? "—" : `${data.percent}%`}
            </span>
            <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest">
              {t("monitor.charts.utilization")}
            </span>
          </div>
        </div>
      )}

      {/* Active / Idle pills */}
      <div className="flex items-center gap-3">
        <span className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-emerald-50 dark:bg-emerald-950/60 text-emerald-600 dark:text-emerald-300 text-[10px] font-bold uppercase tracking-wider border border-emerald-100 dark:border-emerald-800/50">
          <Zap size={10} />
          {data.active ?? "—"} {t("monitor.active_short", { defaultValue: "active" })}
        </span>
        <span className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-slate-50 dark:bg-slate-800/60 text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-wider border border-slate-100 dark:border-slate-700/50">
          <Coffee size={10} />
          {data.idle ?? "—"} {t("monitor.idle_short", { defaultValue: "idle" })}
        </span>
      </div>

      {/* Scaling hints */}
      {!isLoading && data.percent !== null && data.thresholdsAvailable && data.percent >= (data.criticalAtPercent ?? Number.POSITIVE_INFINITY) && (
        <Typography className="text-[10px] text-rose-500 dark:text-rose-400 font-medium text-center" variant="p">
          {t("monitor.charts.scale_up_warning")}
        </Typography>
      )}
      {!isLoading && data.percent !== null && data.thresholdsAvailable && data.percent < (data.scaleDownBelowPercent ?? 0) && data.percent > 0 && (
        <Typography className="text-[10px] text-sky-500 dark:text-sky-400 font-medium text-center" variant="p">
          {t("monitor.charts.scale_down_hint")}
        </Typography>
      )}
    </div>
  );
}
