import React, { useState, useEffect } from "react";

import { format } from "date-fns";

interface UseDragCellSelectionProps {
  /** Ordered employee schedule rows (for employee-index lookup). */
  filteredSchedules: { employee_id: string }[];
  /** Month day Date objects (for date-index lookup). */
  monthDays: Date[];
  /** Currently selected cells (keyed "employeeId:dateStr"). */
  selectedCells: Set<string>;
  /** Setter for selected cells — called by the drag-paint reducer. */
  setSelectedCells: React.Dispatch<React.SetStateAction<Set<string>>>;
}

interface UseDragCellSelectionReturn {
  /** Keyboard activation handler for accessible mobile cells. */
  handleCellKeyDown: (employeeId: string, dateStr: string, e: React.KeyboardEvent) => void;
  /** PointerDown handler — attach to each grid cell. */
  handleCellPointerDown: (employeeId: string, dateStr: string, e: React.PointerEvent) => void;
  /** PointerEnter handler — attach to each grid cell (no-op when not dragging). */
  handleCellPointerEnter: (employeeId: string, dateStr: string) => void;
  /** Whether a drag-paint operation is currently in progress. */
  isDragging: boolean;
}

/**
 * Drag-paint cell selection for the monthly roster grid.
 *
 * When the user presses the primary pointer button on a cell and drags across
 * the grid, this hook computes the rectangle between the anchor cell and the
 * current cell and adds or removes all enclosed cells from the selection.
 *
 * The paint direction (add vs remove) is decided by the anchor cell: if the
 * anchor is already selected, the drag erases; otherwise it paints.
 */
export function useDragCellSelection({
  selectedCells,
  setSelectedCells,
  filteredSchedules,
  monthDays,
}: UseDragCellSelectionProps): UseDragCellSelectionReturn {
  const [dragStartCell, setDragStartCell] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragMode, setDragMode] = useState<"add" | "remove">("add");
  const [dragSnapshot, setDragSnapshot] = useState<Set<string>>(new Set());

  // Global pointerup listener to end the drag regardless of where the pointer
  // is released (may be outside the grid).
  useEffect(() => {
    const handleGlobalPointerUp = () => {
      setIsDragging(false);
      setDragStartCell(null);
    };

    if (isDragging) {
      window.addEventListener("pointerup", handleGlobalPointerUp);
      window.addEventListener("pointercancel", handleGlobalPointerUp);
      return () => {
        window.removeEventListener("pointerup", handleGlobalPointerUp);
        window.removeEventListener("pointercancel", handleGlobalPointerUp);
      };
    }
  }, [isDragging]);

  const toggleCell = (
    employeeId: string,
    dateStr: string,
    startDrag: boolean,
  ) => {
    const key = `${employeeId}:${dateStr}`;
    const mode: "add" | "remove" = selectedCells.has(key) ? "remove" : "add";
    if (startDrag) {
      setIsDragging(true);
      setDragStartCell(key);
      setDragMode(mode);
    }

    setSelectedCells((prev) => {
      const next = new Set(prev);
      if (mode === "remove") {
        next.delete(key);
      } else {
        next.add(key);
      }
      if (startDrag) setDragSnapshot(new Set(next));
      return next;
    });
  };

  const handleCellPointerDown = (
    employeeId: string,
    dateStr: string,
    e: React.PointerEvent,
  ) => {
    // Only primary button (left mouse / single touch / pen contact)
    if (e.button !== 0) return;
    toggleCell(employeeId, dateStr, true);
  };

  const handleCellKeyDown = (
    employeeId: string,
    dateStr: string,
    e: React.KeyboardEvent,
  ) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    toggleCell(employeeId, dateStr, false);
  };

  const handleCellPointerEnter = (employeeId: string, dateStr: string) => {
    if (!isDragging || !dragStartCell) return;

    const [startEmpId, startDateStr] = dragStartCell.split(":");

    const startEmpIdx = filteredSchedules.findIndex(
      (s) => s.employee_id === startEmpId,
    );
    const endEmpIdx = filteredSchedules.findIndex(
      (s) => s.employee_id === employeeId,
    );

    if (startEmpIdx === -1 || endEmpIdx === -1) return;

    const startDayIdx = monthDays.findIndex(
      (d) => format(d, "yyyy-MM-dd") === startDateStr,
    );
    const endDayIdx = monthDays.findIndex(
      (d) => format(d, "yyyy-MM-dd") === dateStr,
    );

    if (startDayIdx === -1 || endDayIdx === -1) return;

    const minEmpIdx = Math.min(startEmpIdx, endEmpIdx);
    const maxEmpIdx = Math.max(startEmpIdx, endEmpIdx);
    const minDayIdx = Math.min(startDayIdx, endDayIdx);
    const maxDayIdx = Math.max(startDayIdx, endDayIdx);

    setSelectedCells(() => {
      const next = new Set(dragSnapshot);
      for (let i = minEmpIdx; i <= maxEmpIdx; i++) {
        const empId = filteredSchedules[i].employee_id;
        for (let j = minDayIdx; j <= maxDayIdx; j++) {
          const dStr = format(monthDays[j], "yyyy-MM-dd");
          const cellKey = `${empId}:${dStr}`;
          if (dragMode === "remove") {
            next.delete(cellKey);
          } else {
            next.add(cellKey);
          }
        }
      }
      return next;
    });
  };

  return {
    isDragging,
    handleCellKeyDown,
    handleCellPointerDown,
    handleCellPointerEnter,
  };
}
