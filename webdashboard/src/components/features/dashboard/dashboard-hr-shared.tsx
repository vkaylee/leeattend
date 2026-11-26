import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

/** Translation function accepting optional i18next interpolation options. */
export type TranslateFn = (key: string, options?: Record<string, string | number>) => string;

export interface PeriodOption {
  endDate: string;
  label: string;
  startDate: string;
  value: string;
}

export interface AnalyticsWarning {
  code: string;
  message: string;
}

export const HR_CHART_TOOLTIP_STYLE = {
  borderRadius: "12px",
  border: "none",
  boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
  fontSize: 12,
};

export function toISODate(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

export function getDefaultDateRange(days = 30): { startDate: string; endDate: string } {
  const now = new Date();
  const start = new Date(now);
  start.setDate(start.getDate() - (days - 1));
  return { startDate: toISODate(start), endDate: toISODate(now) };
}

export function getHrPeriodOptions(t: TranslateFn): PeriodOption[] {
  const today = new Date();
  const endDate = toISODate(today);

  const months3 = new Date(today.getFullYear(), today.getMonth() - 3, 1);
  const months6 = new Date(today.getFullYear(), today.getMonth() - 6, 1);

  const year1 = new Date(today);
  year1.setFullYear(year1.getFullYear() - 1);

  const year2 = new Date(today);
  year2.setFullYear(year2.getFullYear() - 2);

  return [
    { label: t("dashboard.hr.turnover_period_3m"), value: "3m", startDate: toISODate(months3), endDate },
    { label: t("dashboard.hr.turnover_period_6m"), value: "6m", startDate: toISODate(months6), endDate },
    { label: t("dashboard.hr.turnover_period_1y"), value: "1y", startDate: toISODate(year1), endDate },
    { label: t("dashboard.hr.turnover_period_2y"), value: "2y", startDate: toISODate(year2), endDate },
    { label: t("dashboard.hr.custom_range"), value: "custom", startDate: "", endDate: "" },
  ];
}

export function HrAnalyticsKpiCard({
  label,
  value,
  sub,
  accentClass,
}: {
  accentClass?: string;
  label: string;
  sub?: string;
  value: string;
}) {
  return (
    <div className="flex-1 min-w-[140px] rounded-2xl border border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-800/30 px-4 py-3">
      <Typography className="text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500" variant="p">
        {label}
      </Typography>
      <div className="mt-1 flex items-baseline gap-2 flex-wrap">
        <span className={cn("text-xl font-bold text-slate-900 dark:text-slate-100", accentClass)}>
          {value}
        </span>
        {sub ? (
          <span className="text-[10px] font-semibold text-slate-400 dark:text-slate-500">{sub}</span>
        ) : null}
      </div>
    </div>
  );
}

export function HrAnalyticsWarningStrip({ warnings, t }: { t: TranslateFn; warnings: AnalyticsWarning[] }) {
  if (warnings.length === 0) return null;

  return (
    <div aria-live="polite" className="mb-5 space-y-2">
      {warnings.map((w) => {
        const label = w.message.startsWith("dashboard.") ? t(w.message) : w.message;
        return (
          <div
            className="flex items-start gap-2 rounded-xl border border-amber-200 dark:border-amber-800 bg-amber-50 dark:bg-amber-950/40 px-4 py-2.5"
            key={w.code}
            role="note"
          >
            <span aria-hidden="true" className="text-amber-500">⚠</span>
            <span className="text-[11px] font-semibold text-amber-700 dark:text-amber-400">
              {label}
            </span>
          </div>
        );
      })}
    </div>
  );
}

export function HrAnalyticsTabs<T extends string>({
  tabs,
  activeTab,
  setActiveTab,
  t,
}: {
  activeTab: T;
  setActiveTab: (k: T) => void;
  t: TranslateFn;
  tabs: { key: T; labelKey: string }[];
}) {
  return (
    <div className="flex max-w-full overflow-x-auto scrollbar-hide bg-slate-100 dark:bg-slate-800/50 p-1 rounded-2xl w-fit mb-6 border border-slate-200/50 dark:border-slate-700/50">
      {tabs.map((tab) => (
        <Button
          className={cn(
            "px-5 py-2 rounded-xl text-xs font-bold uppercase tracking-widest transition-all h-auto shadow-none border border-transparent",
            activeTab === tab.key
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent",
          )}
          key={tab.key}
          onClick={() => { setActiveTab(tab.key); }}
          type="button"
          variant="ghost"
        >
          {t(tab.labelKey)}
        </Button>
      ))}
    </div>
  );
}
