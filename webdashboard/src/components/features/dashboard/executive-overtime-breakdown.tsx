import { ShieldAlert, Table2, TrendingUp } from "lucide-react";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { ChartSummary } from "@/components/features/dashboard/chart-summary";
import { ChartTableTwin, type ChartTableColumn } from "@/components/features/dashboard/chart-table-twin";
import { DashboardChartError } from "@/components/features/dashboard/dashboard-chart-state";
import { Typography } from "@/components/ui/typography";
import { useOvertimeBreakdown, type ChartRangeParams } from "@/hooks/use-dashboard";

interface Row {
  approved_ot_hours: number;
  cost_available: boolean;
  department_name: string;
  overtime_cost: string | null;
  system_ot_hours: number;
  unapproved_ot_hours: number;
}

function formatHours(value: number): string {
  return `${value.toFixed(1)}h`;
}

function formatCost(value: string | null, currency: string): string {
  if (value == null) return "—";
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return "—";
  return `${currency} ${parsed.toLocaleString(undefined, { maximumFractionDigits: 0 })}`.trim();
}

export function ExecutiveOvertimeBreakdown({ range, topN = 10, costStatus = "data_gap" }: { range: ChartRangeParams; topN?: number; costStatus?: "restricted" | "data_gap" | "available" }) {
  const { t } = useTranslation();
  const { data, isLoading, error } = useOvertimeBreakdown(range, topN);

  const rows: Row[] = useMemo(
    () =>
      (data?.rows ?? []).map((row) => ({
        department_name: row.department_name,
        system_ot_hours: row.system_ot_hours,
        approved_ot_hours: row.approved_ot_hours,
        unapproved_ot_hours: row.unapproved_ot_hours,
        overtime_cost: row.overtime_cost,
        cost_available: row.cost_available,
      })),
    [data],
  );

  const currency = data?.organization.currency ?? "USD";

  const columns: ChartTableColumn<Row>[] = [
    { header: t("dashboard.executive.breakdown.department"), key: "department_name" },
    { header: t("dashboard.executive.breakdown.system_ot"), key: "system_ot_hours", numeric: true, render: (r) => formatHours(r.system_ot_hours) },
    { header: t("dashboard.executive.breakdown.approved_ot"), key: "approved_ot_hours", numeric: true, render: (r) => formatHours(r.approved_ot_hours) },
    { header: t("dashboard.executive.breakdown.unapproved_ot"), key: "unapproved_ot_hours", numeric: true, render: (r) => formatHours(r.unapproved_ot_hours) },
    { header: t("dashboard.executive.breakdown.ot_cost"), key: "overtime_cost", numeric: true, render: (r) => {
        if (r.overtime_cost != null && r.cost_available) {
          return formatCost(r.overtime_cost, currency);
        }
        // The backend nulls cost for restricted callers AND when no paid/finalized
        // payslip exists in the period. The KPI grid's `overtime_cost` status tells
        // us which one it is — use that to pick the right tooltip.
        if (costStatus === "restricted") {
          return <span className="inline-flex items-center gap-1" title={t("dashboard.executive.breakdown.cost_restricted_tooltip")}><ShieldAlert aria-hidden="true" className="h-3.5 w-3.5" />{t("dashboard.executive.breakdown.not_available")}</span>;
        }
        return <span title={t("dashboard.executive.breakdown.cost_no_payroll_tooltip")}>{t("dashboard.executive.breakdown.not_available")}</span>;
      } },
  ];

  const summary = t("dashboard.executive.breakdown.summary", { count: rows.length });

  return (
    <section aria-labelledby="executive-overtime-breakdown-heading" className="mt-4 rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-950">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="bg-amber-50 dark:bg-amber-950 w-9 h-9 rounded-xl flex items-center justify-center text-amber-600 dark:text-amber-400 shadow-inner border border-amber-100 dark:border-amber-800">
            <TrendingUp size={18} />
          </div>
          <div>
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" id="executive-overtime-breakdown-heading" variant="h2">
              {t("dashboard.executive.breakdown.title")}
            </Typography>
            <Typography className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
              {currency}
            </Typography>
          </div>
        </div>
      </div>

      {isLoading ? (
        <div className="h-[160px] flex items-center justify-center">
          <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
        </div>
      ) : error ? (
        <DashboardChartError icon={<Table2 size={48} />} />
      ) : rows.length === 0 ? (
        <div className="h-[160px] flex flex-col items-center justify-center">
          <Table2 className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
          <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
            {t("common.no_data")}
          </Typography>
        </div>
      ) : (
        <>
          <ChartSummary text={summary} />
          <ChartTableTwin
            caption={summary}
            columns={columns}
            rows={rows}
            searchableKeys={["department_name"]}
          />
        </>
      )}
    </section>
  );
}

export default ExecutiveOvertimeBreakdown;
