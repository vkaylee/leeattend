import { useEffect, useRef } from "react";

interface UseMonthlyRosterKeyboardProps {
  /** Current selection — shortcuts are suppressed when empty. */
  selectedCells: Set<string>;
  /** Open the shift picker modal. */
  setIsShiftPickerOpen: (open: boolean) => void;
  /** Clear the current cell selection. */
  setSelectedCells: (cells: Set<string>) => void;
  /** Clear roster overrides for the selected cells. */
  onClearOverrides: () => void;
}

/**
 * Registers keyboard shortcuts for the monthly roster grid when cells are
 * selected:
 * - `A` → open the shift picker to assign/replace shifts
 * - `Delete` / `Backspace` → clear roster overrides for selected cells
 * - `Escape` → deselect all cells
 *
 * Shortcuts are suppressed when focus is inside an input, textarea, or
 * contenteditable element so they don't interfere with text editing.
 */
export function useMonthlyRosterKeyboard({
  selectedCells,
  setIsShiftPickerOpen,
  setSelectedCells,
  onClearOverrides,
}: UseMonthlyRosterKeyboardProps): void {
  // Stable callback refs so the effect doesn't need them in the dep array.
  const onClearRef = useRef(onClearOverrides);
  onClearRef.current = onClearOverrides;

  useEffect(() => {
    if (selectedCells.size === 0) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      const activeEl = document.activeElement;
      if (
        activeEl &&
        (activeEl.tagName === "INPUT" ||
          activeEl.tagName === "TEXTAREA" ||
          activeEl.getAttribute("contenteditable") === "true")
      ) {
        return;
      }

      if (e.key === "a" || e.key === "A") {
        e.preventDefault();
        setIsShiftPickerOpen(true);
      } else if (e.key === "Delete" || e.key === "Backspace") {
        e.preventDefault();
        onClearRef.current();
      } else if (e.key === "Escape") {
        e.preventDefault();
        setSelectedCells(new Set());
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [selectedCells.size, setIsShiftPickerOpen, setSelectedCells]);
}
