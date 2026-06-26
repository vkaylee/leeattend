import { useCallback, useMemo } from "react";

const STORAGE_KEY = "leeattend-quick-access";
const MAX_ITEMS = 10;
const DISPLAY_COUNT = 3;

// ── Helpers ────────────────────────────────────────────────────────

function getQuickAccess(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((id): id is string => typeof id === "string");
  } catch {
    return [];
  }
}

function setQuickAccess(ids: string[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(ids.slice(0, MAX_ITEMS)));
  } catch { /* noop */ }
}

// ── Hook ───────────────────────────────────────────────────────────

/**
 * Tracks recently accessed apps via localStorage.
 * Records an app access and returns the top N most recent.
 */
export function useQuickAccess() {
  const allIds = useMemo(() => getQuickAccess(), []);

  const top3 = useMemo(() => allIds.slice(0, DISPLAY_COUNT), [allIds]);

  const recordAccess = useCallback((appId: string) => {
    const current = getQuickAccess();
    const filtered = current.filter((id) => id !== appId);
    filtered.unshift(appId);
    setQuickAccess(filtered);
  }, []);

  return { top3, all: allIds, recordAccess };
}
