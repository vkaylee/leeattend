import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  flexRender,
  type ColumnDef,
  type SortingState,
  type RowSelectionState,
  type ColumnPinningState,
} from "@tanstack/react-table";
import { ChevronUp, ChevronDown, MoreVertical } from "lucide-react";
import React, { useState, useEffect, useRef, useMemo } from "react";

import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

import { useSmartTableClipboard } from "./smart-table-hooks/use-smart-table-clipboard";
import { useSmartTableDrag } from "./smart-table-hooks/use-smart-table-drag";
import { useSmartTableKeyboard } from "./smart-table-hooks/use-smart-table-keyboard";

export interface SmartTableColumnMeta {
  editable?: boolean;
  editType?: "text" | "number" | "select";
  editOptions?: { value: string; label: string }[];
  sticky?: "left" | "right";
  headerClassName?: string;
  cellClassName?: string;
}

export interface SmartTableProps<T> {
  data: T[];
  columns: (ColumnDef<T> & { meta?: SmartTableColumnMeta })[];
  getRowId: (row: T) => string;
  onCellSave?: (rowId: string, columnId: string, value: any) => Promise<void> | void;
  onBulkCellSave?: (changes: { rowId: string; columnId: string; value: any }[]) => Promise<void> | void;
  validateCell?: (rowId: string, columnId: string, value: any) => string | null | undefined;
  selectedRowIds?: Set<string>;
  onSelectionChange?: (selectedIds: Set<string>) => void;
  enableDragSelect?: boolean;
  contextMenuItems?: (row: T) => { label: string; onClick: () => void; icon?: React.ReactNode; disabled?: boolean }[];
  stickyLeftColumns?: number;
  stickyRightColumns?: number;
  isLoading?: boolean;
  emptyMessage?: string;
  className?: string;
  footerRow?: React.ReactNode;
  onRowClick?: (row: T, e: React.MouseEvent) => void;
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
  enableDragSelect = false,
  contextMenuItems,
  stickyLeftColumns = 0,
  stickyRightColumns = 0,
  isLoading = false,
  emptyMessage = "No data available",
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

  const handleRowSelectionChange = (updaterOrValue: any) => {
    if (!onSelectionChange) return;
    const newSelection = typeof updaterOrValue === "function" ? updaterOrValue(rowSelection) : updaterOrValue;
    const nextSelectedSet = new Set<string>();
    Object.keys(newSelection).forEach((key) => {
      if (newSelection[key]) {
        nextSelectedSet.add(key);
      }
    });
    onSelectionChange(nextSelectedSet);
  };

  // Pinning Configuration
  const columnIds = useMemo(() => columns.map((col) => col.id || ((col as any).accessorKey as string) || ""), [columns]);
  const columnPinning = useMemo((): ColumnPinningState => {
    const left: string[] = [];
    const right: string[] = [];
    columns.forEach((col, index) => {
      const id = col.id || ((col as any).accessorKey as string) || "";
      if (col.meta?.sticky === "left" || (stickyLeftColumns > 0 && index < stickyLeftColumns)) {
        left.push(id);
      }
      if (col.meta?.sticky === "right" || (stickyRightColumns > 0 && index >= columns.length - stickyRightColumns)) {
        right.push(id);
      }
    });
    return { left, right };
  }, [columns, stickyLeftColumns, stickyRightColumns]);

  // TanStack Table Instance
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

  const flatHeaders = table.getFlatHeaders();
  const rows = table.getRowModel().rows;
  const colCount = columns.length;
  const rowCount = rows.length;

  // Grid & Validation States shared with hook modules
  const [cellErrors, setCellErrors] = useState<Record<string, string>>({});
  const [localCellValues, setLocalCellValues] = useState<Record<string, any>>({});

  // 1. Drag selection hook
  const {
    isDragging,
    dragStartRow,
    dragCurrentRow,
    handleRowMouseDown,
    handleRowMouseEnter,
  } = useSmartTableDrag({
    enableDragSelect,
    selectedRowIds,
    rows,
    onSelectionChange,
  });

  // 2. Keyboard & inline editing hook
  const {
    focusedCell,
    setFocusedCell,
    editingCell,
    setEditingCell,
    editValue,
    setEditValue,
    cellRefs,
    startCellEdit,
    saveCellEdit,
    handleCellKeyDown,
  } = useSmartTableKeyboard({
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

  const handleRowContextMenu = (row: T, e: React.MouseEvent) => {
    if (!contextMenuItems) return;
    e.preventDefault();
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      row,
    });
  };

  // Dynamic pin placement inline styles helper
  const getPinStyles = (column: any): React.CSSProperties => {
    const isPinned = column.getIsPinned();
    if (!isPinned) return {};
    const offset = column.getStart(isPinned);
    return {
      position: "sticky",
      [isPinned]: `${offset}px`,
      zIndex: isPinned ? 2 : 1,
    };
  };

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
          <thead className="sticky top-0 z-20 bg-white/95 dark:bg-slate-900/95 backdrop-blur border-b border-slate-100 dark:border-slate-800 shadow-sm">
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header, colIndex) => {
                  const isPinned = header.column.getIsPinned();
                  const isLastPinnedLeft = isPinned === "left" && header.column.getIsLastColumn("left");
                  const isFirstPinnedRight = isPinned === "right" && header.column.getIsFirstColumn("right");
                  const meta = header.column.columnDef.meta as SmartTableColumnMeta | undefined;

                  return (
                    <th
                      className={cn(
                        "group relative px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest bg-slate-50/50 dark:bg-slate-900/50 transition-colors border-r border-b border-slate-100 dark:border-slate-800 last:border-r-0 select-none",
                        isPinned && "bg-slate-100/60 dark:bg-slate-900",
                        isLastPinnedLeft && "border-r border-r-slate-200 dark:border-r-slate-700 shadow-[2px_0_5px_-2px_rgba(0,0,0,0.1)]",
                        isFirstPinnedRight && "border-l border-l-slate-200 dark:border-l-slate-700 shadow-[-2px_0_5px_-2px_rgba(0,0,0,0.1)]",
                        meta?.headerClassName
                      )}
                      key={header.id}
                      style={{
                        width: header.getSize(),
                        ...getPinStyles(header.column),
                      }}
                    >
                      {header.isPlaceholder ? null : header.column.getCanSort() ? (
                        <button
                          type="button"
                          className="flex items-center gap-1.5 cursor-pointer font-bold uppercase tracking-widest text-[10px] text-slate-500 dark:text-slate-400 bg-transparent border-none p-0 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary-500 rounded"
                          onClick={header.column.getToggleSortingHandler()}
                        >
                          {flexRender(header.column.columnDef.header, header.getContext())}
                          {{
                            asc: <ChevronUp className="h-3 w-3 text-primary-500" />,
                            desc: <ChevronDown className="h-3 w-3 text-primary-500" />,
                          }[header.column.getIsSorted() as string] ?? null}
                        </button>
                      ) : (
                        <div className="flex items-center gap-1.5">
                          {flexRender(header.column.columnDef.header, header.getContext())}
                        </div>
                      )}

                      {/* Resizer handle */}
                      {header.column.getCanResize() && (
                        /* eslint-disable-next-line jsx-a11y/no-static-element-interactions */
                        <div
                          onMouseDown={header.getResizeHandler()}
                          onTouchStart={header.getResizeHandler()}
                          className={cn(
                            "absolute right-0 top-0 h-full w-1 cursor-col-resize select-none bg-slate-200 dark:bg-slate-700 opacity-0 group-hover:opacity-100 transition-opacity",
                            header.column.getIsResizing() && "bg-primary-500 opacity-100 w-1.5"
                          )}
                        />
                      )}
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>

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
              rows.map((row, rowIndex) => {
                const isSelected = selectedRowIds?.has(row.id) || false;
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
                      isDraggingRange && "bg-primary-50/30 dark:bg-primary-950/20 outline outline-2 outline-primary-300 dark:outline-primary-800 outline-offset-[-2px] z-10 relative"
                    )}
                    key={row.id}
                    onMouseDown={(e) => handleRowMouseDown(rowIndex, e)}
                    onMouseEnter={() => handleRowMouseEnter(rowIndex)}
                    onContextMenu={(e) => handleRowContextMenu(row.original, e)}
                    onClick={(e) => {
                      const target = e.target as HTMLElement;
                      if (
                        target.closest("button") ||
                        target.closest("select") ||
                        target.closest("input") ||
                        target.closest("[role='button']")
                      ) {
                        return;
                      }
                      onRowClick?.(row.original, e);
                    }}
                  >
                    {row.getVisibleCells().map((cell, colIndex) => {
                      const isCellFocused =
                        focusedCell?.rowIndex === rowIndex && focusedCell?.colIndex === colIndex;
                      const isCellEditing =
                        editingCell?.rowIndex === rowIndex && editingCell?.colIndex === colIndex;

                      const isPinned = cell.column.getIsPinned();
                      const isLastPinnedLeft = isPinned === "left" && cell.column.getIsLastColumn("left");
                      const isFirstPinnedRight = isPinned === "right" && cell.column.getIsFirstColumn("right");

                      const meta = cell.column.columnDef.meta as SmartTableColumnMeta | undefined;

                      const errorKey = `${row.id}-${cell.column.id}`;
                      const cellError = cellErrors[errorKey];
                      const hasError = !!cellError;

                      // Override value if there is a local validation error
                      if (hasError) {
                        const overrideValue = localCellValues[errorKey];
                        try {
                          cell.getValue = () => overrideValue;
                          // @ts-ignore
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
                        meta?.cellClassName
                      );

                      const key = `${rowIndex}-${colIndex}`;

                      return (
                        <td
                          className={cellClass}
                          key={cell.id}
                          style={{
                            width: cell.column.getSize(),
                            ...getPinStyles(cell.column),
                          }}
                          ref={(el) => {
                            cellRefs.current[key] = el;
                          }}
                          tabIndex={isCellFocused ? 0 : -1}
                          onClick={() => setFocusedCell({ rowIndex, colIndex })}
                          onKeyDown={(e) => handleCellKeyDown(rowIndex, colIndex, cell.column, row, e)}
                          onDoubleClick={() => {
                            if (meta?.editable) {
                              startCellEdit(rowIndex, colIndex, row.getValue(cell.column.id));
                            }
                          }}
                        >
                          {isCellEditing ? (
                            meta?.editType === "select" ? (
                              <select
                                className="w-full h-full bg-white dark:bg-slate-800 text-sm border-none outline-none ring-2 ring-primary-500 rounded px-2 py-1.5 focus:ring-primary-500"
                                value={editValue}
                                autoFocus
                                onChange={(e) => setEditValue(e.target.value)}
                                onBlur={() => saveCellEdit(rowIndex, colIndex, cell.column, row)}
                              >
                                {meta.editOptions?.map((opt) => (
                                  <option key={opt.value} value={opt.value}>
                                    {opt.label}
                                  </option>
                                ))}
                              </select>
                            ) : (
                              <input
                                className="w-full h-full bg-white dark:bg-slate-800 text-sm border-none outline-none ring-2 ring-primary-500 rounded px-2 py-1.5 focus:ring-primary-500 font-medium"
                                value={editValue ?? ""}
                                autoFocus
                                type={meta?.editType === "number" ? "number" : "text"}
                                onChange={(e) => setEditValue(e.target.value)}
                                onBlur={() => saveCellEdit(rowIndex, colIndex, cell.column, row)}
                              />
                            )
                          ) : (
                            flexRender(cell.column.columnDef.cell, cell.getContext())
                          )}

                          {hasError && (
                            <>
                              <span className="absolute top-1.5 right-1.5 w-2 h-2 rounded-full bg-rose-500 animate-pulse pointer-events-none" />
                              <div
                                id={`error-${row.id}-${cell.column.id}`}
                                role="tooltip"
                                className="absolute hidden group-hover:block group-focus-within:block bottom-full left-1/2 -translate-x-1/2 mb-2 z-30 px-2.5 py-1.5 text-xs font-semibold text-white bg-rose-600 dark:bg-rose-700 rounded-lg shadow-lg whitespace-nowrap pointer-events-none transition-all duration-200"
                              >
                                {cellError}
                                <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-1 border-4 border-transparent border-t-rose-600 dark:border-t-rose-700" />
                              </div>
                            </>
                          )}
                        </td>
                      );
                    })}
                  </tr>
                );
              })
            )}

            {/* Render footerRow if provided (e.g. infinite scroll sentinel row) */}
            {footerRow}
          </tbody>
        </table>
      </div>

      {/* Portal Context Menu invisible trigger */}
      {contextMenu && (
        <DropdownMenu
          open={!!contextMenu}
          onOpenChange={(open) => {
            if (!open) setContextMenu(null);
          }}
        >
          <DropdownMenuTrigger asChild>
            <div
              style={{
                position: "fixed",
                left: contextMenu.x,
                top: contextMenu.y,
                width: 1,
                height: 1,
              }}
            />
          </DropdownMenuTrigger>
          <DropdownMenuContent className="w-48 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-xl rounded-2xl p-1.5">
            {contextMenuItems?.(contextMenu.row).map((item, idx) => (
              <DropdownMenuItem
                key={idx}
                disabled={item.disabled}
                onClick={() => {
                  item.onClick();
                  setContextMenu(null);
                }}
                className={cn(
                  "flex items-center gap-2 px-3 py-2 text-xs font-bold uppercase tracking-tight rounded-xl cursor-pointer hover:bg-slate-100 dark:hover:bg-slate-800",
                  item.disabled && "opacity-50 pointer-events-none"
                )}
              >
                {item.icon}
                <span>{item.label}</span>
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      )}
    </div>
  );
}
