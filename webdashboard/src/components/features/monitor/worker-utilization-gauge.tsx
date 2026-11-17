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

function isAtOrAboveThreshold(percent: number | null, threshold: number | null): boolean {
  return percent !== null && percent >= (threshold ?? Number.POSITIVE_INFINITY);
}

function getPercentColor(data: UtilizationData): string {
  if (data.percent === null || !data.thresholdsAvailable) {
    return "text-slate-400 dark:text-slate-500";
  }

  if (isAtOrAboveThreshold(data.percent, data.criticalAtPercent)) {
    return "text-rose-500";
  }

  if (isAtOrAboveThreshold(data.percent, data.warningAtPercent)) {
    return "text-amber-500";
  }

  return "text-emerald-500";
}

function LoadingState() {
  return (
    <div className="w-[160px] h-[160px] flex items-center justify-center">
      <div className="w-20 h-20 rounded-full border-4 border-slate-100 dark:border-slate-700 border-t-emerald-400 motion-safe:animate-spin" />
    </div>
  );
}

function UnavailableState() {
  const { t } = useTranslation();

  return (
    <div className="w-[160px] h-[160px] flex flex-col items-center justify-center gap-2 text-center" role="status">
      <span aria-hidden="true" className="text-4xl font-black tracking-tighter text-slate-400 dark:text-slate-500">—</span>
      <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
        {t("monitor.metric_unavailable", { defaultValue: "Metric unavailable" })}
      </Typography>
    </div>
  );
}

function UtilizationChart({ data }: { data: UtilizationData }) {
  const { t } = useTranslation();
  const chartData = [{ value: data.percent ?? 0, fill: "url(#gaugeGradient)" }];
  const percentColor = getPercentColor(data);

  return (
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

      <div className="absolute inset-0 flex flex-col items-center justify-center">
        <span
          className={cn(
            "text-3xl font-black tracking-tighter tabular-nums motion-safe:transition-colors duration-500",
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
  );
}

function GaugeContent({ data, isLoading }: Props) {
  if (isLoading) {
    return <LoadingState />;
  }

  if (data.unavailable) {
    return <UnavailableState />;
  }

  return <UtilizationChart data={data} />;
}

function UtilizationTable({ data, isLoading }: Props) {
  const { t } = useTranslation();
  const unavailableLabel = t("monitor.metric_unavailable", { defaultValue: "Metric unavailable" });

  return (
    <table className="sr-only">
      <caption>{t("monitor.utilization_table", { defaultValue: "Worker utilization summary" })}</caption>
      <tbody>
        <tr><th scope="row">{t("monitor.active_short", { defaultValue: "Active" })}</th><td>{isLoading || data.active === null ? unavailableLabel : data.active}</td></tr>
        <tr><th scope="row">{t("monitor.idle_short", { defaultValue: "Idle" })}</th><td>{isLoading || data.idle === null ? unavailableLabel : data.idle}</td></tr>
        <tr><th scope="row">{t("monitor.utilization", { defaultValue: "Worker utilization" })}</th><td>{isLoading || data.percent === null ? unavailableLabel : `${data.percent}%`}</td></tr>
      </tbody>
    </table>
  );
}

function UtilizationPills({ data }: { data: UtilizationData }) {
  const { t } = useTranslation();

  return (
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
  );
}

function ScaleUpHint() {
  const { t } = useTranslation();

  return (
    <Typography className="text-[10px] text-rose-500 dark:text-rose-400 font-medium text-center" variant="p">
      {t("monitor.charts.scale_up_warning")}
    </Typography>
  );
}

function ScaleDownHint() {
  const { t } = useTranslation();

  return (
    <Typography className="text-[10px] text-sky-500 dark:text-sky-400 font-medium text-center" variant="p">
      {t("monitor.charts.scale_down_hint")}
    </Typography>
  );
}

function ScalingHints({ data, isLoading }: Props) {
  const { percent } = data;

  if (isLoading || percent === null || !data.thresholdsAvailable) {
    return null;
  }

  const showScaleUpHint = isAtOrAboveThreshold(percent, data.criticalAtPercent);
  const showScaleDownHint = percent < (data.scaleDownBelowPercent ?? 0) && percent > 0;

  return (
    <>
      {showScaleUpHint ? <ScaleUpHint /> : null}
      {showScaleDownHint ? <ScaleDownHint /> : null}
    </>
  );
}

export default function WorkerUtilizationGauge({ data, isLoading }: Props) {
  const { t } = useTranslation();

  return (
    <div className="bg-white/70 dark:bg-slate-900/70 backdrop-blur-xl border border-slate-200/60 dark:border-slate-700/60 rounded-[2rem] p-6 shadow-lg dark:shadow-black/20 flex flex-col items-center gap-4 relative overflow-hidden group">
      <div className="absolute inset-0 opacity-0 group-hover:opacity-100 motion-safe:transition-opacity duration-700 motion-reduce:transition-none pointer-events-none">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-40 h-40 rounded-full bg-gradient-to-br from-emerald-400/20 to-teal-400/10 blur-2xl" />
      </div>

      <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] w-full text-center" variant="h5">
        {t("monitor.charts.utilization")}
      </Typography>

      <GaugeContent data={data} isLoading={isLoading} />
      <UtilizationTable data={data} isLoading={isLoading} />
      <UtilizationPills data={data} />
      <ScalingHints data={data} isLoading={isLoading} />
    </div>
  );
}
