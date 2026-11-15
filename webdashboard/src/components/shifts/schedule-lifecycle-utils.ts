export function formatScheduleLocalTime(value: string | null | undefined, timezone: string): string | null {
  if (!value || !timezone) return null;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return null;

  let parts: Intl.DateTimeFormatPart[];
  try {
    parts = new Intl.DateTimeFormat("en-GB", {
      hour: "2-digit",
      hourCycle: "h23",
      minute: "2-digit",
      second: "2-digit",
      timeZone: timezone,
    }).formatToParts(date);
  } catch {
    return null;
  }
  const values = Object.fromEntries(parts.map(({ type, value: part }) => [type, part]));
  if (!values.hour || !values.minute || !values.second) return null;
  return `${values.hour}:${values.minute}:${values.second}`;
}
