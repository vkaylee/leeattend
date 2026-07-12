/**
 * Pure A/B/C no-show zone model. No React, no i18n, no side effects — trivially
 * unit-testable and shared by the table UI and the Excel export.
 *
 * Zone timeline for an absent employee at reference time `A`:
 *   shift_start                shift_start + grace               shift_end
 *     │◄──── Zone A ──────────►│◄────── Zone B ────────────────►│◄── Zone C ──►│
 *     │ "Chờ · còn X phút"     │ "Grace hết – cần xác minh ngay" │ "Vắng (kết thúc ca)" │
 *
 * - Zone A: A ≤ start + grace  → still in grace, no action yet (countdown).
 * - Zone B: start + grace < A < shift_end → grace expired, shift in progress:
 *           this is the operational action queue (elapsed).
 * - Zone C: A ≥ shift_end → fixed absent record.
 *
 * TIME MODEL (redesign — TZ-aware, absolute time):
 * The old implementation compared `now.getHours()` (a wall-clock reading that
 * throws away the date and ignores timezone) against "HH:MM:SS" shift times.
 * That breaks the moment the reference time `A` is any instant other than "the
 * client's current local wall clock" — which is exactly what the timeline
 * scrubber needs (A fixed in the past/future, ±12h, in the *workspace* TZ).
 *
 * So this version works entirely in absolute epoch-minutes. The caller resolves
 * the shift's start/end to absolute instants from (shiftDate + "HH:MM:SS") in
 * the workspace timezone, and passes the reference time `A` as a `Date`. The
 * helper `epochMinutesInTz` does the wall-clock → UTC conversion with
 * `Intl.DateTimeFormat` (no extra deps — date-fns-tz is not in the bundle).
 */

const MINUTES_PER_DAY = 24 * 60;
const MS_PER_MINUTE = 60_000;

export type Zone = "a" | "b" | "c";

export interface ZoneInfo {
  /** For Zone A: minutes of grace remaining. For Zone B: minutes past grace. */
  minutes: number | null;
  zone: Zone;
}

/** Inputs to `computeZone`. All time math is derived from these; the function
 *  never reads the system clock, so it stays pure and deterministic. */
export interface ComputeZoneInput {
  /** Resolved grace in minutes; defaults to 15 when null. */
  graceMinutes: number | null | undefined;
  /** The absolute reference instant `A` (the scrubber's pinned center). */
  referenceTime: Date;
  /** Shift end "HH:MM:SS" | null, interpreted on `shiftDate` in `tz`. */
  scheduledEnd: string | null | undefined;
  /** Shift start "HH:MM:SS" | null, interpreted on `shiftDate` in `tz`. */
  scheduledStart: string | null | undefined;
  /** The shift's assigned calendar day "yyyy-MM-dd" (overnight shifts are
   *  anchored to their start date). */
  shiftDate: string;
  /** IANA workspace timezone, e.g. "Asia/Ho_Chi_Minh". */
  tz: string;
}

/** "HH:MM" / "HH:MM:SS" → minutes since midnight, or null when unparseable. */
export function parseTimeToMinutes(time: string | null | undefined): number | null {
  if (!time) return null;
  const parts = time.split(":").map(Number);
  if (parts.length < 2 || parts.some((n) => Number.isNaN(n))) return null;
  return parts[0] * 60 + parts[1];
}

/**
 * Absolute epoch-minutes of a wall-clock (date + time) read in `tz`.
 *
 * `date` is "yyyy-MM-dd"; `time` is "HH:MM" or "HH:MM:SS". Uses
 * `Intl.DateTimeFormat.formatToParts` to read the wall-clock back from a guess
 * UTC instant and corrects in two passes (handles DST gaps/overlaps). Returns
 * `NaN` only if `tz` is invalid — callers guard with a valid IANA string.
 */
export function epochMinutesInTz(
  date: string,
  time: string,
  tz: string,
): number {
  const hm = time.split(":").slice(0, 2).join(":");
  const localIso = `${date}T${hm}:00`;

  const formatter = new Intl.DateTimeFormat("en-GB", {
    timeZone: tz,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
  });

  const readWall = (utc: Date): Date => {
    const parts = formatter.formatToParts(utc);
    const p: Record<string, string> = {};
    for (const part of parts) p[part.type] = part.value;
    return new Date(`${p.year}-${p.month}-${p.day}T${p.hour}:${p.minute}:${p.second}Z`);
  };

  // First approximation: treat the wall-clock as UTC, then correct by the
  // offset between intended wall-clock and what the tz actually reads back.
  let utc = new Date(`${localIso}Z`);
  for (let i = 0; i < 2; i += 1) {
    const wall = readWall(utc);
    const diff = new Date(`${localIso}Z`).getTime() - wall.getTime();
    utc = new Date(utc.getTime() + diff);
  }
  return Math.round(utc.getTime() / MS_PER_MINUTE);
}

/**
 * Compute the A/B/C zone for one absent row at absolute reference time `A`.
 *
 * Pure: given the same inputs it always returns the same `ZoneInfo`, and it
 * never touches React/i18n/the network. The table UI and the Excel export both
 * call this, so they can never disagree.
 */
export function computeZone(input: ComputeZoneInput): ZoneInfo {
  const {
    scheduledStart,
    scheduledEnd,
    graceMinutes,
    referenceTime,
    shiftDate,
    tz,
  } = input;

  const grace = graceMinutes ?? 15;

  // Missing shift times or context → neutral Zone C (never throws, never mislabels).
  if (
    scheduledStart == null ||
    scheduledEnd == null ||
    !shiftDate ||
    !tz
  ) {
    return { zone: "c", minutes: null };
  }

  // Absolute epoch-minutes of the shift boundaries, anchored to `shiftDate` in
  // the workspace timezone. Overnight shifts (end ≤ start) push the end +24h.
  const startMins = epochMinutesInTz(shiftDate, scheduledStart, tz);
  let endMins = epochMinutesInTz(shiftDate, scheduledEnd, tz);
  if (endMins <= startMins) {
    endMins += MINUTES_PER_DAY;
  }

  const refMins = referenceTime.getTime() / MS_PER_MINUTE;
  const graceEndsAt = startMins + grace;

  if (refMins >= endMins) {
    return { zone: "c", minutes: null };
  }
  if (refMins <= graceEndsAt) {
    return { zone: "a", minutes: Math.ceil(graceEndsAt - refMins) };
  }
  return { zone: "b", minutes: Math.ceil(refMins - graceEndsAt) };
}

/** True when the user pinned `A` on a calendar day before today (in workspace
 *  tz) → every row is a fixed historical record, no live countdown. */
export function isViewingPastDate(shiftDate: string, today: string): boolean {
  if (!shiftDate || !today) return false;
  // ISO "yyyy-MM-dd" compares lexicographically.
  return shiftDate < today;
}
