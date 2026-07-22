/* eslint-disable react-refresh/only-export-components -- shared toolkit
 * for IconRail's state-driven tooltip positioning. */
import { useCallback, useRef, useSyncExternalStore, useState } from "react";
import { createPortal } from "react-dom";

// ── Shared: observe element rect without reading ref.current in render ──

// Stores the latest element in render-state via a callback ref, so we never
// touch ref.current during render (which trips react-hooks/refs).
export function useTooltipTarget<T extends HTMLElement>() {
  const [el, setEl] = useState<T | null>(null);
  const [hovered, setHovered] = useState(false);
  const refCb = useCallback((node: T | null) => { setEl(node); }, []);
  return { el, hovered, setHovered, refCb } as const;
}

/**
 * Observes an element's bounding rect via ResizeObserver + scroll events.
 * Uses useSyncExternalStore to avoid setState-in-useEffect (DOM measurement
 * is a legitimate side effect — useSyncExternalStore is the canonical React
 * pattern for external store subscriptions like DOM rect observation).
 */
function useElementRect(el: HTMLElement | null, active: boolean): DOMRect | null {
  const snapshotRef = useRef<DOMRect | null>(null);

  const getSnapshot = useCallback((): DOMRect | null => {
    if (!el || !active) return null;

    const next = el.getBoundingClientRect();
    const previous = snapshotRef.current;
    if (
      previous?.x === next.x
      && previous.y === next.y
      && previous.width === next.width
      && previous.height === next.height
    ) {
      return previous;
    }

    snapshotRef.current = next;
    return next;
  }, [el, active]);

  const subscribe = useCallback(
    (onStoreChange: () => void) => {
      if (!el || !active) {
        return () => {};
      }
      const measure = () => { onStoreChange(); };
      const ro = new ResizeObserver(measure);
      ro.observe(el);
      window.addEventListener("scroll", measure, true);
      return () => {
        ro.disconnect();
        window.removeEventListener("scroll", measure, true);
      };
    },
    [el, active],
  );

  return useSyncExternalStore(subscribe, getSnapshot);
}

// Re-renders when hovered AND element is measurable.
// DOM rect is observed via useSyncExternalStore — no setState-in-effect.
export function TooltipPortal({
  el,
  hovered,
  label,
}: {
  el: HTMLElement | null;
  hovered: boolean;
  label: string;
}) {
  const rect = useElementRect(el, hovered);

  if (!el || !hovered || !rect) return null;

  return createPortal(
    <span
      className="fixed px-3 py-1.5 bg-slate-800 dark:bg-slate-200 text-white dark:text-slate-800 text-xs font-bold rounded-xl whitespace-nowrap pointer-events-none shadow-lg z-[60]"
      role="tooltip"
      style={{
        left: rect.right + 12,
        top: rect.top + rect.height / 2,
        transform: "translateY(-50%)",
      }}
    >
      {label}
    </span>,
    document.body,
  );
}
