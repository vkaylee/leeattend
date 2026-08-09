import { Star } from "lucide-react";
import React from "react";

import SearchBox from "@/components/common/search-box";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { Shift } from "@/lib/api";
import { cn } from "@/lib/utils";

import { useShiftPicker, FILTER_CHIPS, getShiftBadge } from "./hooks/use-shift-picker";
import { ShiftPickerCard } from "./shift-picker-card";
import { OffDayOption, PickerEmptyState, ScopeFilters } from "./shift-picker-parts";

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
    activeChip, setActiveChip,
    activeIndex, setActiveIndex,
    employerIds,
    filtered,
    formatTime,
    getMetrics,
    handleClearSearch,
    handleKeyDown,
    handleSelect,
    hasFrequent,
    locationIds,
    nonFrequentFiltered,
    orderedShifts,
    registerItemRef,
    searchInputRef,
    searchQuery, setSearchQuery,
    selectedEmployerId, setSelectedEmployerId,
    selectedLocationId, setSelectedLocationId,
    showFrequentChip,
    sortedFrequentShifts,
    t,
    usageCounts,
  } = picker;

  /** Renders one card, wiring it to the shared roving-cursor index. */
  const renderCard = (shift: Shift) => {
    const index = orderedShifts.indexOf(shift);
    return (
      <ShiftPickerCard
        badge={getShiftBadge(shift, t)}
        formatTime={formatTime}
        isActive={index === activeIndex}
        key={shift.id}
        metrics={getMetrics(shift)}
        onClick={() => { handleSelect(shift.id); }}
        onMouseEnter={() => { setActiveIndex(index); }}
        ref={registerItemRef(index)}
        shift={shift}
        t={t}
        usageCount={usageCounts[shift.id]}
      />
    );
  };

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

        {/* Keyboard affordance — discoverable, not hidden in a tooltip */}
        {filtered.length > 0 && (
          <Typography
            className="text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500 text-center"
            variant="p"
          >
            {t("shifts.picker.keyboard_hint")}
          </Typography>
        )}

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
        <ScopeFilters
          employerIds={employerIds}
          locationIds={locationIds}
          onEmployerChange={setSelectedEmployerId}
          onLocationChange={setSelectedLocationId}
          selectedEmployerId={selectedEmployerId}
          selectedLocationId={selectedLocationId}
          t={t}
        />

        {/* ── Shift List ───────────────────────────────────────────── */}
        {filtered.length === 0 ? (
          <PickerEmptyState onClearSearch={handleClearSearch} t={t} />
        ) : (
          <div
            aria-label={t("shifts.modal.select_shift_title")}
            className="grid grid-cols-1 gap-3 max-h-[50dvh] overflow-y-auto pr-1"
            role="group"
          >
            {/* ── Frequently Used Section ──────────────────────────── */}
            {hasFrequent ? <>
                <div className="flex items-center gap-2 pt-1 pb-0.5">
                  <span className="inline-flex items-center gap-1 text-[9px] font-bold uppercase tracking-[0.15em] text-amber-700 dark:text-amber-300 bg-amber-50 dark:bg-amber-950/40 px-2.5 py-1 rounded-lg">
                    <Star aria-hidden="true" className="fill-current" size={10} />
                    {t("shifts.picker.frequent_section_title")}
                  </span>
                </div>
                {sortedFrequentShifts.map(renderCard)}
                <div className="border-t border-slate-100 dark:border-slate-800 pt-1" />
              </> : null}

            {/* ── Main Shift List ──────────────────────────────────── */}
            {nonFrequentFiltered.map(renderCard)}
          </div>
        )}

        {/* ── OFF Day Button ───────────────────────────────────────── */}
        <OffDayOption onSelect={() => { handleSelect(null); }} t={t} />

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
