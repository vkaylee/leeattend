import { Wallet } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
    Bar,
    BarChart,
    CartesianGrid,
    Cell,
    Rectangle,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";

import type { DashboardMetric, DashboardMetricsResponse } from "@/hooks/dashboard-metric-contract";

import { ChartSummary } from "@/components/features/dashboard/chart-summary";
import { ChartTableTwin, type ChartTableColumn } from "@/components/features/dashboard/chart-table-twin";
import { MetricSelector } from "@/components/features/dashboard/metric-selector";
import { Typography } from "@/components/ui/typography";

type ViewKey = "hours" | "cost";

const SERIES_COLORS: Record<string, string> = {
  system_ot_hours: "#f59e0b",
  approved_ot_hours: "#10b981",
  unapproved_ot_hours: "#ef4444",
  overtime_cost: "#f59e0b",
  ot_cost_per_employee: "#10b981",
  labor_cost: "#3b82f6",
  payroll_actual: "#6366f1",
  payroll_forecast: "#94a3b8",
  budget_variance: "#ef4444",
};

/**
 * One axis type per view — never mix units (no dual-axis). The hours view plots
 * the three hour series; the approval rate is percent so it is reported as text.
 * The cost view plots every currency series on one currency axis.
 */
const HOUR_BARS = ["system_ot_hours", "approved_ot_hours", "unapproved_ot_hours"] as const;
const HOUR_TEXT_METRICS = ["ot_approval_rate"] as const;
const COST_BARS = [
  "overtime_cost",
  "ot_cost_per_employee",
  "labor_cost",
  "payroll_actual",
  "payroll_forecast",
  "budget_variance",
] as const;

interface BarRow {
  fill: string;
  key: string;
  label: string;
  unit: string;
  value: number;
}

function number(value: string | number | null): number | null {
  if (value === null) return null;
  const parsed = typeof value === "number" ? value : Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function formatValue(value: number | null, unit: string, currency: string): string {
  if (value === null) return "—";
  if (unit === "percent") return `${value.toFixed(1)}%`;
  if (unit === "hours") return `${value.toFixed(1)}h`;
  if (unit === "currency") return `${currency} ${value.toLocaleString(undefined, { maximumFractionDigits: 0 })}`.trim();
  return value.toLocaleString();
}

/** Metric is plottable only when it has a real value (not restricted/data_gap). */
function plottable(metric: DashboardMetric | undefined): DashboardMetric | undefined {
  return metric && metric.value !== null ? metric : undefined;
}

export function ExecutiveCostPanel({ data, currency }: { data?: DashboardMetricsResponse; currency: string }) {
  const { t } = useTranslation();
  const [view, setView] = useState<ViewKey>("hours");

  const metrics: Map<string, DashboardMetric> = useMemo(
    () => new Map((data?.metrics ?? []).map((metric) => [metric.key, metric])),
    [data],
  );

  const viewOptions: { label: string; value: ViewKey }[] = [
    { label: t("dashboard.executive.view_selector.hours"), value: "hours" },
    { label: t("dashboard.executive.view_selector.cost"), value: "cost" },
  ];

  const bars = view === "hours" ? HOUR_BARS : COST_BARS;
  const barRows: BarRow[] = useMemo(() => {
    const rows: BarRow[] = [];
    for (const key of bars) {
      const metric = plottable(metrics.get(key));
      if (!metric) continue;
      const value = number(metric.value);
      rows.push({
        fill: SERIES_COLORS[key] ?? "#6366f1",
        key,
        label: t(`dashboard.executive.metrics.${key}.label`),
        value: value ?? 0,
        unit: metric.unit,
      });
    }
    return rows;
  }, [bars, metrics, t]);

  // Text-reported percent metric (approval rate) — kept off the hour axis.
  const textMetrics: DashboardMetric[] = useMemo(() => {
    if (view !== "hours") return [];
    return HOUR_TEXT_METRICS
      .map((key) => metrics.get(key))
      .filter((metric): metric is DashboardMetric => metric !== undefined);
  }, [view, metrics]);

  const columns: ChartTableColumn<BarRow>[] = [
    { header: t("dashboard.executive.panel.metric"), key: "label" },
    { header: t("dashboard.executive.panel.value"), key: "value", numeric: true, render: (r) => formatValue(r.value, r.unit, currency) },
    { header: t("dashboard.executive.panel.unit"), key: "unit" },
  ];

  // Every series in a view shares one unit type (hours view = hours, cost view =
  // currency) — a single axis. Use the first plottable row's unit for tooltips.
  const viewUnit = barRows[0]?.unit ?? "";

  const summary = t("dashboard.executive.panel.summary", { view: t(`dashboard.executive.view_selector.${view}`) });

  return (
    <div className="mt-4 rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-950">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-3">
          <div className="bg-indigo-50 dark:bg-indigo-950 w-9 h-9 rounded-xl flex items-center justify-center text-indigo-600 dark:text-indigo-400 shadow-inner border border-indigo-100 dark:border-indigo-800">
            <Wallet size={18} />
          </div>
          <div>
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" variant="h2">
              {t("dashboard.executive.panel.title")}
            </Typography>
            <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
              {currency}
            </Typography>
          </div>
        </div>
        <MetricSelector
          aria-label={t("dashboard.executive.panel.view_selector")}
          onChange={setView}
          options={viewOptions}
          value={view}
        />
      </div>

      {barRows.length === 0 ? (
        <div className="h-[160px] flex flex-col items-center justify-center">
          <Wallet className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
          <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
            {t("common.no_data")}
          </Typography>
        </div>
      ) : (
        <>
          <ChartSummary text={summary} visible />
          {textMetrics.length > 0 ? (
            <div className="mb-3 flex flex-wrap gap-2">
              {textMetrics.map((m) => (
                <span
                  className="inline-flex items-center gap-1.5 rounded-full border border-indigo-200 bg-indigo-50 px-3 py-1 text-[11px] font-semibold text-indigo-700 dark:border-indigo-900 dark:bg-indigo-950/40 dark:text-indigo-300"
                  key={m.key}
                >
                  {t(`dashboard.executive.metrics.${m.key}.label`)}
                  <span className="font-mono">{m.value !== null ? `${Number(m.value).toFixed(1)}%` : "—"}</span>
                </span>
              ))}
            </div>
          ) : null}
          <div aria-label={summary} role="img">
            <ResponsiveContainer height={220} width="100%">
              <BarChart data={barRows} margin={{ top: 5, right: 10, left: -20, bottom: 5 }}>
                <CartesianGrid className="text-slate-100 dark:text-slate-800" stroke="currentColor" strokeDasharray="3 3" />
                <XAxis
                    axisLine={false}
                    className="text-slate-400 dark:text-slate-500"
                    dataKey="label"
                    interval="preserveStartEnd"
                    tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                    tickLine={false}
                />
                <YAxis
                    axisLine={false}
                    className="text-slate-400 dark:text-slate-500"
                    tick={{ fontSize: 9, fontWeight: 700, fill: "currentColor" }}
                    tickLine={false}
                />
                <Tooltip
                    contentStyle={{
                        borderRadius: "1rem",
                        border: "none",
                        boxShadow: "0 8px 30px rgba(0,0,0,.1)",
                        fontSize: "11px",
                        fontWeight: 600,
                    }}
                    formatter={(value: unknown, name) => [formatValue(Number(value ?? 0), viewUnit, currency), name]}
                />
                <Bar
                  dataKey="value"
                  name={barRows.map((r) => r.label).join(", ")}
                  radius={[6, 6, 0, 0]}
                  shape={(props) => <Rectangle {...props} fill={barRows[props.index ?? 0]?.fill} />}
                >
                  {barRows.map((row) => <Cell fill={row.fill} key={row.key} />)}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
          <ChartTableTwin
            caption={summary}
            columns={columns}
            rows={barRows}
            searchableKeys={["label"]}
          />
        </>
      )}
    </div>
  );
}

export default ExecutiveCostPanel;
