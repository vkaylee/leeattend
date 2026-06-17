import React, { useState, useEffect, useRef } from "react";

// Local inline interface to avoid dependency loops
interface LocalSmartTableColumnMeta {
  editable?: boolean;
  editType?: "text" | "number" | "select";
  editOptions?: { value: string; label: string }[];
  sticky?: "left" | "right";
  headerClassName?: string;
  cellClassName?: string;
}

interface UseSmartTableKeyboardProps {
  rowCount: number;
  colCount: number;
  columns: any[];
  rows: any[];
  validateCell?: (rowId: string, columnId: string, value: any) => string | null | undefined;
  onCellSave?: (rowId: string, columnId: string, value: any) => Promise<void> | void;
  cellErrors: Record<string, string>;
  setCellErrors: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  localCellValues: Record<string, any>;
  setLocalCellValues: React.Dispatch<React.SetStateAction<Record<string, any>>>;
}

export function useSmartTableKeyboard({
  rowCount,
  colCount,
  columns,
  rows,
  validateCell,
  onCellSave,
  cellErrors,
  setCellErrors,
  localCellValues,
  setLocalCellValues,
}: UseSmartTableKeyboardProps) {
  const [focusedCell, setFocusedCell] = useState<{ rowIndex: number; colIndex: number } | null>(null);
  const [editingCell, setEditingCell] = useState<{ rowIndex: number; colIndex: number } | null>(null);
  const [editValue, setEditValue] = useState<any>("");

  const cellRefs = useRef<Record<string, HTMLTableCellElement | null>>({});

  // Focus effect
  useEffect(() => {
    if (focusedCell && !editingCell) {
      const key = `${focusedCell.rowIndex}-${focusedCell.colIndex}`;
      const cellElement = cellRefs.current[key];
      if (cellElement) {
        cellElement.focus();
      }
    }
  }, [focusedCell, editingCell]);

  const moveFocus = (rowIdx: number, colIdx: number) => {
    if (rowIdx >= 0 && rowIdx < rowCount && colIdx >= 0 && colIdx < colCount) {
      setFocusedCell({ rowIndex: rowIdx, colIndex: colIdx });
    }
  };

  const startCellEdit = (rowIndex: number, colIndex: number, value: any) => {
    setEditingCell({ rowIndex, colIndex });
    setEditValue(value);
  };

  const saveCellEdit = (rowIndex: number, colIndex: number, column: any, row: any) => {
    const errorKey = `${row.id}-${column.id}`;
    let error: string | null | undefined = null;
    if (validateCell) {
      error = validateCell(row.id, column.id, editValue);
    }

    if (error) {
      setCellErrors((prev) => ({ ...prev, [errorKey]: error }));
      setLocalCellValues((prev) => ({ ...prev, [errorKey]: editValue }));
    } else {
      setCellErrors((prev) => {
        const copy = { ...prev };
        delete copy[errorKey];
        return copy;
      });
      setLocalCellValues((prev) => {
        const copy = { ...prev };
        delete copy[errorKey];
        return copy;
      });

      if (onCellSave) {
        const result = onCellSave(row.id, column.id, editValue);
        if (result instanceof Promise) {
          result.catch((err) => console.error("Error saving cell:", err));
        }
      }
    }
    setEditingCell(null);
  };

  const handleCellKeyDown = (
    rowIndex: number,
    colIndex: number,
    column: any,
    row: any,
    e: React.KeyboardEvent
  ) => {
    // If editing, handle edit keys
    if (editingCell && editingCell.rowIndex === rowIndex && editingCell.colIndex === colIndex) {
      if (e.key === "Enter") {
        e.preventDefault();
        saveCellEdit(rowIndex, colIndex, column, row);
      } else if (e.key === "Escape") {
        e.preventDefault();
        setEditingCell(null);
      } else if (e.key === "Tab") {
        e.preventDefault();
        saveCellEdit(rowIndex, colIndex, column, row);
        const nextCol = e.shiftKey ? colIndex - 1 : colIndex + 1;
        moveFocus(rowIndex, nextCol);
      }
      return; // Do not process navigation arrows in input fields
    }

    // Grid Navigation Keys
    let nextRow = rowIndex;
    let nextCol = colIndex;
    let handled = false;

    switch (e.key) {
      case "ArrowUp":
        nextRow = Math.max(0, rowIndex - 1);
        handled = true;
        break;
      case "ArrowDown":
        nextRow = Math.min(rowCount - 1, rowIndex + 1);
        handled = true;
        break;
      case "ArrowLeft":
        nextCol = Math.max(0, colIndex - 1);
        handled = true;
        break;
      case "ArrowRight":
        nextCol = Math.min(colCount - 1, colIndex + 1);
        handled = true;
        break;
      case "Tab":
        e.preventDefault();
        if (e.shiftKey) {
          nextCol = colIndex - 1;
          if (nextCol < 0) {
            nextCol = colCount - 1;
            nextRow = Math.max(0, rowIndex - 1);
          }
        } else {
          nextCol = colIndex + 1;
          if (nextCol >= colCount) {
            nextCol = 0;
            nextRow = Math.min(rowCount - 1, rowIndex + 1);
          }
        }
        handled = true;
        break;
      case "Enter":
      case "F2":
        {
          const meta = column.columnDef.meta as LocalSmartTableColumnMeta | undefined;
          if (meta?.editable) {
            e.preventDefault();
            startCellEdit(rowIndex, colIndex, row.getValue(column.id));
          }
        }
        handled = true;
        break;
      default:
        break;
    }

    if (handled) {
      moveFocus(nextRow, nextCol);
    }
  };

  return {
    focusedCell,
    setFocusedCell,
    editingCell,
    setEditingCell,
    editValue,
    setEditValue,
    cellRefs,
    moveFocus,
    startCellEdit,
    saveCellEdit,
    handleCellKeyDown,
  };
}
