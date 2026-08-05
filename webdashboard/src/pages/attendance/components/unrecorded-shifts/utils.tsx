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
  // eslint-disable-next-line react-refresh/only-export-components
export function zoneLabel(
  row: ZonedNoShowRow,
  t: (k: string, o?: Record<string, unknown>) => string,
): { key: string; hint: string | null } {
  if (row.recording_issue === "missing_punch_out") {
    return {
      key: t("unrecorded_shifts.issue_missing_punch_out", {
        defaultValue: "Thiếu punch-out",
      }),
      hint: row.expected_end_at
        ? t("unrecorded_shifts.expected_end_hint", {
            defaultValue: "Dự kiến kết thúc {{time}}",
            time: formatDateTime(row.expected_end_at),
          })
        : null,
    };
  }
  if (row.recording_issue === "active_overtime") {
    return {
      key: t("unrecorded_shifts.issue_active_overtime", {
        defaultValue: "Đang tăng ca",
      }),
      hint: null,
    };
  }
  if (row.recording_issue === "punch_out_pending") {
    return {
      key: t("unrecorded_shifts.issue_punch_out_pending", {
        defaultValue: "Chưa xác nhận punch-out",
      }),
      hint: null,
    };
  }
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

export function formatDateTime(value: string | null | undefined): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

/** Format "HH:MM:SS" → "HH:MM" safely. */
  // eslint-disable-next-line react-refresh/only-export-components
export function formatShiftTime(timeStr: string | null | undefined): string {
  if (!timeStr) return "—";
  try {
    const parts = timeStr.split(":");
    if (parts.length >= 2) {
      return `${parts[0].padStart(2, "0")}:${parts[1].padStart(2, "0")}`;
    }
  } catch { /* split/padStart fails on unexpected input — return raw time string */
  }
  return timeStr;
}

/** Decorative zone icon — aria-hidden, label lives on the parent cell. */
export function ZoneIcon({ zone }: { zone: ZonedNoShowRow["zone"] }) {
  if (zone === "b") return <AlertTriangle aria-hidden="true" className="size-3.5" />;
  if (zone === "a") return <Clock aria-hidden="true" className="size-3.5" />;
  return <Check aria-hidden="true" className="size-3.5" />;
}
