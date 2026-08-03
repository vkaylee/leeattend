import { format } from "date-fns";
import { enUS, vi, zhCN, type Locale } from "date-fns/locale";

// ── Constants ───────────────────────────────────────────────────────────────

export const LOCALE_MAP: Record<string, Locale> = {
  vi,
  en: enUS,
  zh: zhCN,
};

const UTC_TZ = "UTC";

const LOCALE_INTL_MAP: Record<string, string> = {
  vi: "vi-VN",
  zh: "zh-CN",
  en: "en-US",
};

// ── Timezone-aware helpers ──────────────────────────────────────────────────

/**
 * Build an Intl formatter for the given IANA timezone, falling back to UTC
 * when the workspace timezone is missing or invalid — the same fallback the
 * backend uses (`parse_workspace_tz`).
 */
export function makeWorkspaceTzFormatter(timezone: string | undefined, hour12: boolean): Intl.DateTimeFormat {
  try {
    return new Intl.DateTimeFormat("en-GB", {
      timeZone: timezone ?? UTC_TZ,
      hour: "2-digit",
      minute: "2-digit",
      hour12,
    });
  } catch {
    return new Intl.DateTimeFormat("en-GB", {
      timeZone: UTC_TZ,
      hour: "2-digit",
      minute: "2-digit",
      hour12,
    });
  }
}

/** Date-only formatter in the workspace timezone (UTC fallback). */
export function makeWorkspaceDateFormatter(timezone: string | undefined): Intl.DateTimeFormat {
  try {
    return new Intl.DateTimeFormat("en-GB", {
      timeZone: timezone ?? UTC_TZ,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    });
  } catch {
    return new Intl.DateTimeFormat("en-GB", {
      timeZone: UTC_TZ,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    });
  }
}

/**
 * Format a real UTC timestamp in the workspace timezone (matching the Excel
 * export's `format_datetime(actual_punch_in, tz)`). Naive schedule times
 * ("HH:mm:ss") are never passed here — they carry no timezone.
 */
export function formatInWorkspaceTz(date: Date, timePattern: string, timezone: string | undefined): string {
  const hour12 = timePattern.includes("a");
  const formatted = makeWorkspaceTzFormatter(timezone, hour12).format(date);
  // date-fns "hh:mm a" emits uppercase AM/PM; en-GB Intl emits lowercase "am"/"pm".
  return hour12 ? formatted.replace(" am", " AM").replace(" pm", " PM") : formatted;
}

// ── Pure formatting functions ───────────────────────────────────────────────

/**
 * Format a date without time component.
 * Returns "---" for null/undefined, or the raw string on parse failure.
 */
export function formatDateOnly(date: string | Date | null | undefined, pattern: string, locale: Locale): string {
  if (!date) return "---";
  try {
    return format(new Date(date), pattern, { locale });
  } catch {
    return String(date);
  }
}

/**
 * Format a time value. Handles two cases:
 * 1. Naive "HH:mm:ss" strings (scheduled_start/end) — no timezone shift
 * 2. Real UTC timestamps — rendered in the workspace timezone (Excel parity)
 * Returns "---" for null/undefined, or the raw string on parse failure.
 */
export function formatTimeOnly(
  date: string | Date | null | undefined,
  timePattern: string,
  timezone: string | undefined,
  locale: Locale,
): string {
  if (!date) return "---";
  try {
    const d = new Date(date);
    if (isNaN(d.getTime()) && typeof date === "string" && date.includes(":")) {
      const [h, m, s] = date.split(":");
      const dt = new Date();
      dt.setHours(parseInt(h), parseInt(m), s ? parseInt(s) : 0);
      return format(dt, timePattern, { locale });
    }
    return formatInWorkspaceTz(d, timePattern, timezone);
  } catch {
    return String(date);
  }
}

/**
 * Format a time value in 24-hour format.
 * Same logic as formatTimeOnly but always uses "HH:mm" pattern.
 * Returns "" for null/undefined (used as input placeholder).
 */
export function formatTime24hOnly(
  date: string | Date | null | undefined,
  timezone: string | undefined,
  locale: Locale,
): string {
  if (!date) return "";
  try {
    const d = new Date(date);
    if (isNaN(d.getTime()) && typeof date === "string" && date.includes(":")) {
      const [h, m, s] = date.split(":");
      const dt = new Date();
      dt.setHours(parseInt(h), parseInt(m), s ? parseInt(s) : 0);
      return format(dt, "HH:mm", { locale });
    }
    return formatInWorkspaceTz(d, "HH:mm", timezone);
  } catch {
    return "";
  }
}

/**
 * Format a combined date+time in the configured workspace format.
 * Both date and time parts render in the workspace timezone so timestamps
 * near midnight don't shift to a different calendar day.
 * Returns "---" for null/undefined, or the raw string on parse failure.
 */
export function formatDateTimeOnly(
  date: string | Date | null | undefined,
  dateFormat: string,
  timePattern: string,
  timezone: string | undefined,
): string {
  if (!date) return "---";
  try {
    const d = new Date(date);
    if (isNaN(d.getTime())) {
      return String(date);
    }
    const dayPart = makeWorkspaceDateFormatter(timezone).format(d);
    const timePart = formatInWorkspaceTz(d, timePattern, timezone);
    const [dd, mm, yyyy] = dayPart.split("/");
    const ordered =
      dateFormat.startsWith("MM/DD")
        ? `${mm}/${dd}/${yyyy}`
        : dateFormat.startsWith("YYYY")
          ? `${yyyy}-${mm}-${dd}`
          : `${dd}/${mm}/${yyyy}`;
    return `${ordered} ${timePart}`;
  } catch {
    return String(date);
  }
}

/**
 * Format month and year based on the workspace date pattern.
 * DD/MM/YYYY → MM/yyyy, MM/DD/YYYY → MM/yyyy, YYYY-MM-DD → yyyy-MM
 * Returns "---" for null/undefined, or the raw string on parse failure.
 */
export function formatMonthYearOnly(
  date: string | Date | null | undefined,
  datePattern: string,
  locale: Locale,
): string {
  if (!date) return "---";
  try {
    const pattern = datePattern.includes("dd/MM")
      ? "MM/yyyy"
      : datePattern.includes("MM/dd")
        ? "MM/yyyy"
        : "yyyy-MM";
    return format(new Date(date), pattern, { locale });
  } catch {
    return String(date);
  }
}

/**
 * Locale-aware date+time formatting using Intl.DateTimeFormat with medium
 * dateStyle/timeStyle. Suitable for audit logs, system timestamps, and
 * other contexts where the workspace timezone is irrelevant.
 * Returns "---" for null/undefined, or the raw string on parse failure.
 */
export function formatDateTimeLocaleOnly(
  date: string | Date | null | undefined,
  currentLanguage: string,
): string {
  if (!date) return "---";
  try {
    const intlLocale = LOCALE_INTL_MAP[currentLanguage] ?? "en-US";
    return new Intl.DateTimeFormat(intlLocale, {
      dateStyle: "medium",
      timeStyle: "medium",
    }).format(new Date(date));
  } catch {
    return String(date);
  }
}
