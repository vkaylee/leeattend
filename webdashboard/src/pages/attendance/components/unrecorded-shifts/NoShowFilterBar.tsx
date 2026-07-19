import {
  AlertTriangle,
  Building2,
  Clock,
  X,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

import { ShiftTypePicker } from "../shift-type-picker";

interface NoShowFilterBarProps {
  selectedDate: string;
  today: string;
  isViewingToday: boolean;
  selectedDept: string;
  departments: string[];
  selectedShiftTypes: string[];
  onlyZoneB: boolean;
  onDateChange: (date: string) => void;
  onDeptChange: (dept: string) => void;
  onShiftTypesChange: (types: string[]) => void;
  onOnlyZoneBToggle: () => void;
  onClearFilters: () => void;
}

/**
 * Filter bar for the No-Show / Unrecorded Shifts view.
 * Includes date picker, department dropdown, shift-type picker,
 * Zone-B-only toggle, and a clear-filters button.
 */
export function NoShowFilterBar({
  selectedDate,
  today,
  isViewingToday,
  selectedDept,
  departments,
  selectedShiftTypes,
  onlyZoneB,
  onDateChange,
  onDeptChange,
  onShiftTypesChange,
  onOnlyZoneBToggle,
  onClearFilters,
}: NoShowFilterBarProps) {
  const { t } = useTranslation();

  return (
    <>
      {/* Date picker */}
      <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm p-3">
        <div className="flex items-center gap-3 flex-wrap">
          <label
            className="text-[10px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500 flex items-center gap-2"
            htmlFor="unrecorded-date-picker"
          >
            <Clock size={12} />
            {t("unrecorded_shifts.date_label")}
          </label>
          <input
            className="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2 font-bold text-sm text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500/30"
            id="unrecorded-date-picker"
            max={today}
            onChange={(e) => { onDateChange(e.target.value); }}
            type="date"
            value={selectedDate}
          />

          {isViewingToday && (
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800 text-emerald-700 dark:text-emerald-300 text-[10px] font-bold uppercase tracking-widest">
              <span className="relative flex size-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                <span className="relative inline-flex rounded-full size-2 bg-emerald-500" />
              </span>
              {t("unrecorded_shifts.live_badge")}
            </span>
          )}
        </div>
      </div>

      {/* Filters bar */}
      <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm p-3 flex flex-wrap items-center gap-3">
        {/* Department */}
        <div className="relative">
          <Building2
            className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none"
            size={14}
          />
          <select
            className="pl-8 pr-8 py-2 rounded-xl text-[11px] font-bold uppercase tracking-widest bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-600 dark:text-slate-300 appearance-none cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary-500/30"
            id="unrecorded-dept-filter"
            onChange={(e) => { onDeptChange(e.target.value); }}
            value={selectedDept}
          >
            <option value="__all__">{t("unrecorded_shifts.all_departments")}</option>
            {departments.map((dept) => (
              <option key={dept} value={dept}>{dept}</option>
            ))}
          </select>
        </div>

        <ShiftTypePicker onChange={onShiftTypesChange} selected={selectedShiftTypes} />

        {/* Only Zone B toggle */}
        <Button
          aria-pressed={onlyZoneB}
          className={cn(
            "px-3 py-2 gap-2 normal-case",
            onlyZoneB
              ? "bg-amber-50 dark:bg-amber-950/40 border-amber-300 dark:border-amber-700 text-amber-700 dark:text-amber-300 hover:bg-amber-50 dark:hover:bg-amber-950/40"
              : "text-slate-500 dark:text-slate-400 hover:border-amber-300 dark:hover:border-amber-700",
          )}
          id="unrecorded-only-zone-b"
          onClick={onOnlyZoneBToggle}
          variant="outline"
        >
          <AlertTriangle className="size-3.5" />
          {t("unrecorded_shifts.only_zone_b")}
        </Button>

        <Button
          className="px-2 py-2 gap-1.5 normal-case font-bold text-slate-400 dark:text-slate-500"
          onClick={onClearFilters}
          variant="ghost"
        >
          <X className="size-3.5" />
          {t("unrecorded_shifts.clear_filters")}
        </Button>

        {selectedDate < today && (
          <span className="text-[10px] text-slate-400 italic">
            {t("unrecorded_shifts.viewing_past", {
              defaultValue: "Historical — fixed records",
            })}
          </span>
        )}
      </div>
    </>
  );
}
