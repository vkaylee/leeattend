import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  type ColumnDef,
  type SortingState,
  type RowSelectionState,
  type ColumnPinningState,
  type Column,
  type Row,
  type RowData,
} from "@tanstack/react-table";
import React, { useCallback, useMemo, useState } from "react";

import { cn } from "@/lib/utils";

import { useSmartTableClipboard } from "./smart-table-hooks/use-smart-table-clipboard";
import { useSmartTableDrag } from "./smart-table-hooks/use-smart-table-drag";
import { useSmartTableKeyboard } from "./smart-table-hooks/use-smart-table-keyboard";
import { SmartTableHeader } from "./smart-table/smart-table-header";
import { SmartTableRow } from "./smart-table/smart-table-row";
import SmartTableContextMenu from "./smart-table/smart-table-context-menu";

export interface SmartTableColumnMeta {
  cellClassName?: string;
  editable?: boolean;
  editOptions?: { value: string; label: string }[];
  editType?: "text" | "number" | "select";
  headerClassName?: string;
  sticky?: "left" | "right";
}

declare module "@tanstack/react-table" {
  // augments @tanstack/react-table ColumnMeta using locally-defined SmartTableColumnMeta
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type, unused-imports/no-unused-vars -- module augmentation; TData/TValue required by interface contract
  interface ColumnMeta<TData extends RowData, TValue> extends SmartTableColumnMeta {}
}

export interface ContextMenuItem {
  disabled?: boolean;
  icon?: React.ReactNode;
  label: string;
  onClick: () => void;
}

export interface SmartTableProps<T> {
  className?: string;
  columns: (ColumnDef<T> & { meta?: SmartTableColumnMeta })[];
  contextMenuItems?: (row: T) => ContextMenuItem[];
  data: T[];
  emptyMessage?: string;
  enableDragSelect?: boolean;
  footerRow?: React.ReactNode;
  getRowId: (row: T) => string;
  isLoading?: boolean;
  onBulkCellSave?: (changes: { rowId: string; columnId: string; value: unknown }[]) => Promise<void> | void;
  onCellSave?: (rowId: string, columnId: string, value: unknown) => Promise<void> | void;
  onRowClick?: (row: T, e: React.MouseEvent) => void;
  onSelectionChange?: (selectedIds: Set<string>) => void;
  selectedRowIds?: Set<string>;
  stickyLeftColumns?: number;
  stickyRightColumns?: number;
  validateCell?: (rowId: string, columnId: string, value: unknown) => string | null | undefined;
}

// eslint-disable-next-line max-lines-per-function -- React Table integration requires substantial setup boilerplate
export default function SmartTable<T>({
  data,
  columns,
  getRowId,
  onCellSave,
  onBulkCellSave,
  validateCell,
  selectedRowIds,
  onSelectionChange,
  enableDragSelect,
  contextMenuItems,
  stickyLeftColumns,
  stickyRightColumns,
  isLoading,
  emptyMessage,
  className,
  footerRow,
  onRowClick,
}: SmartTableProps<T>) {
  // Sorting State
  const [sorting, setSorting] = useState<SortingState>([]);
  
  // Row Selection State internal sync
  const rowSelection = useMemo(() => {
    if (!selectedRowIds) return {};
    const selection: RowSelectionState = {};
    data.forEach((row) => {
      const id = getRowId(row);
      if (selectedRowIds.has(id)) {
        selection[id] = true;
      }
    });
    return selection;
  }, [selectedRowIds, data, getRowId]);

  const handleRowSelectionChange = useCallback((
    updaterOrValue: RowSelectionState | ((old: RowSelectionState) => RowSelectionState)
  ) => {
    if (!onSelectionChange) return;
    const newSelection = typeof updaterOrValue === "function" ? updaterOrValue(rowSelection) : updaterOrValue;
    const nextSelectedSet = new Set<string>();
    Object.keys(newSelection).forEach((key) => {
      if (newSelection[key]) {
        nextSelectedSet.add(key);
      }
    });
    onSelectionChange(nextSelectedSet);
  }, [onSelectionChange, rowSelection]);

  const columnPinning = useMemo((): ColumnPinningState => {
    const left: string[] = [];
    const right: string[] = [];
    // eslint-disable-next-line complexity -- pinning logic has inherent branching for left/right states
    columns.forEach((col, index) => {
      const id = (col.id ?? ("accessorKey" in col ? (col.accessorKey as string) : "")) || "";
      const sLC = stickyLeftColumns ?? 0;
      const sRC = stickyRightColumns ?? 0;
      if (col.meta?.sticky === "left" || (sLC > 0 && index < sLC)) {
        left.push(id);
      }
      if (col.meta?.sticky === "right" || (sRC > 0 && index >= columns.length - sRC)) {
        right.push(id);
      }
    });
    return { left, right };
  }, [columns, stickyLeftColumns, stickyRightColumns]);

  // TanStack Table Instance
   
  // eslint-disable-next-line react-hooks/incompatible-library -- TanStack Table useReactTable is the standard API
  const table = useReactTable({
    data,
    columns,
    state: {
      sorting,
      rowSelection,
      columnPinning,
    },
    onSortingChange: setSorting,
    onRowSelectionChange: handleRowSelectionChange,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getRowId,
    columnResizeMode: "onChange",
  });

  const rows = table.getRowModel().rows;
  const colCount = columns.length;
  const rowCount = rows.length;

  // Grid & Validation States shared with hook modules
  const [cellErrors, setCellErrors] = useState<Record<string, string>>({});
  const [localCellValues, setLocalCellValues] = useState<Record<string, unknown>>({});

  // 1. Drag selection hook
  const {
    isDragging,
    dragStartRow,
    dragCurrentRow,
    handleRowMouseDown,
    handleRowMouseEnter,
  } = useSmartTableDrag({
    enableDragSelect: enableDragSelect ?? false,
    selectedRowIds,
    rows,
    onSelectionChange,
  });

  // 2. Keyboard & inline editing hook
  const {
    focusedCell,
    setFocusedCell,
    editingCell,
    editValue,
    setEditValue,
    cellRefs,
    startCellEdit,
    saveCellEdit,
    handleCellKeyDown,
  } = useSmartTableKeyboard<T>({
    rowCount,
    colCount,
    validateCell,
    onCellSave,
    setCellErrors,
    setLocalCellValues,
  });

  // 3. Clipboard copy/paste hook
  const {
    handleKeyDown,
    handlePaste,
  } = useSmartTableClipboard({
    focusedCell,
    editingCell,
    selectedRowIds,
    rows,
    rowCount,
    colCount,
    validateCell,
    onCellSave,
    onBulkCellSave,
    cellErrors,
    setCellErrors,
    localCellValues,
    setLocalCellValues,
  });

  // Context Menu state
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    row: T;
  } | null>(null);

  const handleRowContextMenu = useCallback((row: T, e: React.MouseEvent) => {
    if (!contextMenuItems) return;
    e.preventDefault();
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      row,
    });
  }, [contextMenuItems]);

  // Dynamic pin placement inline styles helper
  const getPinStyles = useCallback((column: Column<T>): React.CSSProperties => {
    const isPinned = column.getIsPinned();
    if (!isPinned) return {};
    const offset = column.getStart(isPinned);
    return {
      position: "sticky",
      [isPinned]: `${offset}px`,
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- isPinned is string | false
      zIndex: isPinned ? 2 : 1,
    };
  }, []);

  return (
    /* eslint-disable-next-line jsx-a11y/no-static-element-interactions */
    <div
      className={cn("relative flex flex-col h-full w-full", className)}
      onKeyDown={handleKeyDown}
      onPaste={handlePaste}
    >
      {/* Scrollable Container with rounded corners */}
      <div className="flex-1 overflow-auto scrollbar-thin border border-slate-200 dark:border-slate-800 rounded-3xl bg-white dark:bg-slate-900 shadow-sm relative">
        <table className="w-full border-collapse text-left table-fixed" style={{ width: table.getTotalSize() }}>
          {/* Header */}
          <SmartTableHeader getPinStyles={getPinStyles} table={table} />

          {/* Body */}
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              <tr>
                <td className="px-8 py-20 text-center text-xs font-bold uppercase tracking-wider text-slate-400 animate-pulse" colSpan={colCount}>
                  Loading...
                </td>
              </tr>
            ) : rowCount === 0 ? (
              <tr>
                <td className="px-8 py-20 text-center text-xs font-bold uppercase tracking-wider text-slate-400/80" colSpan={colCount}>
                  {emptyMessage}
                </td>
              </tr>
            ) : (
              rows.map((row: Row<T>, rowIndex: number) => (
                <SmartTableRow
                  cellErrors={cellErrors}
                  cellRefs={cellRefs}
                  dragCurrentRow={dragCurrentRow}
                  dragStartRow={dragStartRow}
                  editingCell={editingCell}
                  editValue={editValue}
                  focusedCell={focusedCell}
                  getPinStyles={getPinStyles}
                  handleCellKeyDown={handleCellKeyDown}
                  handleRowContextMenu={handleRowContextMenu}
                  handleRowMouseDown={handleRowMouseDown}
                  handleRowMouseEnter={handleRowMouseEnter}
                  isDragging={isDragging}
                  key={row.id}
                  localCellValues={localCellValues}
                  onRowClick={onRowClick}
                  row={row}
                  rowIndex={rowIndex}
                  saveCellEdit={saveCellEdit}
                  selectedRowIds={selectedRowIds}
                  setEditValue={setEditValue}
                  setFocusedCell={setFocusedCell}
                  startCellEdit={startCellEdit}
                />
              ))
            )}

            {/* Render footerRow if provided (e.g. infinite scroll sentinel row) */}
            {footerRow}
          </tbody>
        </table>
      </div>

      {/* Context Menu */}
      <SmartTableContextMenu
        contextMenu={contextMenu}
        contextMenuItems={contextMenuItems}
        setContextMenu={setContextMenu}
      />
    </div>
  );
}
