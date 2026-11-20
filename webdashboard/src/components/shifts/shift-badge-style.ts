/**
 * Visual tokens for shift cards.
 *
 * These used to be derived at render time by chaining
 * `dot.replace("bg-", "text-").replace("-500", "-600")`, which broke silently
 * whenever a palette step changed and produced classes Tailwind could not see
 * at build time. Every class is now spelled out literally.
 */

import type { Shift } from "@/lib/api";

import { getShiftIntervals } from "@/lib/shift-card-metrics";

import { isNightShift, isSplitShift } from "./shift-utils";

export type ShiftAccent = "afternoon" | "morning" | "night" | "split";

export interface ShiftBadgeStyle {
  /** Solid accent used by the left rail and the timeline work blocks. */
  accent: string;
  /** Soft background for the badge pill. */
  bg: string;
  /** Empty when the shift needs no pill (plain day shifts). */
  label: string;
  /** Pill text colour, contrast-checked against `bg` (WCAG AA, ≥4.5:1). */
  text: string;
}

const ACCENT_STYLES: Record<ShiftAccent, Omit<ShiftBadgeStyle, "label">> = {
  afternoon: {
    accent: "bg-sky-500",
    bg: "bg-sky-50 dark:bg-sky-950/40",
    text: "text-sky-700 dark:text-sky-300",
  },
  morning: {
    accent: "bg-amber-500",
    bg: "bg-amber-50 dark:bg-amber-950/40",
    text: "text-amber-700 dark:text-amber-300",
  },
  night: {
    accent: "bg-indigo-500",
    bg: "bg-indigo-50 dark:bg-indigo-950/40",
    text: "text-indigo-700 dark:text-indigo-300",
  },
  split: {
    accent: "bg-teal-500",
    bg: "bg-teal-50 dark:bg-teal-950/40",
    text: "text-teal-700 dark:text-teal-300",
  },
};

/** Classify a shift for colour-coding purposes. */
export function getShiftAccent(shift: Shift): ShiftAccent {
  if (isNightShift(shift)) return "night";
  if (isSplitShift(shift)) return "split";
  const firstStart = getShiftIntervals(shift)[0]?.start_time ?? "";
  return firstStart && firstStart < "12:00" ? "morning" : "afternoon";
}

export function getShiftBadge(shift: Shift, t: (key: string) => string): ShiftBadgeStyle {
  const accent = getShiftAccent(shift);
  const style = ACCENT_STYLES[accent];

  const labels: Record<ShiftAccent, string> = {
    afternoon: "",
    morning: "",
    night: t("shifts.picker.filter_night"),
    split: t("shifts.picker.filter_split"),
  };

  return { ...style, label: labels[accent] };
}
