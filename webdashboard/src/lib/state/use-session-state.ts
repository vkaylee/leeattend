import { useCallback, useState } from "react";

import { logError } from "@/lib/logger";

type Updater<T> = T | ((prev: T) => T);

function readStored<T>(storage: Storage | null, key: string, fallback: T): T {
  if (!storage) return fallback;
  try {
    const raw = storage.getItem(key);
    if (raw === null) return fallback;
    return JSON.parse(raw) as T;
  } catch (error) {
    logError(error, { component: "useSessionState", extra: { key } });
    return fallback;
  }
}

function writeStored(storage: Storage | null, key: string, value: unknown): void {
  if (!storage) return;
  try {
    storage.setItem(key, JSON.stringify(value));
  } catch (error) {
    logError(error, { component: "useSessionState", extra: { key } });
  }
}

/**
 * Private UI state persisted to `sessionStorage` — survives navigation
 * within the same browser tab, cleared when the tab closes.
 *
 * Use for: scroll positions, form drafts, panel/drawer open state,
 * selected rows, ephemeral toggles.
 *
 * NOT for shareable state (tab, filter, sort) — that belongs on the URL
 * via TanStack Router `validateSearch` + [[use-search-patch]].
 *
 * @param key Storage key. Namespace it with route/page to avoid collisions,
 *   e.g. `attendance-logs:scroll`.
 * @param initial Fallback value when storage is empty or unreadable.
 */
export function useSessionState<T>(key: string, initial: T): [T, (next: Updater<T>) => void] {
  return useStoredState(
    key,
    initial,
    typeof window !== "undefined" ? window.sessionStorage : null,
  );
}

/**
 * Cross-session UI preferences persisted to `localStorage`.
 *
 * Use for: theme, sidebar collapsed, column visibility, density,
 * user-chosen defaults that should survive a browser restart.
 */
export function usePersistentState<T>(key: string, initial: T): [T, (next: Updater<T>) => void] {
  return useStoredState(
    key,
    initial,
    typeof window !== "undefined" ? window.localStorage : null,
  );
}

function useStoredState<T>(
  key: string,
  initial: T,
  storage: Storage | null,
): [T, (next: Updater<T>) => void] {
  const [value, setValue] = useState<T>(() => readStored(storage, key, initial));

  const update = useCallback(
    (next: Updater<T>) => {
      setValue((prev) => {
        const resolved =
          typeof next === "function" ? (next as (p: T) => T)(prev) : next;
        writeStored(storage, key, resolved);
        return resolved;
      });
    },
    [key, storage],
  );

  return [value, update];
}
