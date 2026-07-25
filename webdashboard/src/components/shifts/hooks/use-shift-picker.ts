import type { TFunction } from "i18next";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { FrequentMap } from "@/lib/frequent-shifts-storage";
import type { ShiftCardMetrics } from "@/lib/shift-card-metrics";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Shift } from "@/lib/api";
import { readFrequentShifts, writeFrequentShift, getTopFrequentIds } from "@/lib/frequent-shifts-storage";
import { getShiftCardMetrics } from "@/lib/shift-card-metrics";

import type { FilterChip } from "./shift-picker-filters";

import { shiftMatchesQuery } from "../shift-utils";
import { applyChipFilter, applyScopeFilters, isDefaultView, uniqueIds } from "./shift-picker-filters";
import { NO_ACTIVE_INDEX, useRovingCursor } from "./use-roving-cursor";

// Badge tokens and filter definitions live in their own modules now; re-exported
// so existing importers of this hook keep working.
export type { ShiftAccent, ShiftBadgeStyle } from "../shift-badge-style";
export { getShiftAccent, getShiftBadge } from "../shift-badge-style";
export type { ChipDef, FilterChip } from "./shift-picker-filters";
export { FILTER_CHIPS } from "./shift-picker-filters";

// ── Hook ─────────────────────────────────────────────────────────────

interface UseShiftPickerOptions {
  isOpen: boolean;
  onClose: () => void;
  onSelectShift: (shiftId: string | null) => void;
  shifts?: Shift[];
}

interface UseShiftPickerReturn {
  activeChip: FilterChip;
  /** Index into {@link orderedShifts} of the arrow-key cursor; -1 when unset. */
  activeIndex: number;
  employerIds: string[];
  filtered: Shift[];
  formatTime: (v: string) => string;
  /** Aggregate metrics per shift id, memoised across renders. */
  getMetrics: (shift: Shift) => ShiftCardMetrics;
  handleClearSearch: () => void;
  handleKeyDown: (e: React.KeyboardEvent) => void;
  handleSelect: (shiftId: string | null) => void;
  hasFrequent: boolean;
  locationIds: string[];
  nonFrequentFiltered: Shift[];
  /** Frequent section followed by the rest — the visual order on screen. */
  orderedShifts: Shift[];
  registerItemRef: (index: number) => (el: HTMLButtonElement | null) => void;
  searchInputRef: React.RefObject<HTMLInputElement | null>;
  searchQuery: string;
  selectedEmployerId: string;
  selectedLocationId: string;
  setActiveChip: (v: FilterChip) => void;
  setActiveIndex: (v: number) => void;
  setSearchQuery: (v: string) => void;
  setSelectedEmployerId: (v: string) => void;
  setSelectedLocationId: (v: string) => void;
  showFrequentChip: boolean;
  sortedFrequentShifts: Shift[];
  t: TFunction;
  /** How many times each shift has been picked on this device. */
  usageCounts: FrequentMap;
}

export function useShiftPicker({
  isOpen,
  onSelectShift,
  onClose,
  shifts,
}: UseShiftPickerOptions): UseShiftPickerReturn {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  // ── State ────────────────────────────────────────────────────────
  const [searchQuery, setSearchQuery] = useState("");
  const [activeChip, setActiveChip] = useState<FilterChip>("all");
  const [frequentMap, setFrequentMap] = useState<FrequentMap>({});
  const [selectedLocationId, setSelectedLocationId] = useState<string>("");
  const [selectedEmployerId, setSelectedEmployerId] = useState<string>("");
  const searchInputRef = useRef<HTMLInputElement>(null);

  const resetFilters = useCallback(() => {
    setSearchQuery("");
    setActiveChip("all");
    setSelectedLocationId("");
    setSelectedEmployerId("");
  }, []);

  // Reset state on open, and auto-focus search input
  useEffect(() => {
    if (isOpen) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- intentional form reset when picker opens; all state updates batch in React 18+
      resetFilters();
      setFrequentMap(readFrequentShifts());
      const timer = setTimeout(() => {
        searchInputRef.current?.focus();
      }, 100);
      return () => { clearTimeout(timer); };
    }
  }, [isOpen, resetFilters]);

  // ── Frequent shift IDs ───────────────────────────────────────────
  const frequentIds = useMemo(() => getTopFrequentIds(frequentMap), [frequentMap]);

  // ── Unique location/employer IDs across all shifts ──────────────
  const locationIds = useMemo(() => uniqueIds(shifts ?? [], "location_id"), [shifts]);
  const employerIds = useMemo(() => uniqueIds(shifts ?? [], "employer_id"), [shifts]);

  // ── Derived: filtered + grouped shifts ───────────────────────────
  const { filtered, hasFrequent } = useMemo(() => {
    const pool = shifts ?? [];

    // Step 1: free-text search, Step 2: type chip, Step 3: location/employer scope.
    const searched = searchQuery.trim()
      ? pool.filter((s) => shiftMatchesQuery(s, searchQuery))
      : pool;
    const chipFiltered = applyChipFilter(searched, activeChip, frequentIds);
    const scopeFiltered = applyScopeFilters(chipFiltered, selectedLocationId, selectedEmployerId);

    // The frequent section only appears in the unfiltered default view,
    // otherwise it would duplicate rows the user explicitly filtered for.
    const isDefault = isDefaultView(activeChip, searchQuery, selectedLocationId, selectedEmployerId);
    return {
      filtered: scopeFiltered,
      hasFrequent: isDefault && scopeFiltered.some((s) => frequentIds.includes(s.id)),
    };
  }, [shifts, searchQuery, activeChip, frequentIds, selectedLocationId, selectedEmployerId]);

  // ── Sorted frequent shifts (most used first) ─────────────────────
  const sortedFrequentShifts = useMemo(() => {
    if (!hasFrequent) return [];
    return filtered
      .filter((s) => frequentIds.includes(s.id))
      .sort((a, b) => (frequentMap[b.id] ?? 0) - (frequentMap[a.id] ?? 0));
  }, [filtered, hasFrequent, frequentIds, frequentMap]);

  const nonFrequentFiltered = useMemo(
    () => (hasFrequent ? filtered.filter((s) => !frequentIds.includes(s.id)) : filtered),
    [filtered, hasFrequent, frequentIds],
  );

  // ── Flat visual order, used to map the arrow-key cursor to a card ──
  const orderedShifts = useMemo(
    () => [...sortedFrequentShifts, ...nonFrequentFiltered],
    [sortedFrequentShifts, nonFrequentFiltered],
  );

  // ── Per-shift metrics cache ──────────────────────────────────────
  const metricsCache = useMemo(() => {
    const cache = new Map<string, ShiftCardMetrics>();
    for (const shift of shifts ?? []) {
      cache.set(shift.id, getShiftCardMetrics(shift));
    }
    return cache;
  }, [shifts]);

  const getMetrics = useCallback(
    (shift: Shift): ShiftCardMetrics => metricsCache.get(shift.id) ?? getShiftCardMetrics(shift),
    [metricsCache],
  );

  const cursor = useRovingCursor(
    orderedShifts.length,
    orderedShifts.map((s) => s.id).join("|"),
  );

  // ── Handlers ─────────────────────────────────────────────────────
  const handleSelect = useCallback((shiftId: string | null) => {
    if (shiftId) {
      writeFrequentShift(shiftId);
    }
    onSelectShift(shiftId);
    onClose();
  }, [onSelectShift, onClose]);

  const handleClearSearch = useCallback(() => {
    resetFilters();
    cursor.reset();
    searchInputRef.current?.focus();
  }, [resetFilters, cursor]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        cursor.moveCursor(1);
        return;
      case "ArrowUp":
        e.preventDefault();
        cursor.moveCursor(-1);
        return;
      case "Enter": {
        // Enter with no cursor picks the top result — the fast path after typing.
        const index = cursor.activeIndex === NO_ACTIVE_INDEX ? 0 : cursor.activeIndex;
        const target = orderedShifts.at(index);
        if (target) {
          e.preventDefault();
          handleSelect(target.id);
        }
        return;
      }
      case "Escape":
        if (searchQuery) {
          e.stopPropagation();
          handleClearSearch();
        }
        return;
      default:
        return;
    }
  }, [cursor, orderedShifts, handleSelect, searchQuery, handleClearSearch]);

  return {
    activeChip,
    activeIndex: cursor.activeIndex,
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
    registerItemRef: cursor.registerItemRef,
    searchInputRef,
    searchQuery,
    selectedEmployerId,
    selectedLocationId,
    setActiveChip,
    setActiveIndex: cursor.setActiveIndex,
    setSearchQuery,
    setSelectedEmployerId,
    setSelectedLocationId,
    showFrequentChip: frequentIds.length > 0,
    sortedFrequentShifts,
    t,
    usageCounts: frequentMap,
  };
}
