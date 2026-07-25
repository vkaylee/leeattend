/**
 * Filter definitions and pure filter functions for the shift picker.
 * Split out of `use-shift-picker` so the hook stays readable.
 */

import type { Shift } from "@/lib/api";

import { isNightShift, isSplitShift } from "../shift-utils";

export type FilterChip = "all" | "frequent" | "night" | "single" | "split";

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

export function applyChipFilter(
  shifts: Shift[],
  chip: FilterChip,
  frequentIds: string[],
): Shift[] {
  switch (chip) {
    case "night":
      return shifts.filter((s) => isNightShift(s));
    case "single":
      return shifts.filter((s) => !isSplitShift(s));
    case "split":
      return shifts.filter((s) => isSplitShift(s));
    case "frequent":
      return shifts.filter((s) => frequentIds.includes(s.id));
    default:
      return shifts;
  }
}

/**
 * True when nothing is narrowing the list. The "frequently used" section only
 * renders in this state — with a filter applied it would show the same shifts
 * twice, once in the section and once in the filtered list below it.
 */
export function isDefaultView(
  chip: FilterChip,
  searchQuery: string,
  locationId: string,
  employerId: string,
): boolean {
  return chip === "all" && !searchQuery.trim() && !locationId && !employerId;
}

/** Unique non-null values of one shift field, for the scope dropdowns. */
export function uniqueIds(shifts: Shift[], key: "employer_id" | "location_id"): string[] {
  return [...new Set(shifts.map((s) => s[key]).filter(Boolean))] as string[];
}

export function applyScopeFilters(
  shifts: Shift[],
  locationId: string,
  employerId: string,
): Shift[] {
  let result = shifts;
  if (locationId) {
    result = result.filter((s) => s.location_id === locationId);
  }
  if (employerId) {
    result = result.filter((s) => s.employer_id === employerId);
  }
  return result;
}
