import { Search } from "lucide-react";
import React from "react";

import SearchBox from "@/components/common/search-box";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { Shift } from "@/lib/api";
import { cn } from "@/lib/utils";

import {
  useShiftPicker,
  FILTER_CHIPS,
  getShiftBadge,
} from "./hooks/use-shift-picker";
import { ShiftTimeBadge } from "./shift-time-badge";

// ── ShiftButton ─────────────────────────────────────────────────────

function ShiftButton({
  shift,
  onClick,
  formatTime,
  t,
}: {
  formatTime: (v: string) => string;
  onClick: () => void;
  shift: Shift;
  t: (key: string) => string;
}) {
  const badge = getShiftBadge(shift, t);

  return (
    <Button
      className={cn(
        "p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group w-full",
        "hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-950/30",
        "border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30"
      )}
      onClick={onClick}
      variant="ghost"
    >
      {/* Color strip */}
      <div className={cn("absolute left-0 top-0 bottom-0 w-1.5 rounded-l-full", badge.dot)} />

      <div className="flex items-center justify-between gap-3">
        <div className="min-w-0">
          <div className="font-bold text-sm uppercase tracking-tight text-slate-700 dark:text-slate-200">{shift.name}</div>
          {!!shift.report_symbol && (
            <span className="text-[9px] font-mono font-bold text-slate-400 dark:text-slate-500 bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded ml-0">
              [{shift.report_symbol}]
            </span>
          )}
          <div className="flex flex-wrap gap-x-3 gap-y-1 mt-1.5">
            <ShiftTimeBadge formatTime={formatTime} shift={shift} t={t} />
          </div>
        </div>

        {/* Badge pill */}
        {!!badge.label && (
          <span className={cn(
            "shrink-0 px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-wider",
            badge.bg,
            badge.dot.replace("bg-", "text-").replace("-500", "-600").replace("-400", "-500") + " dark:" + badge.dot.replace("bg-", "text-").replace("-500", "-400").replace("-400", "-300")
          )}>
            {badge.label}
          </span>
        )}
      </div>
    </Button>
  );
}

// ── Props ────────────────────────────────────────────────────────────

interface ShiftPickerModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectShift: (shiftId: string | null) => void;
  shifts?: Shift[];
  targetDates: string[];
}

// ── Main Component ───────────────────────────────────────────────────

export function ShiftPickerModal({
  isOpen, onClose, targetDates, shifts, onSelectShift,
}: ShiftPickerModalProps) {
  const picker = useShiftPicker({ isOpen, onSelectShift, onClose, shifts });
  const {
    searchQuery, setSearchQuery,
    activeChip, setActiveChip,
    selectedLocationId, setSelectedLocationId,
    selectedEmployerId, setSelectedEmployerId,
    searchInputRef,
    formatTime, t,
    locationIds, employerIds,
    filtered,
    hasFrequent,
    sortedFrequentShifts,
    nonFrequentFiltered,
    showFrequentChip,
    handleSelect,
    handleClearSearch,
    handleKeyDown,
  } = picker;

  return (
    <Modal
      backdropClassName="bg-slate-900/5 dark:bg-slate-950/10 backdrop-blur-none"
      isOpen={isOpen}
      maxWidth="xl"
      onClose={onClose}
      subtitle={t("shifts.modal.select_shift_subtitle", { count: targetDates.length })}
      title={t("shifts.modal.select_shift_title")}
    >
      <div className="space-y-4 pt-2">
        {/* Tip for single-day selection */}
        {targetDates.length === 1 && (
          <Typography className="text-[10px] text-slate-400 dark:text-slate-500 italic text-center" variant="p">
            {t("work_segments.segments", "Tip: Assign multiple shifts to the same day for split-shift scheduling.")}
          </Typography>
        )}

        {/* ── Search Input ─────────────────────────────────────────── */}
        <SearchBox
          inputRef={searchInputRef}
          onChange={setSearchQuery}
          onKeyDown={handleKeyDown}
          placeholder={t("shifts.picker.search_placeholder")}
          value={searchQuery}
        />

        {/* ── Filter Chips ─────────────────────────────────────────── */}
        <div className="flex gap-2 overflow-x-auto pb-1 -mx-1 px-1 scrollbar-hide">
          {FILTER_CHIPS.filter((chip) => chip.key !== "frequent" || showFrequentChip).map((chip) => (
            <Button
              aria-pressed={activeChip === chip.key}
              className={cn(
                "shrink-0 px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider transition-all duration-200 h-auto",
                "border-2 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/50",
                activeChip === chip.key
                  ? "bg-primary-600 text-white border-primary-600 shadow-lg shadow-primary-200 dark:shadow-primary-900/30"
                  : "bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
              )}
              key={chip.key}
              onClick={() => { setActiveChip(chip.key); }}
              type="button"
              variant="ghost"
            >
              {t(chip.labelKey)}
            </Button>
          ))}
        </div>

        {/* ── Location / Employer Scope Filters ─────────────────────── */}
        {(locationIds.length > 0 || employerIds.length > 0) && (
          <div className="flex gap-2 flex-wrap">
            {locationIds.length > 0 && (
              <select
                aria-label={t("shifts.picker.filter_location")}
                className="px-3 py-2 rounded-xl text-xs font-bold uppercase tracking-wider
                  bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-400
                  border-2 border-slate-200 dark:border-slate-700
                  focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/50
                  hover:border-primary-300 dark:hover:border-primary-700
                  transition-all duration-200 cursor-pointer"
                onChange={(e) => { setSelectedLocationId(e.target.value); }}
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
                className="px-3 py-2 rounded-xl text-xs font-bold uppercase tracking-wider
                  bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-400
                  border-2 border-slate-200 dark:border-slate-700
                  focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/50
                  hover:border-primary-300 dark:hover:border-primary-700
                  transition-all duration-200 cursor-pointer"
                onChange={(e) => { setSelectedEmployerId(e.target.value); }}
                value={selectedEmployerId}
              >
                <option value="">{t("shifts.picker.filter_all_employers")}</option>
                {employerIds.map((id) => (
                  <option key={id} value={id}>{id}</option>
                ))}
              </select>
            )}
          </div>
        )}

        {/* ── Shift List ───────────────────────────────────────────── */}
        {filtered.length === 0 ? (
          /* Empty search state */
          <div className="py-16 text-center animate-in fade-in duration-300">
            <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center">
              <Search className="h-8 w-8 text-slate-300 dark:text-slate-600" />
            </div>
            <Typography className="text-sm font-bold text-slate-500 dark:text-slate-400 uppercase tracking-tight" variant="h3">
              {t("shifts.picker.no_results_title")}
            </Typography>
            <Typography className="text-[10px] text-slate-400 dark:text-slate-500 mt-1" variant="p">
              {t("shifts.picker.no_results_desc")}
            </Typography>
            <Button
              className="mt-4 px-5 py-2 rounded-xl text-xs"
              onClick={handleClearSearch}
              variant="outline"
            >
              {t("shifts.picker.clear_search")}
            </Button>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-3 max-h-[50vh] overflow-y-auto pr-1">
            {/* ── Frequently Used Section ──────────────────────────── */}
            {hasFrequent ? <>
                <div className="flex items-center gap-2 pt-1 pb-0.5">
                  <span className="text-[9px] font-bold uppercase tracking-[0.15em] text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/40 px-2.5 py-1 rounded-lg">
                    ★ {t("shifts.picker.frequent_section_title")}
                  </span>
                </div>
                {sortedFrequentShifts.map((shift) => (
                  <ShiftButton
                    formatTime={formatTime}
                    key={shift.id}
                    onClick={() => { handleSelect(shift.id); }}
                    shift={shift}
                    t={t}
                  />
                ))}
                <div className="border-t border-slate-100 dark:border-slate-800 pt-1" />
              </> : null}

            {/* ── Main Shift List ──────────────────────────────────── */}
            {nonFrequentFiltered.map((shift) => (
              <ShiftButton
                formatTime={formatTime}
                key={shift.id}
                onClick={() => { handleSelect(shift.id); }}
                shift={shift}
                t={t}
              />
            ))}
          </div>
        )}

        {/* ── OFF Day Button ───────────────────────────────────────── */}
        <Button
          className="p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group hover:border-slate-300 dark:hover:border-slate-600 hover:bg-slate-100 dark:hover:bg-slate-700 border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 border-dashed w-full"
          onClick={() => { handleSelect(null); }}
          variant="ghost"
        >
          <div className="font-bold text-sm uppercase tracking-tight text-slate-500 dark:text-slate-400">{t("shifts.off_day")}</div>
          <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold mt-1.5">
            {t("shifts.modal.mark_as_off")}
          </div>
        </Button>

        {/* ── Cancel ───────────────────────────────────────────────── */}
        <Button
          className="w-full px-6 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all"
          onClick={onClose}
          variant="ghost"
        >
          {t("common.cancel")}
        </Button>
      </div>
    </Modal>
  );
}
