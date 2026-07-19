import {
  AlertTriangle,
  Check,
  Clock,
} from "lucide-react";

import type { ZonedNoShowRow } from "../../hooks/use-no-show-rows";

/**
 * i18n key + hint for a zone label. Kept alongside the icon/text so the row is
 * never distinguishable by color alone (WCAG 2.1 AA — multi-channel cues).
 */
export function zoneLabel(
  row: ZonedNoShowRow,
  t: (k: string, o?: Record<string, unknown>) => string,
): { key: string; hint: string | null } {
  if (row.zone === "b") {
    return {
      key: t("unrecorded_shifts.zone_b"),
      hint: row.zoneMinutes == null
        ? null
        : t("unrecorded_shifts.zone_b_hint", { minutes: row.zoneMinutes }),
    };
  }
  if (row.zone === "a") {
    return {
      key: t("unrecorded_shifts.zone_a"),
      hint: row.zoneMinutes == null
        ? null
        : t("unrecorded_shifts.zone_a_hint", { minutes: row.zoneMinutes }),
    };
  }
  return { key: t("unrecorded_shifts.zone_c"), hint: null };
}

/** Format "HH:MM:SS" → "HH:MM" safely. */
export function formatShiftTime(timeStr: string | null | undefined): string {
  if (!timeStr) return "—";
  try {
    const parts = timeStr.split(":");
    if (parts.length >= 2) {
      return `${parts[0].padStart(2, "0")}:${parts[1].padStart(2, "0")}`;
    }
  } catch {
    // fall through
  }
  return timeStr;
}

/** Decorative zone icon — aria-hidden, label lives on the parent cell. */
export function ZoneIcon({ zone }: { zone: ZonedNoShowRow["zone"] }) {
  if (zone === "b") return <AlertTriangle aria-hidden="true" className="size-3.5" />;
  if (zone === "a") return <Clock aria-hidden="true" className="size-3.5" />;
  return <Check aria-hidden="true" className="size-3.5" />;
}
