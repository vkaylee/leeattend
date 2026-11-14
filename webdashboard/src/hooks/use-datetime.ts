import { format } from "date-fns";
import { enUS } from "date-fns/locale";

import { useOrganization } from "@/hooks/use-organization";

import {
  LOCALE_MAP,
  formatDateOnly,
  formatTimeOnly,
  formatTime24hOnly,
  formatDateTimeOnly,
  formatMonthYearOnly,
  formatDateTimeLocaleOnly,
} from "./use-datetime-utils";
import { useLanguage } from "./use-language";

/**
 * Hook providing locale- and organization-aware date/time formatters.
 * Delegates to pure functions in `use-datetime-utils.ts` after resolving
 * organization preferences (date/time format, timezone) and user language.
 */
export function useDateTimeFormatter() {
  const { organization } = useOrganization();
  const { currentLanguage } = useLanguage();

  const dateFormat = organization?.date_format ?? "DD/MM/YYYY";
  const timeFormat = organization?.time_format ?? "24h";
  const timezone = organization?.timezone;

  const datePattern = dateFormat
    .replace("DD", "dd")
    .replace("YYYY", "yyyy");

  const timePattern = timeFormat === "12h" ? "hh:mm a" : "HH:mm";

  const locale = LOCALE_MAP[currentLanguage] ?? enUS;

  return {
    formatDate: (date: string | Date | null | undefined) =>
      formatDateOnly(date, datePattern, locale),
    formatTime: (date: string | Date | null | undefined) =>
      formatTimeOnly(date, timePattern, timezone, locale),
    formatTime24h: (date: string | Date | null | undefined) =>
      formatTime24hOnly(date, timezone, locale),
    formatDateTime: (date: string | Date | null | undefined) =>
      formatDateTimeOnly(date, dateFormat, timePattern, timezone),
    formatMonthYear: (date: string | Date | null | undefined) =>
      formatMonthYearOnly(date, datePattern, locale),
    formatMonthLongYear: (date: string | Date | null | undefined) => {
      if (!date) return "---";
      try {
        return format(new Date(date), "MMMM yyyy", { locale });
      } catch {
        return String(date);
      }
    },
    formatDateTimeLocale: (date: string | Date | null | undefined) =>
      formatDateTimeLocaleOnly(date, currentLanguage),
  };
}
