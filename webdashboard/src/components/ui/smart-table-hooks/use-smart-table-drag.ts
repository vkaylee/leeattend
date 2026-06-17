import React, { useState, useEffect } from "react";

interface UseSmartTableDragProps {
  enableDragSelect: boolean;
  selectedRowIds?: Set<string>;
  rows: any[];
  onSelectionChange?: (selectedIds: Set<string>) => void;
}

export function useSmartTableDrag({
  enableDragSelect,
  selectedRowIds,
  rows,
  onSelectionChange,
}: UseSmartTableDragProps) {
  const [dragStartRow, setDragStartRow] = useState<number | null>(null);
  const [dragCurrentRow, setDragCurrentRow] = useState<number | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  // Global mouseup to stop dragging
  useEffect(() => {
    if (!enableDragSelect) return;
    const handleMouseUp = () => {
      if (isDragging && dragStartRow !== null && dragCurrentRow !== null && onSelectionChange) {
        const start = Math.min(dragStartRow, dragCurrentRow);
        const end = Math.max(dragStartRow, dragCurrentRow);
        const newSelection = new Set(selectedRowIds);
        
        for (let i = start; i <= end; i++) {
          const row = rows[i];
          if (row) {
            newSelection.add(row.id);
          }
        }
        onSelectionChange(newSelection);
      }
      setIsDragging(false);
      setDragStartRow(null);
      setDragCurrentRow(null);
    };

    window.addEventListener("mouseup", handleMouseUp);
    return () => window.removeEventListener("mouseup", handleMouseUp);
  }, [isDragging, dragStartRow, dragCurrentRow, rows, selectedRowIds, onSelectionChange, enableDragSelect]);

  const handleRowMouseDown = (rowIndex: number, e: React.MouseEvent) => {
    if (!enableDragSelect || e.button !== 0) return; // Only left click
    setIsDragging(true);
    setDragStartRow(rowIndex);
    setDragCurrentRow(rowIndex);
  };

  const handleRowMouseEnter = (rowIndex: number) => {
    if (isDragging) {
      setDragCurrentRow(rowIndex);
    }
  };

  return {
    isDragging,
    dragStartRow,
    dragCurrentRow,
    handleRowMouseDown,
    handleRowMouseEnter,
  };
}
