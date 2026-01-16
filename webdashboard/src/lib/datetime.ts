/**
 * Converts a local date and time string to a UTC ISO string based on the provided timezone.
 * Uses Intl.DateTimeFormat for robust conversion without heavy libraries.
 */
export function toUTCISO(dateStr: string, timeStr: string, timezone: string): string {
  const localStr = `${dateStr}T${timeStr}:00`;
  
  try {
    const formatter = new Intl.DateTimeFormat('en-US', {
      timeZone: timezone,
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false
    });
    
    // First approximation: treat local as UTC
    let utcDate = new Date(localStr + 'Z'); 
    
    const getFormatted = (d: Date) => {
      const parts = formatter.formatToParts(d);
      const p: any = {};
      parts.forEach(part => p[part.type] = part.value);
      // Ensure month/day are correctly mapped as Intl parts can vary by locale
      // but 'en-US' with 2-digit should be stable.
      return `${p.year}-${p.month}-${p.day}T${p.hour}:${p.minute}:${p.second}`;
    };

    // Refine the UTC date by calculating the difference
    const formatted = getFormatted(utcDate);
    const diff = new Date(localStr + 'Z').getTime() - new Date(formatted + 'Z').getTime();
    utcDate = new Date(utcDate.getTime() + diff);
    
    return utcDate.toISOString();
  } catch (err) {
    console.error("Timezone conversion failed:", err);
    return localStr + 'Z';
  }
}
