import {
  type ColumnDef,
  type Row,
  type RowData,
} from "@tanstack/react-table";
import React from "react";

import { cn } from "@/lib/utils";

import type {
  CellEditPayload,
  ContextMenuItem,
  EditableCellValue,
  SmartTableColumnMeta,
} from "./smart-table/types";

import { useSmartTable } from "./smart-table-hooks/use-smart-table";
import SmartTableContextMenu from "./smart-table/smart-table-context-menu";
import { SmartTableHeader } from "./smart-table/smart-table-header";
import { SmartTableRow } from "./smart-table/smart-table-row";

// Re-export for consumers that import from the SmartTable barrel
export type { CellEditPayload, ContextMenuItem, EditableCellValue, SmartTableColumnMeta } from "./smart-table/types";

declare module "@tanstack/react-table" {
  // augments @tanstack/react-table ColumnMeta using SmartTableColumnMeta from shared types
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type, unused-imports/no-unused-vars -- module augmentation; TData/TValue required by interface contract
  interface ColumnMeta<TData extends RowData, TValue> extends SmartTableColumnMeta {}
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
  onBulkCellSave?: (changes: CellEditPayload[]) => Promise<void> | void;
  onCellSave?: (rowId: string, columnId: string, value: EditableCellValue) => Promise<void> | void;
  onRowClick?: (row: T, e: React.MouseEvent) => void;
  onSelectionChange?: (selectedIds: Set<string>) => void;
  selectedRowIds?: Set<string>;
  stickyLeftColumns?: number;
  stickyRightColumns?: number;
  validateCell?: (rowId: string, columnId: string, value: EditableCellValue) => string | null | undefined;
}

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
  const smart = useSmartTable<T>({
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
  });

  return (
    /* eslint-disable-next-line jsx-a11y/no-static-element-interactions */
    <div
      className={cn("relative flex flex-col h-full w-full", className)}
      onKeyDown={smart.handleKeyDown}
      onPaste={smart.handlePaste}
    >
      {/* Scrollable Container with rounded corners */}
      <div className="flex-1 overflow-auto scrollbar-thin border border-slate-200 dark:border-slate-800 rounded-3xl bg-white dark:bg-slate-900 shadow-sm relative">
        <table className="w-full border-collapse text-left table-fixed" style={{ width: smart.table.getTotalSize() }}>
          {/* Header */}
          <SmartTableHeader getPinStyles={smart.getPinStyles} table={smart.table} />

          {/* Body */}
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              <tr>
                <td className="px-8 py-20 text-center text-xs font-bold uppercase tracking-wider text-slate-400 animate-pulse" colSpan={smart.colCount}>
                  Loading...
                </td>
              </tr>
            ) : smart.rowCount === 0 ? (
              <tr>
                <td className="px-8 py-20 text-center text-xs font-bold uppercase tracking-wider text-slate-400/80" colSpan={smart.colCount}>
                  {emptyMessage}
                </td>
              </tr>
            ) : (
              smart.rows.map((row: Row<T>, rowIndex: number) => (
                <SmartTableRow
                  cellErrors={smart.cellErrors}
                  cellRefs={smart.cellRefs}
                  dragCurrentRow={smart.dragCurrentRow}
                  dragStartRow={smart.dragStartRow}
                  editingCell={smart.editingCell}
                  editValue={smart.editValue}
                  focusedCell={smart.focusedCell}
                  getPinStyles={smart.getPinStyles}
                  handleCellKeyDown={smart.handleCellKeyDown}
                  handleRowContextMenu={smart.handleRowContextMenu}
                  handleRowMouseDown={smart.handleRowMouseDown}
                  handleRowMouseEnter={smart.handleRowMouseEnter}
                  isDragging={smart.isDragging}
                  key={row.id}
                  localCellValues={smart.localCellValues}
                  onRowClick={onRowClick}
                  row={row}
                  rowIndex={rowIndex}
                  saveCellEdit={smart.saveCellEdit}
                  selectedRowIds={selectedRowIds}
                  setEditValue={smart.setEditValue}
                  setFocusedCell={smart.setFocusedCell}
                  startCellEdit={smart.startCellEdit}
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
        contextMenu={smart.contextMenu}
        contextMenuItems={contextMenuItems}
        setContextMenu={smart.setContextMenu}
      />
    </div>
  );
}
