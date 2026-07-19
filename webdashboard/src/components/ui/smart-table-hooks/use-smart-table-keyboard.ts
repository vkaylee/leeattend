import { type Row } from "@tanstack/react-table";
import React, { useState, useEffect, useRef } from "react";

// Local inline interface to avoid dependency loops
interface LocalSmartTableColumnMeta {
  cellClassName?: string;
  editable?: boolean;
  editOptions?: { value: string; label: string }[];
  editType?: "text" | "number" | "select";
  headerClassName?: string;
  sticky?: "left" | "right";
}

interface UseSmartTableKeyboardProps {
  colCount: number;
  onCellSave?: (rowId: string, columnId: string, value: unknown) => Promise<void> | void;
  rowCount: number;
  setCellErrors: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  setLocalCellValues: React.Dispatch<React.SetStateAction<Record<string, unknown>>>;
  validateCell?: (rowId: string, columnId: string, value: unknown) => string | null | undefined;
}

// eslint-disable-next-line max-lines-per-function -- keyboard navigation requires substantial key-binding logic
export function useSmartTableKeyboard<TData>({
  rowCount,
  colCount,
  validateCell,
  onCellSave,
  setCellErrors,
  setLocalCellValues,
}: UseSmartTableKeyboardProps) {
  const [focusedCell, setFocusedCell] = useState<{ rowIndex: number; colIndex: number } | null>(null);
  const [editingCell, setEditingCell] = useState<{ rowIndex: number; colIndex: number } | null>(null);
  const [editValue, setEditValue] = useState<unknown>("");

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

  const startCellEdit = (rowIndex: number, colIndex: number, value: unknown) => {
    setEditingCell({ rowIndex, colIndex });
    setEditValue(value);
  };

  const saveCellEdit = (
    rowIndex: number,
    colIndex: number,
    column: { id: string; columnDef: { meta?: LocalSmartTableColumnMeta } },
    row: Row<TData>
  ) => {
    const errorKey = `${row.id}-${column.id}`;
    let error: string | null | undefined = null;
    if (validateCell) {
      error = validateCell(row.id, column.id, editValue);
    }

    if (error) {
      setCellErrors((prev: Record<string, string>) => ({ ...prev, [errorKey]: error }));
      setLocalCellValues((prev: Record<string, unknown>) => ({ ...prev, [errorKey]: editValue }));
    } else {
      setCellErrors((prev: Record<string, string>) => {
        const { [errorKey]: _, ...rest } = prev;
        return rest;
      });
      setLocalCellValues((prev: Record<string, unknown>) => {
        const { [errorKey]: _, ...rest } = prev;
        return rest;
      });

      if (onCellSave) {
        const result = onCellSave(row.id, column.id, editValue);
        if (result instanceof Promise) {
          result.catch((err: unknown) => {   console.warn("Error saving cell:", err); });
        }
      }
    }
    setEditingCell(null);
  };

  const handleNavigationKey = (
    key: string,
    rowIndex: number,
    colIndex: number,
    shiftKey: boolean
  ): { nextCol: number; nextRow: number; handled: boolean } => {
    let nextRow = rowIndex;
    let nextCol = colIndex;
    let handled = false;

    switch (key) {
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
        if (shiftKey) {
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
      default:
        break;
    }
    return { nextCol, nextRow, handled };
  };

  const handleCellKeyDown = (args: {
    column: { id: string; columnDef: { meta?: LocalSmartTableColumnMeta } };
    e: React.KeyboardEvent;
    colIndex: number;
    row: Row<TData>;
    rowIndex: number;
  // eslint-disable-next-line complexity -- keyboard navigation has inherent branching for arrow keys, tab, enter, escape
  }) => {
    const { rowIndex, colIndex, column, row, e } = args;

    // If editing, handle edit keys
    if (editingCell?.rowIndex === rowIndex && editingCell.colIndex === colIndex) {
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

    if (e.key === "Enter" || e.key === "F2") {
      const meta = column.columnDef.meta;
      if (meta?.editable) {
        e.preventDefault();
        startCellEdit(rowIndex, colIndex, row.getValue(column.id));
      }
      return;
    }

    const { nextRow, nextCol, handled } = handleNavigationKey(e.key, rowIndex, colIndex, e.shiftKey);
    if (handled) {
      if (e.key === "Tab") {
        e.preventDefault();
      }
      moveFocus(nextRow, nextCol);
    }
  };

  return {
    focusedCell,
    setFocusedCell,
    editingCell,
    editValue,
    setEditValue,
    cellRefs,
    moveFocus,
    startCellEdit,
    saveCellEdit,
    handleCellKeyDown,
  };
}
