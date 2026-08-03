import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  type ColumnDef,
  type SortingState,
  type ColumnPinningState,
  type Column as TanStackColumn,
  type Row,
} from "@tanstack/react-table";
import React, { useCallback, useMemo, useState } from "react";

import type {
  CellEditPayload,
  ContextMenuItem,
  EditableCellValue,
  SmartTableColumnMeta,
} from "../smart-table/types";

import { useSmartTableClipboard } from "../smart-table-hooks/use-smart-table-clipboard";
import { useSmartTableDrag } from "../smart-table-hooks/use-smart-table-drag";
import { useSmartTableKeyboard } from "../smart-table-hooks/use-smart-table-keyboard";
import { buildColumnPinning } from "./column-helpers";
import { useSmartTableRowSelection } from "./use-smart-table-row-selection";

// ── Hook input ────────────────────────────────────────────────────────

interface UseSmartTableProps<T> {
  columns: (ColumnDef<T> & { meta?: SmartTableColumnMeta })[];
  contextMenuItems?: (row: T) => ContextMenuItem[];
  data: T[];
  enableDragSelect?: boolean;
  getRowId: (row: T) => string;
  onBulkCellSave?: (changes: CellEditPayload[]) => Promise<void> | void;
  onCellSave?: (rowId: string, columnId: string, value: EditableCellValue) => Promise<void> | void;
  onSelectionChange?: (selectedIds: Set<string>) => void;
  selectedRowIds?: Set<string>;
  stickyLeftColumns?: number;
  stickyRightColumns?: number;
  validateCell?: (rowId: string, columnId: string, value: EditableCellValue) => string | null | undefined;
}

// ── Hook return ───────────────────────────────────────────────────────

export interface UseSmartTableReturn<T> {
  // Grid validation state
  cellErrors: Record<string, string>;

  // Keyboard & editing
  cellRefs: React.RefObject<Record<string, HTMLTableCellElement | null>>;
  // Derived data
  colCount: number;
  // Context menu
  contextMenu: { x: number; y: number; row: T } | null;

  // Drag selection
  dragCurrentRow: number | null;
  dragStartRow: number | null;
  editingCell: { colIndex: number; rowIndex: number } | null;
  editValue: EditableCellValue;
  focusedCell: { colIndex: number; rowIndex: number } | null;

  // Styling
  getPinStyles: (column: TanStackColumn<T>) => React.CSSProperties;
  handleCellKeyDown: (args: { column: TanStackColumn<T>; e: React.KeyboardEvent; colIndex: number; row: Row<T>; rowIndex: number }) => void;
  // Clipboard
  handleKeyDown: (e: React.KeyboardEvent) => void;
  handlePaste: (e: React.ClipboardEvent) => Promise<void>;
  handleRowContextMenu: (row: T, e: React.MouseEvent) => void;
  handleRowMouseDown: (index: number, e: React.MouseEvent) => void;
  handleRowMouseEnter: (index: number) => void;
  isDragging: boolean;
  localCellValues: Record<string, EditableCellValue>;

  rowCount: number;
  rows: Row<T>[];

  saveCellEdit: (rowIndex: number, colIndex: number, column: TanStackColumn<T>, row: Row<T>) => void;
  setContextMenu: React.Dispatch<React.SetStateAction<{ x: number; y: number; row: T } | null>>;

  setEditValue: React.Dispatch<React.SetStateAction<EditableCellValue>>;
  setFocusedCell: React.Dispatch<React.SetStateAction<{ colIndex: number; rowIndex: number } | null>>;
  startCellEdit: (rowIndex: number, colIndex: number, value: EditableCellValue) => void;

  // Table instance
  table: ReturnType<typeof useReactTable<T>>;
}

// ── Hook ──────────────────────────────────────────────────────────────

export function useSmartTable<T>({
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
}: UseSmartTableProps<T>): UseSmartTableReturn<T> {
  // ── Sorting ──────────────────────────────────────────────────────
  const [sorting, setSorting] = useState<SortingState>([]);

  // ── Row selection ────────────────────────────────────────────────
  const { rowSelection, handleRowSelectionChange } = useSmartTableRowSelection({
    selectedRowIds,
    data,
    getRowId,
    onSelectionChange,
  });

  // ── Column pinning ───────────────────────────────────────────────
  const columnPinning = useMemo((): ColumnPinningState =>
    buildColumnPinning(columns, stickyLeftColumns, stickyRightColumns),
  [columns, stickyLeftColumns, stickyRightColumns]);

  // ── Table instance ───────────────────────────────────────────────
  // eslint-disable-next-line react-hooks/incompatible-library -- TanStack Table's useReactTable API returns functions that cannot be memoized safely; this is a known limitation
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

  // ── Grid validation state ────────────────────────────────────────
  const [cellErrors, setCellErrors] = useState<Record<string, string>>({});
  const [localCellValues, setLocalCellValues] = useState<Record<string, EditableCellValue>>({});

  // ── Sub-hooks ────────────────────────────────────────────────────

  // 1. Drag selection
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

  // 2. Keyboard & inline editing
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

  // 3. Clipboard copy/paste
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

  // ── Context menu ─────────────────────────────────────────────────
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

  // ── Styling helpers ──────────────────────────────────────────────
  const getPinStyles = useCallback((column: TanStackColumn<T>): React.CSSProperties => {
    const isPinned = column.getIsPinned();
    if (!isPinned) return {};
    const offset = column.getStart(isPinned);
    return {
      position: "sticky",
      [isPinned]: `${offset}px`,
      zIndex: 2,
    };
  }, []);

  return {
    table,
    colCount,
    rows,
    rowCount,

    // Drag
    dragCurrentRow,
    dragStartRow,
    handleRowMouseDown,
    handleRowMouseEnter,
    isDragging,

    // Keyboard & editing
    cellRefs,
    editingCell,
    editValue,
    focusedCell,
    handleCellKeyDown,
    saveCellEdit,
    setEditValue,
    setFocusedCell,
    startCellEdit,

    // Clipboard
    handleKeyDown,
    handlePaste,

    // Validation
    cellErrors,
    localCellValues,

    // Context menu
    contextMenu,
    setContextMenu,
    handleRowContextMenu,

    // Styling
    getPinStyles,
  };
}
