import type { ChipDef } from "./types";

// ── Filter chip definitions ────────────────────────────────────────────

export const FILTER_CHIPS: ChipDef[] = [
  { key: "all", labelKey: "shifts.picker.filter_all" },
  { key: "active", labelKey: "common.status_active" },
  { key: "inactive", labelKey: "common.status_inactive" },
  { key: "night", labelKey: "shifts.picker.filter_night" },
];
