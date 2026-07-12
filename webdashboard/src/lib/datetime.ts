import { logError } from "./logger";

/**
 * Converts a local date and time string to a UTC ISO string based on the provided timezone.
 * Uses Intl.DateTimeFormat for robust conversion without heavy libraries.
 */
export function toUTCISO(dateString: string, timeString: string, timezone: string): string {
  const localString = `${dateString}T${timeString}:00`;
  
  try {
    const formatter = new Intl.DateTimeFormat('en-GB', {
      timeZone: timezone,
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hourCycle: 'h23'
    });
    
    // First approximation: treat local as UTC
    let utcDate = new Date(localString + 'Z'); 
    
    const getFormatted = (d: Date) => {
      const parts = formatter.formatToParts(d);
 
      const p: Record<string, string> = {};
      parts.forEach(part => p[part.type] = part.value);
      // Ensure ISO-like string YYYY-MM-DDTHH:mm:ss
      return `${p.year}-${p.month}-${p.day}T${p.hour}:${p.minute}:${p.second}`;
    };

    // Refine the UTC date by calculating the difference
    const formatted = getFormatted(utcDate);
    const diff = new Date(localString + 'Z').getTime() - new Date(formatted + 'Z').getTime();
    utcDate = new Date(utcDate.getTime() + diff);
    
    return utcDate.toISOString();
  } catch (error) {
    logError(error, { component: "TimezoneConversion" });
    return localString + 'Z';
  }
}
