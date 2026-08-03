import type {
  CellCoordinate,
  EditableCellValue,
  SmartTableColumnMeta,
} from "../smart-table/types";

import { maybeNull } from "@/types/api-overrides";

interface ClipboardCell {
  column: {
    columnDef: {
      meta?: SmartTableColumnMeta;
    };
    id: string;
  };
  getValue: () => unknown;
}

interface ClipboardRow {
  getVisibleCells: () => ClipboardCell[];
  id: string;
}

interface UseSmartTableClipboardProps<TRow extends ClipboardRow = ClipboardRow> {
  cellErrors: Record<string, string>;
  colCount: number;
  editingCell: CellCoordinate | null;
  focusedCell: CellCoordinate | null;
  localCellValues: Record<string, EditableCellValue>;
  onBulkCellSave?: (changes: { columnId: string; rowId: string; value: EditableCellValue }[]) => Promise<void> | void;
  onCellSave?: (rowId: string, columnId: string, value: EditableCellValue) => Promise<void> | void;
  rowCount: number;
  rows: TRow[];
  selectedRowIds?: Set<string>;
  setCellErrors: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  setLocalCellValues: React.Dispatch<React.SetStateAction<Record<string, EditableCellValue>>>;
  validateCell?: (rowId: string, columnId: string, value: EditableCellValue) => string | null | undefined;
}

// ── Pure helpers (extracted from handleKeyDown / handlePaste) ──────────

function cellValueToString(cell: ClipboardCell): string {
  const val = cell.getValue() as boolean | null | number | string | undefined;
  return val != null ? String(val) : "";
}

function buildCopyTsv(
  rows: ClipboardRow[],
  focusedCell: CellCoordinate | null,
  selectedRowIds?: Set<string>,
): string {
  const selectedRows = rows.filter((r) => selectedRowIds?.has(r.id));

  if (selectedRows.length > 0) {
    return selectedRows
      .map((r) => r.getVisibleCells().map(cellValueToString).join("\t"))
      .join("\n");
  }

  if (focusedCell) {
    const r = rows[focusedCell.rowIndex];
    // rows[] may return undefined at runtime (index out of bounds)
    const row = maybeNull(r);
    const cells = row?.getVisibleCells();
    const cell = cells ? cells[focusedCell.colIndex] : undefined;
    if (cell) {
      return cellValueToString(cell);
    }
  }

  return "";
}

function parseClipboardGrid(text: string): string[][] {
  const lines = text.split(/\r?\n/);
  const grid = lines.map((line) => line.split("\t"));
  // Strip trailing empty line (common when copying table rows)
  if (grid.length > 0 && grid[grid.length - 1].length === 1 && grid[grid.length - 1][0] === "") {
    grid.pop();
  }
  return grid;
}

interface PasteGridResult {
  changes: { columnId: string; rowId: string; value: EditableCellValue }[];
  newErrors: Record<string, string>;
  newLocalValues: Record<string, EditableCellValue>;
}

interface ApplyPasteParams {
  cellErrors: Record<string, string>;
  colCount: number;
  focusedCell: CellCoordinate;
  localCellValues: Record<string, EditableCellValue>;
  rowCount: number;
  rows: ClipboardRow[];
  validateCell?: (rowId: string, columnId: string, value: EditableCellValue) => string | null | undefined;
}

function applyPasteToGrid(
  grid: string[][],
  params: ApplyPasteParams,
): PasteGridResult {
  const { focusedCell, rows, rowCount, colCount, cellErrors, localCellValues, validateCell } = params;
  const changes: { columnId: string; rowId: string; value: EditableCellValue }[] = [];
  const newErrors = { ...cellErrors };
  const newLocalValues: Record<string, EditableCellValue> = { ...localCellValues };

  function parseValue(rawValue: string, meta: SmartTableColumnMeta): EditableCellValue {
    if (meta.editType === "number") {
      const num = Number(rawValue);
      return isNaN(num) ? rawValue : num;
    }
    return rawValue;
  }

  function processCell(
    row: ClipboardRow, cell: ClipboardCell, rawValue: string, meta: SmartTableColumnMeta,
  ): void {
    const parsedValue = parseValue(rawValue, meta);
    let error: string | null | undefined = null;
    if (validateCell) {
      error = validateCell(row.id, cell.column.id, parsedValue);
    }

    const errorKey = `${row.id}-${cell.column.id}`;
    if (error) {
      newErrors[errorKey] = error;
      newLocalValues[errorKey] = parsedValue;
    } else {
      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete -- clearing previous validation state by key; ref is a shallow copy of state
      delete newErrors[errorKey];
      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
      delete newLocalValues[errorKey];
      changes.push({ columnId: cell.column.id, rowId: row.id, value: parsedValue });
    }
  }

  for (let i = 0; i < grid.length; i++) {
    const targetRowIndex = focusedCell.rowIndex + i;
    if (targetRowIndex >= rowCount) break;

    const row = rows[targetRowIndex];
    const cells = row.getVisibleCells();

    for (let j = 0; j < grid[i].length; j++) {
      const targetColIndex = focusedCell.colIndex + j;
      if (targetColIndex >= colCount) break;

      const cell = cells[targetColIndex];
      const meta = cell.column.columnDef.meta;

      if (meta?.editable) {
        processCell(row, cell, grid[i][j], meta);
      }
    }
  }

  return { changes, newErrors, newLocalValues };
}

async function persistChanges(
  changes: { columnId: string; rowId: string; value: EditableCellValue }[],
  onBulkCellSave?: (changes: { columnId: string; rowId: string; value: EditableCellValue }[]) => Promise<void> | void,
  onCellSave?: (rowId: string, columnId: string, value: EditableCellValue) => Promise<void> | void,
): Promise<void> {
  if (onBulkCellSave) {
    const result = onBulkCellSave(changes);
    if (result instanceof Promise) {
      await result.catch((err: unknown) => { console.warn("Error saving bulk cells:", err); });
    }
  } else if (onCellSave) {
    for (const change of changes) {
      const result = onCellSave(change.rowId, change.columnId, change.value);
      if (result instanceof Promise) {
        await result.catch((err: unknown) => { console.warn("Error saving cell:", err); });
      }
    }
  }
}

// ── Hook ───────────────────────────────────────────────────────────────

export function useSmartTableClipboard<TRow extends ClipboardRow = ClipboardRow>({
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
}: UseSmartTableClipboardProps<TRow>) {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (editingCell) return;

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "c") {
      if (!focusedCell && (!selectedRowIds || selectedRowIds.size === 0)) return;

      e.preventDefault();
      const tsvText = buildCopyTsv(rows, focusedCell, selectedRowIds);

      if (tsvText) {
        navigator.clipboard
          .writeText(tsvText)
          .catch((err: unknown) => { console.warn("Clipboard copy failed: ", err); });
      }
    }
  };

  const handlePaste = async (e: React.ClipboardEvent) => {
    if (editingCell) return;
    if (!focusedCell) return;

    const text = e.clipboardData.getData("text");
    if (!text) return;

    const grid = parseClipboardGrid(text);
    if (grid.length === 0) return;

    e.preventDefault();

    const { changes, newErrors, newLocalValues } = applyPasteToGrid(
      grid, { focusedCell, rows, rowCount, colCount, cellErrors, localCellValues, validateCell },
    );

    setCellErrors(newErrors);
    setLocalCellValues(newLocalValues);

    if (changes.length > 0) {
      await persistChanges(changes, onBulkCellSave, onCellSave);
    }
  };

  return {
    handleKeyDown,
    handlePaste,
  };
}
