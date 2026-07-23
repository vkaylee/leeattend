import {
  flexRender,
  type Row,
  type Column,
  type Cell,
} from "@tanstack/react-table";
import React, { useEffect, useRef } from "react";

import { cn } from "@/lib/utils";

// ── CellEditControl ──────────────────────────────────────────────────────────

interface CellEditControlProps {
  editOptions?: { value: string; label: string }[];
  editType?: "text" | "number" | "select";
  editValue: unknown;
  onBlur: () => void;
  setEditValue: (val: unknown) => void;
}

function CellEditControl({
  editType,
  editValue,
  setEditValue,
  editOptions,
  onBlur,
}: CellEditControlProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const selectRef = useRef<HTMLSelectElement>(null);

  useEffect(() => {
    if (editType === "select") {
      selectRef.current?.focus();
    } else {
      inputRef.current?.focus();
    }
  }, [editType]);

  if (editType === "select") {
    return (
      <select
        className="w-full h-full bg-white dark:bg-slate-800 text-sm border-none outline-none ring-2 ring-primary-500 rounded px-2 py-1.5 focus:ring-primary-500"
        onBlur={onBlur}
        onChange={(e: React.ChangeEvent<HTMLSelectElement>) => { setEditValue(e.target.value); }}
        ref={selectRef}
        value={editValue as string}
      >
        {editOptions?.map((opt) => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
    );
  }

  return (
    <input
      className="w-full h-full bg-white dark:bg-slate-800 text-sm border-none outline-none ring-2 ring-primary-500 rounded px-2 py-1.5 focus:ring-primary-500 font-medium"
      onBlur={onBlur}
      onChange={(e: React.ChangeEvent<HTMLInputElement>) => { setEditValue(e.target.value); }}
      ref={inputRef}
      type={editType === "number" ? "number" : "text"}
      value={(editValue as string | undefined) ?? ""}
    />
  );
}

// ── CellErrorTooltip ────────────────────────────────────────────────────────

function CellErrorTooltip({ cellError, rowId, columnId }: { cellError: string; rowId: string; columnId: string }) {
  return (
    <>
      <span className="absolute top-1.5 right-1.5 w-2 h-2 rounded-full bg-rose-500 animate-pulse pointer-events-none" />
      <div
        className="absolute hidden group-hover:block group-focus-within:block bottom-full left-1/2 -translate-x-1/2 mb-2 z-30 px-2.5 py-1.5 text-xs font-semibold text-white bg-rose-600 dark:bg-rose-700 rounded-lg shadow-lg whitespace-nowrap pointer-events-none transition-all duration-200"
        id={`error-${rowId}-${columnId}`}
        role="tooltip"
      >
        {cellError}
        <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-1 border-4 border-transparent border-t-rose-600 dark:border-t-rose-700" />
      </div>
    </>
  );
}

// ── SmartTableCell ──────────────────────────────────────────────────────────

interface SmartTableCellProps<T> {
  cell: Cell<T, unknown>;
  cellError?: string;
  // RefObject.current is mutable in React 19 types — no need for MutableRefObject
  cellRefs: React.RefObject<Record<string, HTMLTableCellElement | null>>;
  colIndex: number;
  editingCell: { rowIndex: number; colIndex: number } | null;
  editValue: unknown;
  focusedCell: { rowIndex: number; colIndex: number } | null;
  getPinStyles: (column: Column<T>) => React.CSSProperties;
  handleCellKeyDown: (args: { column: Column<T>; e: React.KeyboardEvent; colIndex: number; row: Row<T>; rowIndex: number }) => void;
  localCellValues: Record<string, unknown>;
  row: Row<T>;
  rowIndex: number;
  saveCellEdit: (rowIndex: number, colIndex: number, column: Column<T>, row: Row<T>) => void;
  setEditValue: React.Dispatch<React.SetStateAction<unknown>>;
  setFocusedCell: React.Dispatch<React.SetStateAction<{ rowIndex: number; colIndex: number } | null>>;
  startCellEdit: (rowIndex: number, colIndex: number, value: unknown) => void;
}

// eslint-disable-next-line complexity -- table cell rendering inherently handles pinned/editing/error/focus/drag states
function SmartTableCell<T>({
  cell,
  cellError,
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
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- focusedCell is null when no cell has focus
  const isCellFocused = focusedCell?.rowIndex === rowIndex && focusedCell?.colIndex === colIndex;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- editingCell is null when not editing
  const isCellEditing = editingCell?.rowIndex === rowIndex && editingCell?.colIndex === colIndex;
  const isPinned = cell.column.getIsPinned();
  const isLastPinnedLeft = isPinned === "left" && cell.column.getIsLastColumn("left");
  const isFirstPinnedRight = isPinned === "right" && cell.column.getIsFirstColumn("right");
  const meta = cell.column.columnDef.meta;
  const hasError = !!cellError;

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
    } catch (e) {
      console.warn("Could not override cell.getValue", e);
    }
  }

  const cellClass = cn(
    "group relative px-6 py-4 text-sm font-medium text-slate-700 dark:text-slate-200 border-r border-slate-100 dark:border-slate-800 last:border-r-0 truncate outline-none",
    isPinned && "bg-white dark:bg-slate-900 group-hover/row:bg-slate-50/50 dark:group-hover/row:bg-slate-800/30",
    isCellFocused && (hasError ? "ring-2 ring-rose-500 ring-inset z-10" : "ring-2 ring-primary-500 ring-inset z-10"),
    isCellEditing && "p-0.5",
    isLastPinnedLeft && "border-r border-r-slate-200 dark:border-r-slate-700 shadow-[2px_0_5px_-2px_rgba(0,0,0,0.1)]",
    isFirstPinnedRight && "border-l border-l-slate-200 dark:border-l-slate-700 shadow-[-2px_0_5px_-2px_rgba(0,0,0,0.1)]",
    hasError && "border border-rose-500 bg-rose-50/20 dark:bg-rose-950/25 focus-visible:ring-rose-500",
    meta?.cellClassName,
  );

  return (
    <td
      className={cellClass}
      key={cell.id}
      onClick={() => { setFocusedCell({ rowIndex, colIndex }); }}
      onDoubleClick={() => {
        if (meta?.editable) {
          startCellEdit(rowIndex, colIndex, row.getValue(cell.column.id));
        }
      }}
      onKeyDown={(e: React.KeyboardEvent) => {
        handleCellKeyDown({ column: cell.column, e, colIndex, row, rowIndex });
      }}
      ref={(el: HTMLTableCellElement | null) => {
        const key = `${rowIndex}-${colIndex}`;
        // eslint-disable-next-line react-hooks/immutability, @typescript-eslint/no-unnecessary-condition -- ref-based keyboard nav requires mutable storage; .current is always initialized
        if (cellRefs.current) cellRefs.current[key] = el;
      }}
      style={{ width: cell.column.getSize(), ...getPinStyles(cell.column) }}
      tabIndex={isCellFocused ? 0 : -1}
    >
      {isCellEditing
        ? (
          <CellEditControl
            editOptions={meta?.editOptions}
            editType={meta?.editType}
            editValue={editValue}
            onBlur={() => { saveCellEdit(rowIndex, colIndex, cell.column, row); }}
            setEditValue={setEditValue}
          />
        )
        : flexRender(cell.column.columnDef.cell, cell.getContext())}

      {hasError ? <CellErrorTooltip cellError={cellError} columnId={cell.column.id} rowId={row.id} /> : null}
    </td>
  );
}

// ── SmartTableRow ────────────────────────────────────────────────────────────

export interface SmartTableRowProps<T> {
  cellErrors: Record<string, string>;
  // RefObject.current is mutable in React 19 types — no need for MutableRefObject
  cellRefs: React.RefObject<Record<string, HTMLTableCellElement | null>>;
  dragCurrentRow: number | null;
  dragStartRow: number | null;
  editingCell: { rowIndex: number; colIndex: number } | null;
  editValue: unknown;
  focusedCell: { rowIndex: number; colIndex: number } | null;
  getPinStyles: (column: Column<T>) => React.CSSProperties;
  handleCellKeyDown: (args: { column: Column<T>; e: React.KeyboardEvent; colIndex: number; row: Row<T>; rowIndex: number }) => void;
  handleRowContextMenu: (row: T, e: React.MouseEvent) => void;
  handleRowMouseDown: (index: number, e: React.MouseEvent) => void;
  handleRowMouseEnter: (index: number) => void;
  isDragging: boolean;
  localCellValues: Record<string, unknown>;
  onRowClick?: (row: T, e: React.MouseEvent) => void;
  row: Row<T>;
  rowIndex: number;
  saveCellEdit: (rowIndex: number, colIndex: number, column: Column<T>, row: Row<T>) => void;
  selectedRowIds?: Set<string>;
  setEditValue: React.Dispatch<React.SetStateAction<unknown>>;
  setFocusedCell: React.Dispatch<React.SetStateAction<{ rowIndex: number; colIndex: number } | null>>;
  startCellEdit: (rowIndex: number, colIndex: number, value: unknown) => void;
}

 
export function SmartTableRow<T>(props: SmartTableRowProps<T>) {
  const {
    row, rowIndex, selectedRowIds, isDragging, dragStartRow, dragCurrentRow,
    onRowClick, handleRowContextMenu, handleRowMouseDown, handleRowMouseEnter,
    cellErrors, cellRefs, editingCell, editValue, focusedCell, getPinStyles,
    handleCellKeyDown, localCellValues, saveCellEdit, setEditValue,
    setFocusedCell, startCellEdit,
  } = props;

  const isSelected = selectedRowIds?.has(row.id) ?? false;
  const isDraggingRange =
    isDragging &&
    dragStartRow !== null &&
    dragCurrentRow !== null &&
    rowIndex >= Math.min(dragStartRow, dragCurrentRow) &&
    rowIndex <= Math.max(dragStartRow, dragCurrentRow);

  return (
    <tr
      className={cn(
        "group/row hover:bg-slate-50/50 dark:hover:bg-slate-800/30 transition-colors cursor-pointer",
        isSelected && "bg-primary-50/20 dark:bg-primary-950/10",
        isDraggingRange && "bg-primary-50/30 dark:bg-primary-950/20 outline outline-2 outline-primary-300 dark:outline-primary-800 outline-offset-[-2px] z-10 relative",
      )}
      onClick={(e: React.MouseEvent) => {
        const target = e.target as HTMLElement;
        if (target.closest("button") || target.closest("select") || target.closest("input") || target.closest("[role='button']")) return;
        onRowClick?.(row.original, e);
      }}
      onContextMenu={(e: React.MouseEvent) => { handleRowContextMenu(row.original, e); }}
      onMouseDown={(e: React.MouseEvent) => { handleRowMouseDown(rowIndex, e); }}
      onMouseEnter={() => { handleRowMouseEnter(rowIndex); }}
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
  );
}
