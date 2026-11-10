import { useMemo, useState } from "react";

import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

const MAX_RANGE_DAYS = 366;

interface DashboardDateRangeProps {
  className?: string;
  onChange: (range: { startDate: string; endDate: string }) => void;
  startDate: string;
  endDate: string;
  /** Organization timezone label (informational; the caller sends it as a param). */
  timezone?: string;
}

function toISODate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

/**
 * Custom range control: two native date inputs + strict validation.
 * Never silently falls back — an invalid window surfaces an inline message and
 * withholds the change until the caller's bounds are valid.
 */
export function DashboardDateRange({
  startDate,
  endDate,
  onChange,
  timezone,
  className,
}: DashboardDateRangeProps) {
  const { t } = useTranslation();
  const today = useMemo(() => toISODate(new Date()), []);

  const [start, setStart] = useState(startDate);
  const [end, setEnd] = useState(endDate);

  const error = useMemo(() => {
    if (!start || !end) return t("dashboard.filters.custom_date_error");
    if (end < start) return t("dashboard.filters.custom_date_error");
    if (end > today) return t("dashboard.filters.custom_date_error");
    const days = Math.round((new Date(end).getTime() - new Date(start).getTime()) / 86_400_000) + 1;
    if (days > MAX_RANGE_DAYS) return t("dashboard.filters.custom_date_error");
    return null;
  }, [start, end, today, t]);

  const commit = (nextStart: string, nextEnd: string) => {
    setStart(nextStart);
    setEnd(nextEnd);
    if (nextStart && nextEnd && nextEnd >= nextStart && nextEnd <= today) {
      onChange({ startDate: nextStart, endDate: nextEnd });
    }
  };

  return (
    <div className={cn("flex flex-wrap items-center gap-2", className)}>
      <input
        aria-label={t("dashboard.filters.start_date")}
        className="block rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 px-3 py-2 text-[10px] font-bold text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500/30"
        max={today}
        onChange={(e) => { commit(e.target.value, end); }}
        type="date"
        value={start}
      />
      <span className="text-[10px] font-bold text-slate-400">–</span>
      <input
        aria-label={t("dashboard.filters.end_date")}
        className="block rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 px-3 py-2 text-[10px] font-bold text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500/30"
        max={today}
        onChange={(e) => { commit(start, e.target.value); }}
        type="date"
        value={end}
      />

      {timezone ? (
        <span className="text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">
          {t("dashboard.filters.timezone_hint")}: {timezone}
        </span>
      ) : null}

      {error ? (
        <Typography
          aria-live="polite"
          className="w-full text-[10px] font-semibold text-rose-500 dark:text-rose-400"
          role="alert"
          variant="p"
        >
          {error}
        </Typography>
      ) : null}
    </div>
  );
}
