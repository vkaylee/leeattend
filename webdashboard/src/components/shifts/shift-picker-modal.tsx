import { Clock, Info, Search } from "lucide-react";
import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { ShiftWithIntervals } from "@/pages/shifts/hooks/use-shifts-types";

import SearchBox from "@/components/common/search-box";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Shift } from "@/lib/api";
import { cn } from "@/lib/utils";

// ── Constants ────────────────────────────────────────────────────────

const FREQUENT_SHIFTS_KEY = "leeattend_frequent_shifts";
const FREQUENT_TOP_COUNT = 3;

// ── Types ────────────────────────────────────────────────────────────

type FilterChip = "all" | "frequent" | "night" | "single" | "split";

interface FrequentMap {
  [shiftId: string]: number;
}

// ── Helpers ──────────────────────────────────────────────────────────

function readFrequentShifts(): FrequentMap {
  try {
    const raw = localStorage.getItem(FREQUENT_SHIFTS_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null) return {};
    const obj = parsed as Record<string, unknown>;
    const result: FrequentMap = {};
    for (const [k, v] of Object.entries(obj)) {
      if (typeof k === "string" && typeof v === "number" && v > 0) {
        result[k] = v;
      }
    }
    return result;
  } catch {
    return {};
  }
}

function writeFrequentShift(shiftId: string): void {
  try {
    const map = readFrequentShifts();
    map[shiftId] = (map[shiftId] ?? 0) + 1;
    localStorage.setItem(FREQUENT_SHIFTS_KEY, JSON.stringify(map));
  } catch {
    // localStorage may be unavailable; silently ignore.
  }
}

function getTopFrequentIds(map: FrequentMap): string[] {
  return Object.entries(map)
    .sort(([, a], [, b]) => b - a)
    .slice(0, FREQUENT_TOP_COUNT)
    .map(([id]) => id);
}

/** Check whether any interval of a shift is a night shift. */
function isNightShift(shift: Shift): boolean {
  const typed = shift as ShiftWithIntervals;
  return typed.intervals?.some((iv) => iv.is_night_shift === true) ?? false;
}

/** Check whether a shift is a split/multi-block shift (> 1 interval). */
function isSplitShift(shift: Shift): boolean {
  const typed = shift as ShiftWithIntervals;
  return (typed.intervals?.length ?? 0) > 1;
}

/** Returns true when the query matches the shift name, report symbol, or any interval time. */
function shiftMatchesQuery(shift: Shift, query: string): boolean {
  const q = query.toLowerCase().trim();
  if (!q) return true;

  if (shift.name.toLowerCase().includes(q)) return true;
  if (shift.report_symbol?.toLowerCase().includes(q)) return true;

  const typed = shift as ShiftWithIntervals;
  const intervals = typed.intervals ?? [];
  for (const iv of intervals) {
    if (iv.start_time?.includes(q)) return true;
    if (iv.end_time?.includes(q)) return true;
  }

  return false;
}

// ── Color badge helpers ──────────────────────────────────────────────

interface ShiftBadgeStyle {
  bg: string;
  dot: string;
  label: string;
}

function getShiftBadge(shift: Shift, t: (key: string) => string): ShiftBadgeStyle {
  if (isNightShift(shift)) {
    return { bg: "bg-indigo-50 dark:bg-indigo-950/40", dot: "bg-indigo-500", label: t("shifts.picker.filter_night") };
  }
  if (isSplitShift(shift)) {
    return { bg: "bg-teal-50 dark:bg-teal-950/40", dot: "bg-teal-500", label: t("shifts.picker.filter_split") };
  }
  const typed = shift as ShiftWithIntervals;
  const firstStart = typed.intervals?.[0]?.start_time ?? "";
  // Morning shifts: start before 12:00
  if (firstStart < "12:00") {
    return { bg: "bg-amber-50 dark:bg-amber-950/40", dot: "bg-amber-500", label: "" };
  }
  // Evening/afternoon shifts
  return { bg: "bg-slate-100 dark:bg-slate-800", dot: "bg-slate-400", label: "" };
}

// ── ShiftTimeBadge (existing, moved outside for clarity) ─────────────

interface ShiftTimeProps {
  formatTime: (v: string) => string;
  shift: Shift;
  t: (key: string) => string;
}

function ShiftTimeBadge({ shift, formatTime, t }: ShiftTimeProps) {
  const typed = shift as ShiftWithIntervals;
  const iv = typed.intervals?.[0] as
    | { start_time?: string; end_time?: string; break_start?: string; break_end?: string; break_duration_minutes?: number }
    | undefined;
  const st = iv?.start_time ?? "";
  const et = iv?.end_time ?? "";
  const bs = iv?.break_start;
  const be = iv?.break_end;
  const bd = iv?.break_duration_minutes ?? 0;

  return (
    <>
      <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold flex items-center gap-1">
        <Clock size={10} /> {st ? formatTime(st) : "—"} - {et ? formatTime(et) : "—"}
      </div>
      <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold flex items-center gap-1">
        <Info size={10} />
        {bs ? (
          <span>{bs.substring(0, 5)}-{be?.substring(0, 5)}</span>
        ) : (
          <span>{t("common.break")} {bd}m</span>
        )}
      </div>
    </>
  );
}

// ── Filter chip definitions ──────────────────────────────────────────

interface ChipDef {
  key: FilterChip;
  labelKey: string;
}

const FILTER_CHIPS: ChipDef[] = [
  { key: "all", labelKey: "shifts.picker.filter_all" },
  { key: "frequent", labelKey: "shifts.picker.filter_frequent" },
  { key: "night", labelKey: "shifts.picker.filter_night" },
  { key: "single", labelKey: "shifts.picker.filter_single" },
  { key: "split", labelKey: "shifts.picker.filter_split" },
];

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
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  // ── State ────────────────────────────────────────────────────────
  const [searchQuery, setSearchQuery] = useState("");
  const [activeChip, setActiveChip] = useState<FilterChip>("all");
  const [frequentMap, setFrequentMap] = useState<FrequentMap>({});
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Reset state on open, and auto-focus search input
  useEffect(() => {
    if (isOpen) {
      setSearchQuery("");
      setActiveChip("all");
      setFrequentMap(readFrequentShifts());
      // Small delay for modal animation to finish before focusing
      const timer = setTimeout(() => {
        searchInputRef.current?.focus();
      }, 100);
      return () => { clearTimeout(timer); };
    }
  }, [isOpen]);

  // ── Frequent shift IDs ───────────────────────────────────────────
  const frequentIds = useMemo(() => getTopFrequentIds(frequentMap), [frequentMap]);

  // ── Derived: filtered + grouped shifts ───────────────────────────
  const { filtered, hasFrequent } = useMemo(() => {
    const all = shifts ?? [];

    // Step 1: apply search
    const searched = searchQuery.trim()
      ? all.filter((s) => shiftMatchesQuery(s, searchQuery))
      : all;

    // Step 2: apply filter chip
    let chipFiltered = searched;
    switch (activeChip) {
      case "night":
        chipFiltered = searched.filter((s) => isNightShift(s));
        break;
      case "single":
        chipFiltered = searched.filter((s) => !isSplitShift(s));
        break;
      case "split":
        chipFiltered = searched.filter((s) => isSplitShift(s));
        break;
      case "frequent":
        chipFiltered = searched.filter((s) => frequentIds.includes(s.id));
        break;
      default:
        // "all" — no additional filtering
        break;
    }

    // Step 3: separate frequent from rest (only when "all" chip is active and no search)
    const isDefault = activeChip === "all" && !searchQuery.trim();
    const frequent = isDefault ? chipFiltered.filter((s) => frequentIds.includes(s.id)) : [];
    const rest = isDefault ? chipFiltered.filter((s) => !frequentIds.includes(s.id)) : chipFiltered;
    const hasFreq = frequent.length > 0;

    return { filtered: chipFiltered, frequentShifts: frequent, restShifts: rest, hasFrequent: hasFreq };
  }, [shifts, searchQuery, activeChip, frequentIds]);

  // ── Handlers ─────────────────────────────────────────────────────
  const handleSelect = useCallback((shiftId: string | null) => {
    if (shiftId) {
      writeFrequentShift(shiftId);
    }
    onSelectShift(shiftId);
    onClose();
  }, [onSelectShift, onClose]);

  const handleClearSearch = useCallback(() => {
    setSearchQuery("");
    setActiveChip("all");
    searchInputRef.current?.focus();
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      if (searchQuery) {
        e.stopPropagation();
        handleClearSearch();
      }
      // Otherwise, Modal's Radix Dialog handles Escape to close
    }
  }, [searchQuery, handleClearSearch]);

  // ── Sorted frequent shifts (always computed, never conditional) ──
  const sortedFrequentShifts = useMemo(() => {
    const freq = frequentMap;
    return [...((shifts ?? []).filter((s) => frequentIds.includes(s.id)))]
      .sort((a, b) => (freq[b.id] ?? 0) - (freq[a.id] ?? 0));
  }, [shifts, frequentIds, frequentMap]);

  // ── Non-frequent filtered shifts ───────────────────────────────────
  const nonFrequentFiltered = useMemo(() => {
    if (!hasFrequent) return filtered as Shift[];
    return (filtered as Shift[]).filter((s) => !frequentIds.includes(s.id));
  }, [filtered, hasFrequent, frequentIds]);

  // Derive whether the frequent chip should show (only when there's data)
  const showFrequentChip = frequentIds.length > 0;

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
            <button
              aria-pressed={activeChip === chip.key}
              className={cn(
                "shrink-0 px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider transition-all duration-200",
                "border-2 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/50",
                activeChip === chip.key
                  ? "bg-primary-600 text-white border-primary-600 shadow-lg shadow-primary-200 dark:shadow-primary-900/30"
                  : "bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
              )}
              key={chip.key}
              onClick={() => { setActiveChip(chip.key); }}
              type="button"
            >
              {t(chip.labelKey)}
            </button>
          ))}
        </div>

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
            {hasFrequent && (
              <>
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
              </>
            )}

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

// ── Internal: Shift Button ───────────────────────────────────────────

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
          {shift.report_symbol && (
            <span className="text-[9px] font-mono font-bold text-slate-400 dark:text-slate-500 bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded ml-0">
              [{shift.report_symbol}]
            </span>
          )}
          <div className="flex flex-wrap gap-x-3 gap-y-1 mt-1.5">
            <ShiftTimeBadge formatTime={formatTime} shift={shift} t={t} />
          </div>
        </div>

        {/* Badge pill */}
        {badge.label && (
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
