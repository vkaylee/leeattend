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
    } catch (e) {
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
    } catch (e) {
      return String(date);
    }
  };

  const formatDateTime = (date: string | Date | null | undefined) => {
    if (!date) return "---";
    try {
      return format(new Date(date), `${datePattern} ${timePattern}`);
    } catch (e) {
      return String(date);
    }
  };

  return { formatDate, formatTime, formatDateTime };
}
