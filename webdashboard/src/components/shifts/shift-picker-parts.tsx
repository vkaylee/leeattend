import type { TFunction } from "i18next";

import { CalendarOff, Search } from "lucide-react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

// ── Scope filter dropdowns ───────────────────────────────────────────

const SELECT_CLASS = cn(
  "px-3 py-2 rounded-xl text-xs font-bold uppercase tracking-wider",
  "bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-300",
  "border-2 border-slate-200 dark:border-slate-700",
  "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/50",
  "hover:border-primary-300 dark:hover:border-primary-700",
  "transition-all duration-200 cursor-pointer",
);

interface ScopeFiltersProps {
  employerIds: string[];
  locationIds: string[];
  onEmployerChange: (v: string) => void;
  onLocationChange: (v: string) => void;
  selectedEmployerId: string;
  selectedLocationId: string;
  t: TFunction;
}

export function ScopeFilters({
  employerIds,
  locationIds,
  onEmployerChange,
  onLocationChange,
  selectedEmployerId,
  selectedLocationId,
  t,
}: ScopeFiltersProps) {
  if (locationIds.length === 0 && employerIds.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-2">
      {locationIds.length > 0 && (
        <select
          aria-label={t("shifts.picker.filter_location")}
          className={SELECT_CLASS}
          onChange={(e) => { onLocationChange(e.target.value); }}
          value={selectedLocationId}
        >
          <option value="">{t("shifts.picker.filter_all_locations")}</option>
          {locationIds.map((id) => (
            <option key={id} value={id}>{id}</option>
          ))}
        </select>
      )}
      {employerIds.length > 0 && (
        <select
          aria-label={t("shifts.picker.filter_employer")}
          className={SELECT_CLASS}
          onChange={(e) => { onEmployerChange(e.target.value); }}
          value={selectedEmployerId}
        >
          <option value="">{t("shifts.picker.filter_all_employers")}</option>
          {employerIds.map((id) => (
            <option key={id} value={id}>{id}</option>
          ))}
        </select>
      )}
    </div>
  );
}

// ── Empty state ──────────────────────────────────────────────────────

export function PickerEmptyState({
  onClearSearch,
  t,
}: {
  onClearSearch: () => void;
  t: TFunction;
}) {
  return (
    <div className="animate-in fade-in py-16 text-center duration-300">
      <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-slate-100 dark:bg-slate-800">
        <Search aria-hidden="true" className="h-8 w-8 text-slate-400 dark:text-slate-500" />
      </div>
      <Typography
        className="text-sm font-bold tracking-tight text-slate-600 uppercase dark:text-slate-300"
        variant="h3"
      >
        {t("shifts.picker.no_results_title")}
      </Typography>
      <Typography className="mt-1 text-[11px] text-slate-500 dark:text-slate-400" variant="p">
        {t("shifts.picker.no_results_desc")}
      </Typography>
      <Button className="mt-4 rounded-xl px-5 py-2 text-xs" onClick={onClearSearch} variant="outline">
        {t("shifts.picker.clear_search")}
      </Button>
    </div>
  );
}

// ── OFF day option ───────────────────────────────────────────────────

/**
 * Marks the selected dates as a day off. Uses a native <button> rather than the
 * design-system Button because that component is a fixed-height, centred
 * inline-flex row and this option is a two-line card with a leading icon tile.
 */
export function OffDayOption({ onSelect, t }: { onSelect: () => void; t: TFunction }) {
  return (
    // eslint-disable-next-line react/forbid-elements -- multi-line card layout; see note above
    <button
      className={cn(
        "group w-full rounded-3xl border-2 border-dashed p-4 text-left",
        "transition-all duration-200 active:scale-[0.99]",
        "border-slate-200 bg-slate-50/60 hover:border-rose-300 hover:bg-rose-50/50",
        "dark:border-slate-700 dark:bg-slate-800/30 dark:hover:border-rose-800 dark:hover:bg-rose-950/20",
        "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2",
        "dark:focus-visible:ring-offset-slate-900",
      )}
      onClick={onSelect}
      type="button"
    >
      <span className="flex items-center gap-3">
        <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-2xl bg-slate-200/70 text-slate-500 transition-colors group-hover:bg-rose-100 group-hover:text-rose-600 dark:bg-slate-700/60 dark:text-slate-300 dark:group-hover:bg-rose-950/50 dark:group-hover:text-rose-400">
          <CalendarOff aria-hidden="true" size={16} />
        </span>
        <span className="min-w-0">
          <span className="block text-sm font-bold tracking-tight text-slate-600 uppercase dark:text-slate-300">
            {t("shifts.off_day")}
          </span>
          <span className="block text-[10px] font-bold text-slate-500 dark:text-slate-400">
            {t("shifts.modal.mark_as_off")}
          </span>
        </span>
      </span>
    </button>
  );
}
