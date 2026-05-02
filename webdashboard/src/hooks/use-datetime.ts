import { format } from "date-fns";

import { useWorkspace } from "./use-workspace";

export function useDateTimeFormatter() {
  const { workspace } = useWorkspace();
  
  const dateFormat = workspace?.date_format || "DD/MM/YYYY";
  const timeFormat = workspace?.time_format || "24h";

  // Map backend formats to date-fns tokens
  // Backend: DD/MM/YYYY, MM/DD/YYYY, YYYY-MM-DD
  // date-fns: dd/MM/yyyy, MM/dd/yyyy, yyyy-MM-dd
  const datePattern = dateFormat
    .replace("DD", "dd")
    .replace("YYYY", "yyyy");
  
  const timePattern = timeFormat === "12h" ? "hh:mm a" : "HH:mm";

  const formatDate = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      return format(new Date(date), datePattern);
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
      return format(d, timePattern);
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
      return format(d, "HH:mm");
    } catch {
      return "";
    }
  };

  const formatDateTime = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      return format(new Date(date), `${datePattern ?? ""} ${timePattern ?? ""}`);
    } catch {
      return String(date);
    }
  };

  const formatMonthYear = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      // Use localized month and year based on datePattern
      const pattern = datePattern.includes("dd/MM") ? "MM/yyyy" : datePattern.includes("MM/dd") ? "MM/yyyy" : "yyyy-MM";
      return format(new Date(date), pattern);
    } catch {
      return String(date);
    }
  };

  return { formatDate, formatTime, formatTime24h, formatDateTime, formatMonthYear };
}
