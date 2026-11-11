import {
  type ColumnDef,
  type Row,
  type RowData,
} from "@tanstack/react-table";
import React from "react";

import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
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
import type { TableDensity } from "./table-density-toggle";

// Re-export for consumers that import from the SmartTable barrel
export type { CellEditPayload, ContextMenuItem, EditableCellValue, SmartTableColumnMeta } from "./smart-table/types";
export type { TableDensity } from "./table-density-toggle";

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
  density?: TableDensity;
  emptyMessage?: string;
  enableDragSelect?: boolean;
  footerRow?: React.ReactNode;
  /** Per-row className override, merged after the built-in row classes. */
  getRowClassName?: (row: T, index: number) => string | undefined;
  getRowId: (row: T) => string;
  isLoading?: boolean;
  onBulkCellSave?: (changes: CellEditPayload[]) => Promise<void> | void;
  onCellSave?: (rowId: string, columnId: string, value: EditableCellValue) => Promise<void> | void;
  onRowClick?: (row: T, e: React.MouseEvent) => void;
  onSelectionChange?: (selectedIds: Set<string>) => void;
  /**
   * Renders a full-width detail row below each row for which it returns content.
   * SmartTable owns the `<tr>`/`<td colSpan>` wrapper, so the colSpan always
   * tracks the real visible-column count.
   */
  renderExpandedRow?: (row: T) => React.ReactNode;
  /** Custom skeleton row renderer used while loading. Rendered `skeletonCount` times. */
  renderSkeletonRow?: (index: number) => React.ReactNode;
  selectedRowIds?: Set<string>;
  /** Number of skeleton rows to render while loading. Default: 5 */
  skeletonCount?: number;
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
  getRowClassName,
  renderExpandedRow,
  renderSkeletonRow,
  skeletonCount = 5,
  density = "normal",
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
  const {
    onClickCapture: handleHorizontalDragClick,
    setRef: setHorizontalDragScrollRef,
  } = useHorizontalDragScroll<HTMLDivElement>({ enabled: !enableDragSelect });

  return (
    /* eslint-disable-next-line jsx-a11y/no-static-element-interactions */
    <div
      className={cn("relative flex flex-col h-full w-full", className)}
      onKeyDown={smart.handleKeyDown}
      onPaste={(e) => { void smart.handlePaste(e); }}
    >
      {/* Scrollable Container with rounded corners */}
      <div
        className="flex-1 overflow-auto scrollbar-thin border border-slate-200 dark:border-slate-800 rounded-3xl bg-white dark:bg-slate-900 shadow-sm relative"
        onClickCapture={handleHorizontalDragClick}
        ref={setHorizontalDragScrollRef}
      >
        <table className="w-full border-collapse text-left table-fixed" style={{ width: smart.table.getTotalSize() }}>
          {/* Header */}
          <SmartTableHeader density={density} getPinStyles={smart.getPinStyles} table={smart.table} />

          {/* Body */}
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              renderSkeletonRow ? (
                Array.from({ length: skeletonCount }).map((_, i) => (
                  <React.Fragment key={`smart-skeleton-${i}`}>
                    {renderSkeletonRow(i)}
                  </React.Fragment>
                ))
              ) : (
                <tr>
                  <td className="px-8 py-20 text-center text-xs font-medium text-slate-400 animate-pulse" colSpan={smart.colCount}>
                    Loading...
                  </td>
                </tr>
              )
            ) : smart.rowCount === 0 ? (
              <tr>
                <td className="px-8 py-20 text-center text-xs font-medium text-slate-400/90 dark:text-slate-500" colSpan={smart.colCount}>
                  {emptyMessage}
                </td>
              </tr>
            ) : (
              smart.rows.map((row: Row<T>, rowIndex: number) => (
                <SmartTableRow<T>
                  cellErrors={smart.cellErrors}
                  cellRefs={smart.cellRefs}
                  colCount={smart.colCount}
                  density={density}
                  dragCurrentRow={smart.dragCurrentRow}
                  dragStartRow={smart.dragStartRow}
                  editingCell={smart.editingCell}
                  editValue={smart.editValue}
                  focusedCell={smart.focusedCell}
                  getPinStyles={smart.getPinStyles}
                  getRowClassName={getRowClassName}
                  handleCellKeyDown={smart.handleCellKeyDown}
                  handleRowContextMenu={smart.handleRowContextMenu}
                  handleRowMouseDown={smart.handleRowMouseDown}
                  handleRowMouseEnter={smart.handleRowMouseEnter}
                  isDragging={smart.isDragging}
                  key={row.id}
                  localCellValues={smart.localCellValues}
                  onRowClick={onRowClick}
                  renderExpandedRow={renderExpandedRow}
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
