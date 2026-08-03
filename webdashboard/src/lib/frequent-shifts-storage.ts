/**
 * localStorage persistence for frequently selected shifts.
 * Pure helpers — no React dependencies, fully testable in isolation.
 */

const FREQUENT_SHIFTS_KEY = "leeattend_frequent_shifts";
const FREQUENT_TOP_COUNT = 3;

export type FrequentMap = Record<string, number>;

export function readFrequentShifts(): FrequentMap {
  try {
    const raw = localStorage.getItem(FREQUENT_SHIFTS_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null) return {};
    const obj = parsed as Record<string, unknown>;
    const result: FrequentMap = {};
    for (const [k, v] of Object.entries(obj)) {
      if (typeof k === "string" && typeof v === "number" && v > 0) {
        result[k] = v;
      }
    }
    return result;
  } catch { /* localStorage unavailable or corrupt data — return empty map */
    return {};
  }
}

export function writeFrequentShift(shiftId: string): void {
  try {
    const map = readFrequentShifts();
    map[shiftId] = (map[shiftId] ?? 0) + 1;
    localStorage.setItem(FREQUENT_SHIFTS_KEY, JSON.stringify(map));
  } catch { /* localStorage may be unavailable — silently ignore */ }
}

export function getTopFrequentIds(map: FrequentMap): string[] {
  return Object.entries(map)
    .sort(([, a], [, b]) => b - a)
    .slice(0, FREQUENT_TOP_COUNT)
    .map(([id]) => id);
}
