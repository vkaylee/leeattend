import { CalendarDays, GitCompareArrows } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import FilterPanel from "@/components/ui/filter-panel";
import FilterTrigger from "@/components/ui/filter-trigger";
import { useDepartments } from "@/hooks/use-departments";
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
import { useWorkspace } from "@/hooks/use-workspace";

const periodLabels: Record<(typeof dashboardPeriods)[number], string> = {
  today: "Today",
  this_week: "This week",
  this_month: "This month",
  this_quarter: "This quarter",
  year_to_date: "Year to date",
  custom: "Custom range",
};

const comparisonLabels: Record<(typeof dashboardComparisons)[number], string> = {
  none: "No comparison",
  previous_period: "Previous period",
  same_period_last_year: "Same period last year",
  target: "Target",
  budget: "Budget",
  benchmark: "Benchmark",
};

function initialFilters(): DashboardMetricsFilters {
  if (typeof window === "undefined") return canonicalizeDashboardMetricsFilters({});
  return parseDashboardMetricsFilters(window.location.search);
}

interface DashboardFilterBarProps {
  filters: DashboardMetricsFilters;
  onChange: (filters: DashboardMetricsFilters) => void;
}

export function DashboardFilterBar({ filters, onChange }: DashboardFilterBarProps) {
  const { workspace } = useWorkspace();
  const { data: departments = [] } = useDepartments(workspace?.id, { enabled: !!workspace });
  const [draftFilters, setDraftFilters] = useState(initialFilters);

  useEffect(() => {
    setDraftFilters(filters);
  }, [filters]);

  const departmentOptions = useMemo(
    () => departments.map((department) => ({ label: department.name, value: department.id })),
    [departments],
  );
  const selectedCount = filters.departmentIds.length;
  const updateUrl = (next: DashboardMetricsFilters) => {
    if (typeof window !== "undefined") {
      const query = serializeDashboardMetricsFilters(next);
      const url = query ? `${window.location.pathname}?${query}` : window.location.pathname;
      window.history.replaceState(null, "", url);
    }
    onChange(next);
  };

  return (
    <section aria-label="Dashboard filters" className="flex flex-col gap-3 rounded-2xl border border-slate-200 bg-white/80 p-3 shadow-sm dark:border-slate-800 dark:bg-slate-900/80 sm:flex-row sm:flex-wrap sm:items-center">
      <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
        <FilterPanel
          container="popover"
          headerLabel="Reporting period"
          mode="single"
          onClear={() => updateUrl(canonicalizeDashboardMetricsFilters({ ...filters, period: "this_month" }))}
          onToggle={(period) => {
            const parsed = parseDashboardPeriod(period);
            if (parsed) updateUrl(canonicalizeDashboardMetricsFilters({ ...filters, period: parsed }));
          }}
          options={dashboardPeriods.map((period) => ({ label: periodLabels[period], value: period }))}
          selected={[filters.period]}
          trigger={<FilterTrigger chevron hasActive label={periodLabels[filters.period]} icon={<CalendarDays className="h-4 w-4" />} variant="pill-button" />}
        />

        <FilterPanel
          container="popover"
          headerLabel="Comparison"
          mode="single"
          onClear={() => updateUrl(canonicalizeDashboardMetricsFilters({ ...filters, comparison: "none" }))}
          onToggle={(comparison) => {
            const parsed = parseDashboardComparison(comparison);
            if (parsed) updateUrl(canonicalizeDashboardMetricsFilters({ ...filters, comparison: parsed }));
          }}
          options={dashboardComparisons.map((comparison) => ({ label: comparisonLabels[comparison], value: comparison }))}
          selected={[filters.comparison]}
          trigger={<FilterTrigger chevron hasActive={filters.comparison !== "none"} label={comparisonLabels[filters.comparison]} icon={<GitCompareArrows className="h-4 w-4" />} variant="pill-button" />}
        />

        <FilterPanel
          container="popover"
          headerLabel="Department"
          mode="multi"
          onClear={() => updateUrl({ ...filters, departmentIds: [] })}
          onToggle={(departmentId) => {
            const departmentIds = filters.departmentIds.includes(departmentId)
              ? filters.departmentIds.filter((id) => id !== departmentId)
              : [...filters.departmentIds, departmentId];
            updateUrl(canonicalizeDashboardMetricsFilters({ ...filters, departmentIds }));
          }}
          options={departmentOptions}
          searchable
          selected={filters.departmentIds}
          trigger={<FilterTrigger chevron hasActive={selectedCount > 0} activeCount={selectedCount} label="Department" variant="pill-button" />}
        />
      </div>
      <p className="text-xs text-slate-500 dark:text-slate-400" aria-live="polite">
        Dates use workspace timezone.
      </p>
    </section>
  );
}

export function useDashboardFilterState() {
  const [filters, setFilters] = useState<DashboardMetricsFilters>(initialFilters);
  return [filters, setFilters] as const;
}

export { initialFilters };
