import { format } from "date-fns";
import { enUS, vi, zhCN, type Locale } from "date-fns/locale";

import { useLanguage } from "./use-language";
import { useWorkspace } from "./use-workspace";

const LOCALE_MAP: Record<string, Locale> = {
  vi,
  en: enUS,
  zh: zhCN,
};

const UTC_TZ = "UTC";

/**
 * Build an Intl formatter for the given IANA timezone, falling back to UTC
 * when the workspace timezone is missing or invalid — the same fallback the
 * backend uses (`parse_workspace_tz`).
 */
function makeWorkspaceTzFormatter(timezone: string | undefined, hour12: boolean): Intl.DateTimeFormat {
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
function makeWorkspaceDateFormatter(timezone: string | undefined): Intl.DateTimeFormat {
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
function formatInWorkspaceTz(date: Date, timePattern: string, timezone: string | undefined): string {
  const hour12 = timePattern.includes("a");
  const formatted = makeWorkspaceTzFormatter(timezone, hour12).format(date);
  // date-fns "hh:mm a" emits uppercase AM/PM; en-GB Intl emits lowercase "am"/"pm".
  return hour12 ? formatted.replace(" am", " AM").replace(" pm", " PM") : formatted;
}

export function useDateTimeFormatter() {
  const { workspace } = useWorkspace();
  const { currentLanguage } = useLanguage();

  const dateFormat = workspace?.date_format ?? "DD/MM/YYYY";
  const timeFormat = workspace?.time_format ?? "24h";
  // Punch timestamps are stored as UTC; display them in the workspace timezone,
  // NOT the viewer's browser timezone (which is what made 05:30 UTC render as
  // 12:30 for users in UTC+7 while the Excel export showed the correct 05:30).
  const timezone = workspace?.timezone;

  // Map backend formats to date-fns tokens
  // Backend: DD/MM/YYYY, MM/DD/YYYY, YYYY-MM-DD
  // date-fns: dd/MM/yyyy, MM/dd/yyyy, yyyy-MM-dd
  const datePattern = dateFormat
    .replace("DD", "dd")
    .replace("YYYY", "yyyy");

  const timePattern = timeFormat === "12h" ? "hh:mm a" : "HH:mm";

  const dateFnsLocale = LOCALE_MAP[currentLanguage] ?? enUS;

  const formatDate = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      return format(new Date(date), datePattern, { locale: dateFnsLocale });
    } catch {
      return String(date);
    }
  };

  const formatTime = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      // If it's just HH:mm:ss (a NaiveTime like scheduled_start/end), keep it
      // as-is — it carries no timezone and must not be shifted.
      const d = new Date(date);
      if (isNaN(d.getTime()) && typeof date === 'string' && date.includes(':')) {
        const [h, m, s] = date.split(':');
        const dt = new Date();
        dt.setHours(parseInt(h), parseInt(m), s ? parseInt(s) : 0);
        return format(dt, timePattern, { locale: dateFnsLocale });
      }
      // Real UTC timestamp → render in the workspace timezone (Excel parity).
      return formatInWorkspaceTz(d, timePattern, timezone);
    } catch {
      return String(date);
    }
  };

  const formatTime24h = (date: string | Date | null | undefined) => {
    if (!date) return "";
    try {
      const d = new Date(date);
      if (isNaN(d.getTime()) && typeof date === 'string' && date.includes(':')) {
        const [h, m, s] = date.split(':');
        const dt = new Date();
        dt.setHours(parseInt(h), parseInt(m), s ? parseInt(s) : 0);
        return format(dt, "HH:mm", { locale: dateFnsLocale });
      }
      return formatInWorkspaceTz(d, "HH:mm", timezone);
    } catch {
      return "";
    }
  };

  const formatDateTime = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      const d = new Date(date);
      if (isNaN(d.getTime())) {
        return String(date);
      }
      // Render the date part in the workspace timezone too so a timestamp near
      // midnight doesn't shift to a different calendar day.
      const dayPart = makeWorkspaceDateFormatter(timezone).format(d);
      const timePart = formatInWorkspaceTz(d, timePattern, timezone);
      // Reorder day/month/year to match the configured date format.
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
  };

  const formatMonthYear = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      // Use localized month and year based on datePattern
      const pattern = datePattern.includes("dd/MM") ? "MM/yyyy" : datePattern.includes("MM/dd") ? "MM/yyyy" : "yyyy-MM";
      return format(new Date(date), pattern, { locale: dateFnsLocale });
    } catch {
      return String(date);
    }
  };

  return { formatDate, formatTime, formatTime24h, formatDateTime, formatMonthYear };
}
