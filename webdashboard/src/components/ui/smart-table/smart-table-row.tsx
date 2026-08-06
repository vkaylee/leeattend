import {
  type Row,
  type Column,
  type Cell,
} from "@tanstack/react-table";
import React, { memo, useCallback, useMemo } from "react";

import { cn } from "@/lib/utils";

import type { CellCoordinate, EditableCellValue } from "./types";

import { SmartTableCell } from "./smart-table-cell";

// ── SmartTableRow ────────────────────────────────────────────────────────────

export interface SmartTableRowProps<T> {
  cellErrors: Record<string, string>;
  cellRefs: React.RefObject<Record<string, HTMLTableCellElement | null>>;
  /** Number of visible columns — used as colSpan for the expanded detail row. */
  colCount: number;
  dragCurrentRow: number | null;
  dragStartRow: number | null;
  editingCell: CellCoordinate | null;
  editValue: EditableCellValue;
  focusedCell: CellCoordinate | null;
  getPinStyles: (column: Column<T>) => React.CSSProperties;
  /** Per-row className override, merged after the built-in row classes. */
  getRowClassName?: (row: T, index: number) => string | undefined;
  handleCellKeyDown: (args: { column: Column<T>; e: React.KeyboardEvent; colIndex: number; row: Row<T>; rowIndex: number }) => void;
  handleRowContextMenu: (row: T, e: React.MouseEvent) => void;
  handleRowMouseDown: (index: number, e: React.MouseEvent) => void;
  handleRowMouseEnter: (index: number) => void;
  isDragging: boolean;
  localCellValues: Record<string, EditableCellValue>;
  onRowClick?: (row: T, e: React.MouseEvent) => void;
  /**
   * Renders a full-width detail row directly below this row. SmartTable owns the
   * `<tr>`/`<td colSpan>` wrapper so consumers never hand-maintain a colSpan count.
   * Return a falsy value to render no detail row.
   */
  renderExpandedRow?: (row: T) => React.ReactNode;
  row: Row<T>;
  rowIndex: number;
  saveCellEdit: (rowIndex: number, colIndex: number, column: Column<T>, row: Row<T>) => void;
  selectedRowIds?: Set<string>;
  setEditValue: React.Dispatch<React.SetStateAction<EditableCellValue>>;
  setFocusedCell: React.Dispatch<React.SetStateAction<CellCoordinate | null>>;
  startCellEdit: (rowIndex: number, colIndex: number, value: EditableCellValue) => void;
}

function SmartTableRowInner<T>(props: SmartTableRowProps<T>) {
  const {
    row, rowIndex, selectedRowIds, isDragging, dragStartRow, dragCurrentRow,
    onRowClick, handleRowContextMenu, handleRowMouseDown, handleRowMouseEnter,
    cellErrors, cellRefs, editingCell, editValue, focusedCell, getPinStyles,
    handleCellKeyDown, localCellValues, saveCellEdit, setEditValue,
    setFocusedCell, startCellEdit, getRowClassName, renderExpandedRow, colCount,
  } = props;

  const isSelected = selectedRowIds?.has(row.id) ?? false;
  const isDraggingRange = useMemo(
    () =>
      isDragging &&
      dragStartRow !== null &&
      dragCurrentRow !== null &&
      rowIndex >= Math.min(dragStartRow, dragCurrentRow) &&
      rowIndex <= Math.max(dragStartRow, dragCurrentRow),
    [isDragging, dragStartRow, dragCurrentRow, rowIndex],
  );

  const handleClick = useCallback((e: React.MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.closest("button") || target.closest("select") || target.closest("input") || target.closest("[role='button']")) return;
    onRowClick?.(row.original, e);
  }, [onRowClick, row.original]);

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    handleRowContextMenu(row.original, e);
  }, [handleRowContextMenu, row.original]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    handleRowMouseDown(rowIndex, e);
  }, [handleRowMouseDown, rowIndex]);

  const handleMouseEnter = useCallback(() => {
    handleRowMouseEnter(rowIndex);
  }, [handleRowMouseEnter, rowIndex]);

  const expandedContent = renderExpandedRow?.(row.original);

  return (
    <React.Fragment>
      <tr
        className={cn(
          "group/row hover:bg-slate-50/50 dark:hover:bg-slate-800/30 transition-colors cursor-pointer",
          isSelected && "bg-primary-50/20 dark:bg-primary-950/10",
          isDraggingRange && "bg-primary-50/30 dark:bg-primary-950/20 outline outline-2 outline-primary-300 dark:outline-primary-800 outline-offset-[-2px] z-10 relative",
          getRowClassName?.(row.original, rowIndex),
        )}
        onClick={handleClick}
        onContextMenu={handleContextMenu}
        onMouseDown={handleMouseDown}
        onMouseEnter={handleMouseEnter}
      >
        {row.getVisibleCells().map((cell: Cell<T, unknown>, colIndex: number) => {
          const errorKey = `${row.id}-${cell.column.id}`;
          return (
            <SmartTableCell
              cell={cell}
              cellError={cellErrors[errorKey]}
              cellRefs={cellRefs}
              colIndex={colIndex}
              editingCell={editingCell}
              editValue={editValue}
              focusedCell={focusedCell}
              getPinStyles={getPinStyles}
              handleCellKeyDown={handleCellKeyDown}
              key={cell.id}
              localCellValues={localCellValues}
              row={row}
              rowIndex={rowIndex}
              saveCellEdit={saveCellEdit}
              setEditValue={setEditValue}
              setFocusedCell={setFocusedCell}
              startCellEdit={startCellEdit}
            />
          );
        })}
      </tr>
      {expandedContent ? (
        <tr className="bg-slate-500/5 dark:bg-slate-950/10">
          <td colSpan={colCount}>{expandedContent}</td>
        </tr>
      ) : null}
    </React.Fragment>
  );
}

export const SmartTableRow = memo(SmartTableRowInner) as typeof SmartTableRowInner;
