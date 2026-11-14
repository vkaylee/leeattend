import { AlertTriangle, ArrowDownRight, ArrowUpRight, CheckCircle2, CircleHelp, Minus, RefreshCw, ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { DashboardMetric, DashboardMetricsResponse } from "@/hooks/dashboard-metric-contract";

import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

const METRIC_KEYS = [
  "headcount",
  "attendance_rate",
  "absence_rate",
  "turnover_rate",
  "system_ot_hours",
  "approved_ot_hours",
  "unapproved_ot_hours",
  "ot_approval_rate",
  "overtime",
  "ot_cost_per_employee",
  "labor_cost",
  "payroll_actual",
  "payroll_forecast",
  "budget_variance",
  "pending_hr_actions",
] as const;
type MetricKey = typeof METRIC_KEYS[number];

// The overtime slot renders `overtime_cost` only when it carries a real value
// (authorized + paid/finalized payroll backing it). When cost is data_gap or
// restricted (value null) we fall back to `overtime_hours` so the slot stays
// honest instead of showing a shield/triangle next to nothing.
function resolveOvertime(metrics: Map<string, DashboardMetric>): DashboardMetric | undefined {
  const cost = metrics.get("overtime_cost");
  if (cost && cost.value !== null) return cost;
  return metrics.get("overtime_hours");
}

function number(value: string | number | null) {
  if (value === null) return null;
  const parsed = typeof value === "number" ? value : Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function statusIcon(metric: DashboardMetric) {
  if (metric.status === "restricted") return <ShieldAlert aria-hidden="true" className="h-4 w-4" />;
  if (metric.status === "data_gap") return <AlertTriangle aria-hidden="true" className="h-4 w-4" />;
  if (metric.health === "ok") return <CheckCircle2 aria-hidden="true" className="h-4 w-4" />;
  return <AlertTriangle aria-hidden="true" className="h-4 w-4" />;
}

function deltaIcon(metric: DashboardMetric) {
  const delta = number(metric.comparison?.delta ?? null);
  if (delta === null || delta === 0) return <Minus aria-hidden="true" className="h-3.5 w-3.5" />;
  const positive = delta > 0;
  return positive
    ? <ArrowUpRight aria-hidden="true" className="h-3.5 w-3.5" />
    : <ArrowDownRight aria-hidden="true" className="h-3.5 w-3.5" />;
}

function valueText(metric: DashboardMetric, t: (key: string) => string, currency?: string) {
  const value = number(metric.value);
  if (value === null) return t(`dashboard.executive.status.${metric.status}`);
  if (metric.unit === "percent" || metric.unit === "rate") return `${value.toFixed(1)}%`;
  if (metric.unit === "hours") return `${value.toFixed(1)} ${t("dashboard.executive.units.hours")}`;
  if (metric.unit === "currency") return `${currency ?? ""} ${value.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 2 })}`.trim();
  return value.toLocaleString();
}

function deltaText(metric: DashboardMetric, t: (key: string) => string, currency?: string) {
  const delta = number(metric.comparison?.delta ?? null);
  if (delta === null) return t("dashboard.executive.comparison_unavailable");
  const sign = delta > 0 ? "+" : "";
  if (metric.unit === "percent" || metric.unit === "rate") return `${sign}${delta.toFixed(1)} pp`;
  if (metric.unit === "hours") return `${sign}${delta.toFixed(1)} ${t("dashboard.executive.units.hours")}`;
  if (metric.unit === "currency") return `${sign}${currency ?? ""} ${delta.toLocaleString()}`.trim();
  return `${sign}${delta.toLocaleString()}`;
}

function MetricCard({ metric, currency }: { metric: DashboardMetric; currency?: string }) {
  const { t } = useTranslation();
  const key = metric.key === "overtime_cost" || metric.key === "overtime_hours" ? "overtime" : metric.key;
  const numerator = number(metric.numerator?.value ?? null);
  const denominator = number(metric.denominator?.value ?? null);
  const tone = metric.health === "ok" ? "border-emerald-200 bg-emerald-50/60 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/20 dark:text-emerald-300" : metric.health === "critical" ? "border-rose-200 bg-rose-50/60 text-rose-700 dark:border-rose-900 dark:bg-rose-950/20 dark:text-rose-300" : "border-amber-200 bg-amber-50/60 text-amber-700 dark:border-amber-900 dark:bg-amber-950/20 dark:text-amber-300";
  return (
    <article className="group flex min-h-44 flex-col justify-between rounded-2xl border border-slate-200 bg-white p-4 shadow-sm transition duration-200 hover:-translate-y-0.5 hover:shadow-md dark:border-slate-800 dark:bg-slate-950">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h2 className="text-sm font-semibold text-slate-700 dark:text-slate-200">{t(`dashboard.executive.metrics.${key}.label`)}</h2>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t(`dashboard.executive.metrics.${key}.formula`)}</p>
        </div>
        <span className={cn("inline-flex items-center gap-1 rounded-full border px-2 py-1 text-[11px] font-semibold", tone)}>
          {statusIcon(metric)} {t(`dashboard.executive.status.${metric.status}`)}
        </span>
      </div>
      <div>
        <p className="mt-5 text-2xl font-bold tracking-tight text-slate-950 dark:text-white">{valueText(metric, t, currency)}</p>
        {numerator !== null && denominator !== null ? <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{numerator.toLocaleString()} / {denominator.toLocaleString()} {metric.numerator?.unit}</p> : null}
        <div className="mt-3 flex items-center justify-between gap-2 text-xs text-slate-500 dark:text-slate-400">
          <span className="inline-flex items-center gap-1" title={t("dashboard.executive.comparison_help")}>{deltaIcon(metric)} {deltaText(metric, t, currency)}</span>
          <span className="inline-flex items-center gap-1" title={t("dashboard.executive.freshness_help")}><CircleHelp aria-hidden="true" className="h-3.5 w-3.5" /> {new Date(metric.freshness.as_of).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</span>
        </div>
      </div>
    </article>
  );
}

export function ExecutiveKpiGrid({ data, loading, error, onRetry }: { data?: DashboardMetricsResponse; loading: boolean; error: boolean; onRetry: () => void }) {
  const { t } = useTranslation();
  if (loading) return <div aria-label={t("dashboard.executive.loading")} className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">{METRIC_KEYS.map((key) => <Skeleton className="h-44 rounded-2xl" key={key} />)}</div>;
  if (error) return <div aria-live="assertive" className="flex items-center justify-between gap-3 rounded-2xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700 dark:border-rose-900 dark:bg-rose-950/30 dark:text-rose-300" role="alert"><span>{t("dashboard.executive.error_description")}</span><Button aria-label={t("dashboard.executive.retry")} onClick={onRetry} type="button" variant="outline"><RefreshCw aria-hidden="true" className="mr-2 h-4 w-4" />{t("dashboard.executive.retry")}</Button></div>;
  const metrics = new Map((data?.metrics ?? []).map((metric) => [metric.key, metric]));
  return <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">{METRIC_KEYS.map((key) => { const metric = key === "overtime" ? resolveOvertime(metrics) : metrics.get(key); return metric ? <MetricCard currency={data?.organization.currency} key={key} metric={metric} /> : <div className="flex min-h-44 items-center justify-center rounded-2xl border border-dashed border-slate-300 p-4 text-sm text-slate-500 dark:border-slate-700" key={key}>{t("dashboard.executive.status.data_gap")}</div>; })}</div>;
}
