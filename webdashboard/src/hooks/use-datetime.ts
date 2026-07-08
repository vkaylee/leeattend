import { format } from "date-fns";
import { enUS, vi, zhCN, type Locale } from "date-fns/locale";

import { useLanguage } from "./use-language";
import { useWorkspace } from "./use-workspace";

const LOCALE_MAP: Record<string, Locale> = {
  vi,
  en: enUS,
  zh: zhCN,
};

export function useDateTimeFormatter() {
  const { workspace } = useWorkspace();
  const { currentLanguage } = useLanguage();

  const dateFormat = workspace?.date_format ?? "DD/MM/YYYY";
  const timeFormat = workspace?.time_format ?? "24h";

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
      // If it's just HH:mm:ss, date-fns format might fail if not a full date
      // We wrap it in a full date if needed
      let d = new Date(date);
      if (isNaN(d.getTime()) && typeof date === 'string' && date.includes(':')) {
        const [h, m, s] = date.split(':');
        d = new Date();
        d.setHours(parseInt(h), parseInt(m), s ? parseInt(s) : 0);
      }
      return format(d, timePattern, { locale: dateFnsLocale });
    } catch {
      return String(date);
    }
  };

  const formatTime24h = (date: string | Date | null | undefined) => {
    if (!date) return "";
    try {
      let d = new Date(date);
      if (isNaN(d.getTime()) && typeof date === 'string' && date.includes(':')) {
        const [h, m, s] = date.split(':');
        d = new Date();
        d.setHours(parseInt(h), parseInt(m), s ? parseInt(s) : 0);
      }
      return format(d, "HH:mm", { locale: dateFnsLocale });
    } catch {
      return "";
    }
  };

  const formatDateTime = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      return format(new Date(date), `${datePattern} ${timePattern}`, { locale: dateFnsLocale });
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
