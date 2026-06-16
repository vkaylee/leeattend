import { useEffect, useRef } from "react";

/**
 * Custom hook to trap focus within a container element for accessibility (WCAG 2.1 AA).
 * Also restores focus to the previously active element when the container closes.
 */
export function useFocusTrap(isOpen: boolean) {
  const containerRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!isOpen) return;

    // Store the element that had focus before opening the overlay
    previousFocusRef.current = document.activeElement as HTMLElement;

    const container = containerRef.current;
    if (!container) return;

    // Query for all standard focusable elements that are visible and not disabled
    const getFocusableElements = () => {
      const selectors = [
        "a[href]",
        "area[href]",
        "input:not([disabled])",
        "select:not([disabled])",
        "textarea:not([disabled])",
        "button:not([disabled])",
        "iframe",
        "object",
        "embed",
        "[tabindex]:not([tabindex='-1'])",
        "[contenteditable]"
      ];
      
      return Array.from(
        container.querySelectorAll<HTMLElement>(selectors.join(", "))
      ).filter((el) => {
        const rect = el.getBoundingClientRect();
        const isVisible = rect.width > 0 && rect.height > 0;
        const style = window.getComputedStyle(el);
        return isVisible && style.display !== "none" && style.visibility !== "hidden";
      });
    };

    const focusable = getFocusableElements();
    if (focusable.length > 0) {
      // Set initial focus to the first focusable element
      focusable[0].focus();
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Tab") return;

      const focusableElements = getFocusableElements();
      if (focusableElements.length === 0) {
        e.preventDefault();
        return;
      }

      const firstElement = focusableElements[0];
      const lastElement = focusableElements[focusableElements.length - 1];
      const activeElement = document.activeElement;

      if (e.shiftKey) {
        // Shift + Tab: Wrap from first to last
        if (activeElement === firstElement) {
          lastElement.focus();
          e.preventDefault();
        }
      } else {
        // Tab: Wrap from last to first
        if (activeElement === lastElement) {
          firstElement.focus();
          e.preventDefault();
        }
      }
    };

    container.addEventListener("keydown", handleKeyDown);

    return () => {
      container.removeEventListener("keydown", handleKeyDown);
      // Restore focus to the initiating element upon unmount / closure
      if (previousFocusRef.current) {
        previousFocusRef.current.focus();
      }
    };
  }, [isOpen]);

  return containerRef;
}
