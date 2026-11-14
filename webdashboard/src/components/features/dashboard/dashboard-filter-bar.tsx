import { CalendarDays, GitCompareArrows, RotateCcw } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import FilterPanel from "@/components/ui/filter-panel";
import FilterTrigger from "@/components/ui/filter-trigger";
import {
  canonicalizeDashboardMetricsFilters,
  dashboardComparisons,
  dashboardPeriods,
  parseDashboardComparison,
  parseDashboardMetricsFilters,
  parseDashboardPeriod,
  serializeDashboardMetricsFilters,
  type DashboardMetricsFilters,
} from "@/hooks/dashboard-metrics-filters";
import { useDepartments } from "@/hooks/use-departments";

const periodLabels: Record<(typeof dashboardPeriods)[number], string> = {
  today: "dashboard.filters.period.today",
  this_week: "dashboard.filters.period.this_week",
  this_month: "dashboard.filters.period.this_month",
  this_quarter: "dashboard.filters.period.this_quarter",
  year_to_date: "dashboard.filters.period.year_to_date",
  custom: "dashboard.filters.period.custom",
};

const comparisonLabels: Record<(typeof dashboardComparisons)[number], string> = {
  none: "dashboard.filters.comparison.none",
  previous_period: "dashboard.filters.comparison.previous_period",
  same_period_last_year: "dashboard.filters.comparison.same_period_last_year",
  target: "dashboard.filters.comparison.target",
  budget: "dashboard.filters.comparison.budget",
  benchmark: "dashboard.filters.comparison.benchmark",
};

function initialFilters(): DashboardMetricsFilters {
  if (typeof window === "undefined") return canonicalizeDashboardMetricsFilters({});
  return parseDashboardMetricsFilters(window.location.search);
}

interface DashboardFilterBarProps {
  filters: DashboardMetricsFilters;
  onChange: (filters: DashboardMetricsFilters) => void;
}

function customDateError(filters: DashboardMetricsFilters, message: string) {
  if (filters.period !== "custom") return null;
  if (!filters.startDate || !filters.endDate || filters.endDate < filters.startDate) return message;
  return null;
}

export function DashboardFilterBar({ filters, onChange }: DashboardFilterBarProps) {
  const { t } = useTranslation();
  const { data: departments = [] } = useDepartments();
  const [draftFilters, setDraftFilters] = useState(filters);

  useEffect(() => { setDraftFilters(filters); }, [filters]);

  const departmentOptions = useMemo(
    () => departments.map((department) => ({ label: department.name, value: department.id })),
    [departments],
  );
  const selectedCount = draftFilters.departmentIds.length;
  const dateError = customDateError(draftFilters, t("dashboard.filters.custom_date_error"));
  const hasChanges = serializeDashboardMetricsFilters(draftFilters) !== serializeDashboardMetricsFilters(filters);

  const updateDraft = (changes: Partial<DashboardMetricsFilters>) => {
    setDraftFilters((current) => ({ ...current, ...changes }));
  };

  const updateUrl = (next: DashboardMetricsFilters) => {
    if (typeof window !== "undefined") {
      const query = serializeDashboardMetricsFilters(next);
      const url = `${window.location.pathname}${query ? `?${query}` : ""}${window.location.hash}`;
      window.history.pushState(null, "", url);
    }
    onChange(next);
  };

  const apply = () => {
    if (!dateError) updateUrl(canonicalizeDashboardMetricsFilters(draftFilters));
  };

  const reset = () => {
    const next = canonicalizeDashboardMetricsFilters({});
    setDraftFilters(next);
    updateUrl(next);
  };

  return (
    <section
      aria-label={t("dashboard.filters.label")}
      className="flex flex-col gap-3 rounded-2xl border border-slate-200 bg-white/80 p-3 shadow-sm dark:border-slate-800 dark:bg-slate-900/80"
    >
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        <FilterPanel
          container="popover"
          headerLabel={t("dashboard.filters.period.label")}
          mode="single"
          onClear={() => { updateDraft({ period: "this_month", startDate: undefined, endDate: undefined }); }}
          onToggle={(value) => {
            const period = parseDashboardPeriod(value);
            if (period) updateDraft({ period, startDate: period === "custom" ? draftFilters.startDate : undefined, endDate: period === "custom" ? draftFilters.endDate : undefined });
          }}
          options={dashboardPeriods.map((period) => ({ label: t(periodLabels[period]), value: period }))}
          selected={[draftFilters.period]}
          trigger={<FilterTrigger chevron hasActive icon={<CalendarDays className="h-4 w-4" />} label={t(periodLabels[draftFilters.period])} variant="pill-button" />}
        />
        <FilterPanel
          container="popover"
          headerLabel={t("dashboard.filters.comparison.label")}
          mode="single"
          onClear={() => { updateDraft({ comparison: "none" }); }}
          onToggle={(value) => {
            const comparison = parseDashboardComparison(value);
            if (comparison) updateDraft({ comparison });
          }}
          options={dashboardComparisons.map((comparison) => ({ label: t(comparisonLabels[comparison]), value: comparison }))}
          selected={[draftFilters.comparison]}
          trigger={<FilterTrigger chevron hasActive={draftFilters.comparison !== "none"} icon={<GitCompareArrows className="h-4 w-4" />} label={t(comparisonLabels[draftFilters.comparison])} variant="pill-button" />}
        />
        <FilterPanel
          container="popover"
          headerLabel={t("dashboard.filters.department.label")}
          mode="multi"
          onClear={() => { updateDraft({ departmentIds: [] }); }}
          onToggle={(departmentId) => { updateDraft({ departmentIds: draftFilters.departmentIds.includes(departmentId) ? draftFilters.departmentIds.filter((id) => id !== departmentId) : [...draftFilters.departmentIds, departmentId] }); }}
          options={departmentOptions}
          searchable
          selected={draftFilters.departmentIds}
          trigger={<FilterTrigger activeCount={selectedCount} chevron hasActive={selectedCount > 0} label={t("dashboard.filters.department.label")} variant="pill-button" />}
        />
        {draftFilters.period === "custom" ? (
          <div aria-describedby="dashboard-custom-date-error" className="flex min-w-full flex-col gap-2 sm:min-w-0 sm:flex-row sm:items-end">
            <label className="flex min-w-0 flex-1 flex-col gap-1 text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="dashboard-start-date">
              {t("dashboard.filters.start_date")}
              <input aria-invalid={dateError ? "true" : "false"} className="h-10 rounded-xl border border-slate-200 bg-white px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-700 dark:bg-slate-900" id="dashboard-start-date" onChange={(event) => { updateDraft({ startDate: event.target.value || undefined }); }} type="date" value={draftFilters.startDate ?? ""} />
            </label>
            <label className="flex min-w-0 flex-1 flex-col gap-1 text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="dashboard-end-date">
              {t("dashboard.filters.end_date")}
              <input aria-invalid={dateError ? "true" : "false"} className="h-10 rounded-xl border border-slate-200 bg-white px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-700 dark:bg-slate-900" id="dashboard-end-date" min={draftFilters.startDate} onChange={(event) => { updateDraft({ endDate: event.target.value || undefined }); }} type="date" value={draftFilters.endDate ?? ""} />
            </label>
          </div>
        ) : null}
      </div>
      <div className="flex flex-col gap-2 border-t border-slate-100 pt-3 dark:border-slate-800 sm:flex-row sm:items-center sm:justify-between">
        <p aria-live="polite" className="text-xs text-slate-500 dark:text-slate-400">{t("dashboard.filters.timezone_hint")}</p>
        {dateError ? <p className="text-xs font-medium text-rose-600 dark:text-rose-400" id="dashboard-custom-date-error" role="alert">{dateError}</p> : null}
        <div className="flex justify-end gap-2">
          <Button aria-label={t("dashboard.filters.reset")} className="gap-1.5" disabled={!hasChanges} onClick={reset} type="button" variant="ghost"><RotateCcw className="h-3.5 w-3.5" />{t("dashboard.filters.reset")}</Button>
          <Button disabled={!hasChanges || !!dateError} onClick={apply} type="button" variant="primary">{t("dashboard.filters.apply")}</Button>
        </div>
      </div>
    </section>
  );
}

export function useDashboardFilterState() {
  const [filters, setFilters] = useState<DashboardMetricsFilters>(initialFilters);
  useEffect(() => {
    const handlePopState = () => { setFilters(initialFilters()); };
    window.addEventListener("popstate", handlePopState);
    return () => { window.removeEventListener("popstate", handlePopState); };
  }, []);
  return [filters, setFilters] as const;
}

export { initialFilters };
