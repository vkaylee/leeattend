import type React from "react";

import { useCallback, useEffect, useRef } from "react";

const DRAG_THRESHOLD_PX = 8;
const INTERACTIVE_SELECTOR = [
  "a",
  "button",
  "input",
  "select",
  "textarea",
  "[contenteditable=\"true\"]",
  "[role=\"button\"]",
  "[draggable=\"true\"]",
  "[data-no-horizontal-drag-scroll]",
].join(",");

interface DragState {
  active: boolean;
  initialScrollLeft: number;
  originalCursor: string;
  originalUserSelect: string;
  pointerId: number;
  startX: number;
  startY: number;
}

function isInteractiveTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(INTERACTIVE_SELECTOR) !== null;
}

function isScrollbarTarget(element: HTMLElement, event: PointerEvent): boolean {
  const rect = element.getBoundingClientRect();
  const localX = event.clientX - rect.left;
  const localY = event.clientY - rect.top;
  const verticalScrollbarWidth = element.offsetWidth - element.clientWidth;
  const horizontalScrollbarHeight = element.offsetHeight - element.clientHeight;

  return (
    localX >= element.clientWidth - verticalScrollbarWidth && verticalScrollbarWidth > 0
  ) || (
    localY >= element.clientHeight - horizontalScrollbarHeight && horizontalScrollbarHeight > 0
  );
}

function hasHorizontalOverflow(element: HTMLElement): boolean {
  return element.scrollWidth > element.clientWidth;
}

export interface HorizontalDragScrollOptions {
  enabled?: boolean;
}

export interface HorizontalDragScrollProps<T extends HTMLElement> {
  onClickCapture: React.MouseEventHandler<T>;
  setRef: (element: T | null) => void;
}

/**
 * Adds mouse-only horizontal panning without replacing native scroll behavior.
 * The gesture stays a normal pointer interaction until horizontal intent is clear.
 */
export function useHorizontalDragScroll<T extends HTMLElement>(
  { enabled = true }: HorizontalDragScrollOptions = {},
): HorizontalDragScrollProps<T> {
  const elementRef = useRef<T | null>(null);
  const dragRef = useRef<DragState | null>(null);
  const suppressClickRef = useRef(false);

  useEffect(() => {
    const element = elementRef.current;
    if (!enabled || !element) return;

    const restoreInteractionStyles = (drag: DragState) => {
      element.style.cursor = drag.originalCursor;
      element.style.userSelect = drag.originalUserSelect;
    };

    const finishDrag = (event?: PointerEvent) => {
      const drag = dragRef.current;
      if (!drag) return;
      if (event && event.pointerId !== drag.pointerId) return;

      if (drag.active) {
        suppressClickRef.current = true;
        if (element.hasPointerCapture(drag.pointerId)) {
          element.releasePointerCapture(drag.pointerId);
        }
      }
      restoreInteractionStyles(drag);
      dragRef.current = null;
    };

    const handlePointerDown = (event: PointerEvent) => {
      suppressClickRef.current = false;
      if (
        event.pointerType !== "mouse" ||
        event.button !== 0 ||
        !event.isPrimary ||
        event.shiftKey ||
        event.ctrlKey ||
        event.metaKey ||
        event.altKey ||
        isInteractiveTarget(event.target) ||
        isScrollbarTarget(element, event) ||
        !hasHorizontalOverflow(element)
      ) {
        return;
      }

      dragRef.current = {
        active: false,
        initialScrollLeft: element.scrollLeft,
        originalCursor: element.style.cursor,
        originalUserSelect: element.style.userSelect,
        pointerId: event.pointerId,
        startX: event.clientX,
        startY: event.clientY,
      };
    };

    const handlePointerMove = (event: PointerEvent) => {
      const drag = dragRef.current;
      if (event.pointerId !== drag?.pointerId) return;

      const deltaX = event.clientX - drag.startX;
      const deltaY = event.clientY - drag.startY;
      if (!drag.active) {
        if (Math.max(Math.abs(deltaX), Math.abs(deltaY)) < DRAG_THRESHOLD_PX) return;
        if (Math.abs(deltaX) <= Math.abs(deltaY)) {
          dragRef.current = null;
          return;
        }

        drag.active = true;
        element.setPointerCapture(event.pointerId);
        element.style.cursor = "grabbing";
        element.style.userSelect = "none";
      }

      event.preventDefault();
      const maximumScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
      element.scrollLeft = Math.min(
        maximumScrollLeft,
        Math.max(0, drag.initialScrollLeft - deltaX),
      );
    };

    const handleLostPointerCapture = (event: PointerEvent) => {
      finishDrag(event);
    };

    const handlePointerUp = (event: PointerEvent) => {
      finishDrag(event);
    };

    const handleWindowBlur = () => {
      finishDrag();
    };

    element.addEventListener("pointerdown", handlePointerDown);
    element.addEventListener("lostpointercapture", handleLostPointerCapture);
    window.addEventListener("pointermove", handlePointerMove, { passive: false });
    window.addEventListener("pointerup", handlePointerUp);
    window.addEventListener("pointercancel", handlePointerUp);
    window.addEventListener("blur", handleWindowBlur);

    return () => {
      finishDrag();
      element.removeEventListener("pointerdown", handlePointerDown);
      element.removeEventListener("lostpointercapture", handleLostPointerCapture);
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
      window.removeEventListener("pointercancel", handlePointerUp);
      window.removeEventListener("blur", handleWindowBlur);
    };
  }, [enabled]);

  const handleClickCapture = useCallback<React.MouseEventHandler<T>>((event) => {
    if (!suppressClickRef.current) return;
    suppressClickRef.current = false;
    event.preventDefault();
    event.stopPropagation();
  }, []);

  const setRef = useCallback((element: T | null) => {
    elementRef.current = element;
  }, []);

  return { onClickCapture: handleClickCapture, setRef };
}
