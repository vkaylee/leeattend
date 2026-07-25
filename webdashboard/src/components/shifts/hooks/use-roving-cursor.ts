/**
 * Arrow-key roving cursor over a flat list of buttons.
 *
 * Kept separate from `use-shift-picker` because it holds no shift-specific
 * knowledge — it just tracks "which index is highlighted" and keeps that row
 * scrolled into view.
 */

import { useCallback, useEffect, useRef, useState } from "react";

/** Sentinel for "no row is under the cursor yet". */
export const NO_ACTIVE_INDEX = -1;

export interface RovingCursor {
  activeIndex: number;
  /** Wrap-around move; seeds from either end when nothing is active yet. */
  moveCursor: (delta: number) => void;
  registerItemRef: (index: number) => (el: HTMLButtonElement | null) => void;
  reset: () => void;
  setActiveIndex: (index: number) => void;
}

export function useRovingCursor(count: number, resultKey: string): RovingCursor {
  const [activeIndex, setActiveIndex] = useState<number>(NO_ACTIVE_INDEX);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);

  // Drop the cursor whenever the result set changes under it, so Enter never
  // selects a row the user can no longer see.
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- the cursor must follow the result set, and it lives in state because the list renders the highlight from it
    setActiveIndex(NO_ACTIVE_INDEX);
  }, [resultKey]);

  // Keep the highlighted row inside the scroll viewport.
  useEffect(() => {
    if (activeIndex < 0) return;
    itemRefs.current[activeIndex]?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  const registerItemRef = useCallback(
    (index: number) => (el: HTMLButtonElement | null) => {
      itemRefs.current[index] = el;
    },
    [],
  );

  const moveCursor = useCallback(
    (delta: number) => {
      if (count === 0) return;
      setActiveIndex((prev) => {
        if (prev === NO_ACTIVE_INDEX) return delta > 0 ? 0 : count - 1;
        return (prev + delta + count) % count;
      });
    },
    [count],
  );

  const reset = useCallback(() => {
    setActiveIndex(NO_ACTIVE_INDEX);
  }, []);

  return { activeIndex, moveCursor, registerItemRef, reset, setActiveIndex };
}
