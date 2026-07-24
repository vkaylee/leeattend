import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";

import type { ShiftWithIntervals } from "@/pages/shifts/hooks/use-shifts-types";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Shift } from "@/lib/api";
import { readFrequentShifts, writeFrequentShift, getTopFrequentIds } from "@/lib/frequent-shifts-storage";
import type { FrequentMap } from "@/lib/frequent-shifts-storage";
import { isNightShift, isSplitShift, shiftMatchesQuery } from "../shift-utils";

// ── Types ────────────────────────────────────────────────────────────

export type FilterChip = "all" | "frequent" | "night" | "single" | "split";

export interface ShiftBadgeStyle {
  bg: string;
  dot: string;
  label: string;
}

export interface ChipDef {
  key: FilterChip;
  labelKey: string;
}

export const FILTER_CHIPS: ChipDef[] = [
  { key: "all", labelKey: "shifts.picker.filter_all" },
  { key: "frequent", labelKey: "shifts.picker.filter_frequent" },
  { key: "night", labelKey: "shifts.picker.filter_night" },
  { key: "single", labelKey: "shifts.picker.filter_single" },
  { key: "split", labelKey: "shifts.picker.filter_split" },
];

// ── Color badge helpers ──────────────────────────────────────────────

export function getShiftBadge(shift: Shift, t: (key: string) => string): ShiftBadgeStyle {
  if (isNightShift(shift)) {
    return { bg: "bg-indigo-50 dark:bg-indigo-950/40", dot: "bg-indigo-500", label: t("shifts.picker.filter_night") };
  }
  if (isSplitShift(shift)) {
    return { bg: "bg-teal-50 dark:bg-teal-950/40", dot: "bg-teal-500", label: t("shifts.picker.filter_split") };
  }
  const typed = shift as ShiftWithIntervals;
  const firstStart = typed.intervals?.[0]?.start_time ?? "";
  if (firstStart < "12:00") {
    return { bg: "bg-amber-50 dark:bg-amber-950/40", dot: "bg-amber-500", label: "" };
  }
  return { bg: "bg-slate-100 dark:bg-slate-800", dot: "bg-slate-400", label: "" };
}

// ── Hook ─────────────────────────────────────────────────────────────

interface UseShiftPickerOptions {
  isOpen: boolean;
  onSelectShift: (shiftId: string | null) => void;
  onClose: () => void;
  shifts?: Shift[];
}

interface UseShiftPickerReturn {
  searchQuery: string;
  setSearchQuery: (v: string) => void;
  activeChip: FilterChip;
  setActiveChip: (v: FilterChip) => void;
  selectedLocationId: string;
  setSelectedLocationId: (v: string) => void;
  selectedEmployerId: string;
  setSelectedEmployerId: (v: string) => void;
  searchInputRef: React.RefObject<HTMLInputElement | null>;
  formatTime: (v: string) => string;
  t: TFunction;
  locationIds: string[];
  employerIds: string[];
  filtered: Shift[];
  hasFrequent: boolean;
  sortedFrequentShifts: Shift[];
  nonFrequentFiltered: Shift[];
  showFrequentChip: boolean;
  handleSelect: (shiftId: string | null) => void;
  handleClearSearch: () => void;
  handleKeyDown: (e: React.KeyboardEvent) => void;
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

  // Reset state on open, and auto-focus search input
  useEffect(() => {
    if (isOpen) {
      setSearchQuery("");
      setActiveChip("all");
      setSelectedLocationId("");
      setSelectedEmployerId("");
      setFrequentMap(readFrequentShifts());
      const timer = setTimeout(() => {
        searchInputRef.current?.focus();
      }, 100);
      return () => { clearTimeout(timer); };
    }
  }, [isOpen]);

  // ── Frequent shift IDs ───────────────────────────────────────────
  const frequentIds = useMemo(() => getTopFrequentIds(frequentMap), [frequentMap]);

  // ── Unique location/employer IDs across all shifts ──────────────
  const locationIds = useMemo(() => {
    const all = shifts ?? [];
    return [...new Set(all.map((s) => s.location_id).filter(Boolean))] as string[];
  }, [shifts]);

  const employerIds = useMemo(() => {
    const all = shifts ?? [];
    return [...new Set(all.map((s) => s.employer_id).filter(Boolean))] as string[];
  }, [shifts]);

  // ── Derived: filtered + grouped shifts ───────────────────────────
  const { filtered, hasFrequent, frequentShifts, restShifts } = useMemo(() => {
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
        break;
    }

    // Step 2.5: apply location/employer scope filters
    let scopeFiltered = chipFiltered;
    if (selectedLocationId) {
      scopeFiltered = scopeFiltered.filter((s) => s.location_id === selectedLocationId);
    }
    if (selectedEmployerId) {
      scopeFiltered = scopeFiltered.filter((s) => s.employer_id === selectedEmployerId);
    }

    // Step 3: separate frequent from rest (only when "all" chip is active and no search)
    const isDefault = activeChip === "all" && !searchQuery.trim() && !selectedLocationId && !selectedEmployerId;
    const frequent = isDefault ? scopeFiltered.filter((s) => frequentIds.includes(s.id)) : [];
    const rest = isDefault ? scopeFiltered.filter((s) => !frequentIds.includes(s.id)) : scopeFiltered;
    const hasFreq = frequent.length > 0;

    return { filtered: scopeFiltered, frequentShifts: frequent, restShifts: rest, hasFrequent: hasFreq };
  }, [shifts, searchQuery, activeChip, frequentIds, selectedLocationId, selectedEmployerId]);

  // ── Sorted frequent shifts ───────────────────────────────────────
  const sortedFrequentShifts = useMemo(() => {
    const freq = frequentMap;
    return [...((shifts ?? []).filter((s) => frequentIds.includes(s.id)))]
      .sort((a, b) => (freq[b.id] ?? 0) - (freq[a.id] ?? 0));
  }, [shifts, frequentIds, frequentMap]);

  // ── Non-frequent filtered shifts ─────────────────────────────────
  const nonFrequentFiltered = useMemo(() => {
    if (!hasFrequent) return filtered as Shift[];
    return (filtered as Shift[]).filter((s) => !frequentIds.includes(s.id));
  }, [filtered, hasFrequent, frequentIds]);

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
    setSelectedLocationId("");
    setSelectedEmployerId("");
    searchInputRef.current?.focus();
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      if (searchQuery) {
        e.stopPropagation();
        handleClearSearch();
      }
    }
  }, [searchQuery, handleClearSearch]);

  const showFrequentChip = frequentIds.length > 0;

  return {
    searchQuery,
    setSearchQuery,
    activeChip,
    setActiveChip,
    selectedLocationId,
    setSelectedLocationId,
    selectedEmployerId,
    setSelectedEmployerId,
    searchInputRef,
    formatTime,
    t,
    locationIds,
    employerIds,
    filtered,
    hasFrequent,
    sortedFrequentShifts,
    nonFrequentFiltered,
    showFrequentChip,
    handleSelect,
    handleClearSearch,
    handleKeyDown,
  };
}
