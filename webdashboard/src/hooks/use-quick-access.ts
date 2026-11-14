import { useCallback, useEffect, useMemo, useState } from "react";

const STORAGE_KEY = "leeattend-quick-access";
const EVENT_NAME = "leeattend-quick-access-change";
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
  } catch { /* localStorage unavailable or corrupt — return empty list */
    return [];
  }
}

function setQuickAccess(ids: string[]): void {
  try {
    const sliced = ids.slice(0, MAX_ITEMS);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(sliced));
    if (typeof window !== "undefined") {
      window.dispatchEvent(new CustomEvent(EVENT_NAME, { detail: sliced }));
    }
  } catch { /* localStorage unavailable — quick-access won't persist */ /* noop */ }
}

// ── Hook ───────────────────────────────────────────────────────────

/**
 * Tracks recently accessed apps via localStorage.
 * Records an app access and returns the top N most recent.
 */
export function useQuickAccess() {
  const [allIds, setAllIds] = useState<string[]>(() => getQuickAccess());

  useEffect(() => {
    const handleLocalChange = (e: Event) => {
      const customEvent = e as CustomEvent<string[]>;
      if (Array.isArray(customEvent.detail)) {
        setAllIds(customEvent.detail);
      } else {
        setAllIds(getQuickAccess());
      }
    };

    const handleStorageChange = (e: StorageEvent) => {
      if (e.key === STORAGE_KEY) {
        setAllIds(getQuickAccess());
      }
    };

    window.addEventListener(EVENT_NAME, handleLocalChange);
    window.addEventListener("storage", handleStorageChange);
    return () => {
      window.removeEventListener(EVENT_NAME, handleLocalChange);
      window.removeEventListener("storage", handleStorageChange);
    };
  }, []);

  const top3 = useMemo(() => allIds.slice(0, DISPLAY_COUNT), [allIds]);

  const recordAccess = useCallback((appId: string) => {
    const current = getQuickAccess();
    const filtered = current.filter((id) => id !== appId);
    filtered.unshift(appId);
    const next = filtered.slice(0, MAX_ITEMS);
    setQuickAccess(next);
    setAllIds(next);
  }, []);

  return { top3, all: allIds, recordAccess };
}
