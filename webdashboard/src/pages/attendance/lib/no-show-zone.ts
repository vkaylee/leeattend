/**
 * Pure A/B/C no-show zone model. No React, no i18n, no side effects — trivially
 * unit-testable and shared by the table UI and the Excel export.
 *
 * Zone timeline for an absent employee at reference time `now`:
 *   shift_start                shift_start + grace               shift_end
 *     │◄──── Zone A ──────────►│◄────── Zone B ────────────────►│◄── Zone C ──►│
 *     │ "Chờ · còn X phút"     │ "Grace hết – cần xác minh ngay" │ "Vắng (kết thúc ca)" │
 *
 * - Zone A: now ≤ start + grace  → still in grace, no action yet (countdown).
 * - Zone B: start + grace < now < shift_end → grace expired, shift in progress:
 *           this is the operational action queue (elapsed).
 * - Zone C: now ≥ shift_end, or a past date → fixed absent record.
 */

const MINUTES_PER_DAY = 24 * 60;

export type Zone = "a" | "b" | "c";

export interface ZoneInfo {
  zone: Zone;
  /** For Zone A: minutes of grace remaining. For Zone B: minutes past grace. */
  minutes: number | null;
}

/** "HH:MM" / "HH:MM:SS" → minutes since midnight, or null when unparseable. */
export function parseTimeToMinutes(time: string | null | undefined): number | null {
  if (!time) return null;
  const parts = time.split(":").map(Number);
  if (parts.length < 2 || parts.some((n) => Number.isNaN(n))) return null;
  return parts[0] * 60 + parts[1];
}

/**
 * Compute the A/B/C zone for one absent row.
 *
 * @param scheduledStart "HH:MM:SS" | null  (matches DailyAttendanceReport.scheduled_start)
 * @param scheduledEnd   "HH:MM:SS" | null
 * @param graceMinutes   resolved grace; defaults to 15 when null (matches backend)
 * @param now            injectable reference clock (real-time `now` for live view)
 */
export function computeZone(
  scheduledStart: string | null | undefined,
  scheduledEnd: string | null | undefined,
  graceMinutes: number | null | undefined,
  now: Date,
): ZoneInfo {
  const grace = graceMinutes ?? 15;
  const startMins = parseTimeToMinutes(scheduledStart);
  const endMins = parseTimeToMinutes(scheduledEnd);

  // No shift times → neutral Zone C (never throws, never mislabels).
  if (startMins == null || endMins == null) {
    return { zone: "c", minutes: null };
  }

  // Overnight shifts (e.g. 22:00–06:00) cross midnight. Push the end time into the
  // next day, and if `now` is before the start time, treat it as the post-midnight
  // tail of the shift so the comparisons below stay correct.
  const crossesMidnight = endMins <= startMins;
  const effectiveEnd = crossesMidnight ? endMins + MINUTES_PER_DAY : endMins;
  let nowMins = now.getHours() * 60 + now.getMinutes() + now.getSeconds() / 60;
  if (crossesMidnight && nowMins < startMins) {
    nowMins += MINUTES_PER_DAY;
  }

  const graceEndsAt = startMins + grace;

  if (nowMins >= effectiveEnd) {
    return { zone: "c", minutes: null };
  }
  if (nowMins <= graceEndsAt) {
    return { zone: "a", minutes: Math.ceil(graceEndsAt - nowMins) };
  }
  return { zone: "b", minutes: Math.ceil(nowMins - graceEndsAt) };
}

/** True when the user selected a historical date → all rows fixed to Zone C. */
export function isViewingPastDate(selectedDate: string, today: string): boolean {
  if (!selectedDate || !today) return false;
  // ISO "yyyy-MM-dd" compares lexicographically.
  return selectedDate < today;
}
