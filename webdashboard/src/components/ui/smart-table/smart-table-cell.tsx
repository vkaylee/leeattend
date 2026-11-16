import {
  flexRender,
  type Row,
  type Column,
  type Cell,
} from "@tanstack/react-table";
import React, { memo, useCallback, useMemo } from "react";

import { logError } from "@/lib/logger";
import { cn } from "@/lib/utils";
import { maybeNull } from "@/types/api-overrides";

import type { TableDensity } from "../table-density-toggle";
import type { CellCoordinate, EditableCellValue } from "./types";

import CellEditControl from "./cell-edit-control";
import CellErrorTooltip from "./cell-error-tooltip";

interface SmartTableCellProps<T> {
  cell: Cell<T, unknown>;
  cellError?: string;
  cellRefs: React.RefObject<Record<string, HTMLTableCellElement | null>>;
  colIndex: number;
  density?: TableDensity;
  editingCell: CellCoordinate | null;
  editValue: EditableCellValue;
  focusedCell: CellCoordinate | null;
  getPinStyles: (column: Column<T>) => React.CSSProperties;
  handleCellKeyDown: (args: { column: Column<T>; e: React.KeyboardEvent; colIndex: number; row: Row<T>; rowIndex: number }) => void;
  localCellValues: Record<string, EditableCellValue>;
  row: Row<T>;
  rowIndex: number;
  saveCellEdit: (rowIndex: number, colIndex: number, column: Column<T>, row: Row<T>) => void;
  setEditValue: React.Dispatch<React.SetStateAction<EditableCellValue>>;
  setFocusedCell: React.Dispatch<React.SetStateAction<CellCoordinate | null>>;
  startCellEdit: (rowIndex: number, colIndex: number, value: EditableCellValue) => void;
}

// eslint-disable-next-line complexity -- table cell rendering inherently handles pinned/editing/error/focus/drag states
function SmartTableCellInner<T>({
  cell,
  cellError,
  density = "normal",
  cellRefs,
  colIndex,
  editingCell,
  editValue,
  focusedCell,
  getPinStyles,
  handleCellKeyDown,
  localCellValues,
  row,
  rowIndex,
  saveCellEdit,
  setEditValue,
  setFocusedCell,
  startCellEdit,
}: SmartTableCellProps<T>) {
  const isCellFocused = maybeNull(focusedCell)?.rowIndex === rowIndex && maybeNull(focusedCell)?.colIndex === colIndex;
  const isCellEditing = maybeNull(editingCell)?.rowIndex === rowIndex && maybeNull(editingCell)?.colIndex === colIndex;
  const isPinned = cell.column.getIsPinned();
  const isLastPinnedLeft = isPinned === "left" && cell.column.getIsLastColumn("left");
  const isFirstPinnedRight = isPinned === "right" && cell.column.getIsFirstColumn("right");
  const meta = cell.column.columnDef.meta;
  const hasError = !!cellError;

  // Stable callbacks to avoid re-renders in memoized children
  const handleClick = useCallback(() => {
    setFocusedCell({ rowIndex, colIndex });
  }, [setFocusedCell, rowIndex, colIndex]);

  const handleDoubleClick = useCallback(() => {
    if (meta?.editable) {
      startCellEdit(rowIndex, colIndex, row.getValue(cell.column.id));
    }
  }, [meta?.editable, startCellEdit, rowIndex, colIndex, row, cell.column.id]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    handleCellKeyDown({ column: cell.column, e, colIndex, row, rowIndex });
  }, [handleCellKeyDown, cell.column, colIndex, row, rowIndex]);

  const cellRefCallback = useCallback((el: HTMLTableCellElement | null) => {
    const key = `${rowIndex}-${colIndex}`;
    // ref-based keyboard nav requires mutable storage via .current assignment
    const refs = cellRefs.current;
    // eslint-disable-next-line react-hooks/immutability -- cellRefs is a stable ref; mutation of .current is intentional
    if (maybeNull(refs)) refs[key] = el;
  // eslint-disable-next-line react-hooks/exhaustive-deps -- cellRefs is a stable ref; .current is intentionally excluded to match React Compiler inference
  }, [rowIndex, colIndex]);

  const handleEditBlur = useCallback(() => {
    saveCellEdit(rowIndex, colIndex, cell.column, row);
  }, [saveCellEdit, rowIndex, colIndex, cell.column, row]);

  // Override cell value with local overridden value on validation error
  if (hasError) {
    const errorKey = `${row.id}-${cell.column.id}`;
    const overrideValue = localCellValues[errorKey];
    try {
      // eslint-disable-next-line react-hooks/immutability -- intentional mutation to display error-overridden values
      cell.getValue = () => overrideValue as never;
      // @ts-expect-error -- TanStack Table Cell type does not expose renderValue, but we
      // need to override it to display error-highlighted cell values in validation mode.
      // eslint-disable-next-line react-hooks/immutability
      cell.renderValue = () => overrideValue;
    } catch (e: unknown) {
      // Revert partial cell overrides on mutation failure to keep consistent state
      try {
        // eslint-disable-next-line react-hooks/immutability
        cell.getValue = (() => row.getValue(cell.column.id));
      } catch (e: unknown) { logError(e, { component: 'SmartTableCell' }); /* best-effort cleanup */ }
      console.warn("Could not override cell.getValue", e instanceof Error ? e.message : "Unknown error");
    }
  }

  const densityClass = density === "compact"
    ? "px-3 py-1.5 text-xs"
    : density === "comfortable"
      ? "px-6 py-4 text-sm"
      : "px-5 py-2.5 text-sm";
  const cellClass = useMemo(() => cn(
    "group relative font-medium text-slate-700 dark:text-slate-200 border-r border-slate-100 dark:border-slate-800 last:border-r-0 truncate outline-none",
    densityClass,
    isPinned && "bg-white dark:bg-slate-900 group-hover/row:bg-slate-50/50 dark:group-hover/row:bg-slate-800/30",
    isCellFocused && (hasError ? "ring-2 ring-rose-500 ring-inset z-10" : "ring-2 ring-primary-500 ring-inset z-10"),
    isCellEditing && "p-0.5",
    isLastPinnedLeft && "border-r border-r-slate-200 dark:border-r-slate-700 shadow-[2px_0_5px_-2px_rgba(0,0,0,0.1)]",
    isFirstPinnedRight && "border-l border-l-slate-200 dark:border-l-slate-700 shadow-[-2px_0_5px_-2px_rgba(0,0,0,0.1)]",
    hasError && "border border-rose-500 bg-rose-50/20 dark:bg-rose-950/25 focus-visible:ring-rose-500",
    meta?.cellClassName,
  ), [densityClass, isPinned, isCellFocused, isCellEditing, isLastPinnedLeft, isFirstPinnedRight, hasError, meta?.cellClassName]);

  return (
    <td
      className={cellClass}
      key={cell.id}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      onKeyDown={handleKeyDown}
      ref={cellRefCallback}
      style={{ width: cell.column.getSize(), ...getPinStyles(cell.column) }}
      tabIndex={isCellFocused ? 0 : -1}
    >
      {isCellEditing
        ? (
          <CellEditControl
            editOptions={meta?.editOptions}
            editType={meta?.editType}
            editValue={editValue}
            onBlur={handleEditBlur}
            setEditValue={setEditValue}
          />
        )
        : flexRender(cell.column.columnDef.cell, cell.getContext())}

      {hasError ? <CellErrorTooltip cellError={cellError} columnId={cell.column.id} rowId={row.id} /> : null}
    </td>
  );
}

// Cast needed because React.memo drops the generic type parameter
const SmartTableCell = memo(SmartTableCellInner) as typeof SmartTableCellInner;

export { SmartTableCell };
export type { SmartTableCellProps };
