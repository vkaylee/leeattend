/* eslint-disable react-refresh/only-export-components -- shared toolkit
 * for IconRail's state-driven tooltip positioning. */
import { useCallback, useLayoutEffect, useState } from "react";
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

// Re-renders when hovered AND element is measurable.
// The rect is read inside the effect, never during render.
export function TooltipPortal({
  el,
  hovered,
  label,
}: {
  el: HTMLElement | null;
  hovered: boolean;
  label: string;
}) {
  const [rect, setRect] = useState<DOMRect | null>(null);

  useLayoutEffect(() => {
    if (!el || !hovered) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setRect(null);
      return;
    }
    const measure = () => {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setRect(el.getBoundingClientRect());
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    window.addEventListener("scroll", measure, true);
    return () => {
      ro.disconnect();
      window.removeEventListener("scroll", measure, true);
    };
  }, [el, hovered]);

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
