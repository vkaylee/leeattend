import React from "react";

interface LocalSmartTableColumnMeta {
  cellClassName?: string;
  editable?: boolean;
  editOptions?: { value: string; label: string }[];
  editType?: "text" | "number" | "select";
  headerClassName?: string;
  sticky?: "left" | "right";
}

interface ClipboardCell {
  column: {
    columnDef: {
      meta?: LocalSmartTableColumnMeta;
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
  editingCell: { rowIndex: number; colIndex: number } | null;
  focusedCell: { rowIndex: number; colIndex: number } | null;
  localCellValues: Record<string, unknown>;
  onBulkCellSave?: (changes: { columnId: string; rowId: string; value: unknown }[]) => Promise<void> | void;
  onCellSave?: (rowId: string, columnId: string, value: unknown) => Promise<void> | void;
  rowCount: number;
  rows: TRow[];
  selectedRowIds?: Set<string>;
  setCellErrors: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  setLocalCellValues: React.Dispatch<React.SetStateAction<Record<string, unknown>>>;
  validateCell?: (rowId: string, columnId: string, value: unknown) => string | null | undefined;
}

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

      const selectedRows = rows.filter((r) => selectedRowIds?.has(r.id));
      let tsvText = "";

      if (selectedRows.length > 0) {
        tsvText = selectedRows
          .map((r) =>
            r
              .getVisibleCells()
              .map((cell) => {
                const val = cell.getValue() as boolean | null | number | string | undefined;
                return val != null ? String(val) : "";
              })
              .join("\t")
          )
          .join("\n");
      } else if (focusedCell) {
        const r = rows[focusedCell.rowIndex];
        const cell = r?.getVisibleCells()[focusedCell.colIndex];
        if (cell) {
          const val = cell.getValue() as boolean | null | number | string | undefined;
          tsvText = val != null ? String(val) : "";
        }
      }

      if (tsvText) {
        navigator.clipboard
          .writeText(tsvText)
          .catch((err: unknown) => { console.error("Clipboard copy failed: ", err); });
      }
    }
  };

  const handlePaste = async (e: React.ClipboardEvent) => {
    if (editingCell) return;
    if (!focusedCell) return;

    const text = e.clipboardData.getData("text");
    if (!text) return;

    const lines = text.split(/\r?\n/);
    const grid = lines.map((line) => line.split("\t"));
    if (grid.length > 0 && grid[grid.length - 1].length === 1 && grid[grid.length - 1][0] === "") {
      grid.pop();
    }
    if (grid.length === 0) return;

    e.preventDefault();

    const changes: { columnId: string; rowId: string; value: unknown }[] = [];
    const newErrors = { ...cellErrors };
    const newLocalValues = { ...localCellValues };

    for (let i = 0; i < grid.length; i++) {
      const targetRowIndex = focusedCell.rowIndex + i;
      if (targetRowIndex >= rowCount) break;

      const row = rows[targetRowIndex];
      const cells = row.getVisibleCells();

      for (let j = 0; j < grid[i].length; j++) {
        const targetColIndex = focusedCell.colIndex + j;
        if (targetColIndex >= colCount) break;

        const cell = cells[targetColIndex];
        const column = cell.column;
        const meta = column.columnDef.meta;

        if (meta?.editable) {
          const rawValue = grid[i][j];
          let parsedValue: unknown = rawValue;
          if (meta.editType === "number") {
            const num = Number(rawValue);
            parsedValue = isNaN(num) ? rawValue : num;
          }

          let error: string | null | undefined = null;
          if (validateCell) {
            error = validateCell(row.id, column.id, parsedValue);
          }

          const errorKey = `${row.id}-${column.id}`;
          if (error) {
            newErrors[errorKey] = error;
            newLocalValues[errorKey] = parsedValue;
          } else {
            delete newErrors[errorKey];
            delete newLocalValues[errorKey];
            changes.push({
              columnId: column.id,
              rowId: row.id,
              value: parsedValue,
            });
          }
        }
      }
    }

    setCellErrors(newErrors);
    setLocalCellValues(newLocalValues);

    if (changes.length > 0) {
      if (onBulkCellSave) {
        const result = onBulkCellSave(changes);
        if (result instanceof Promise) {
          result.catch((err: unknown) => { console.error("Error saving bulk cells:", err); });
        }
      } else if (onCellSave) {
        for (const change of changes) {
          const result = onCellSave(change.rowId, change.columnId, change.value);
          if (result instanceof Promise) {
            await result.catch((err: unknown) => { console.error("Error saving cell:", err); });
          }
        }
      }
    }
  };

  return {
    handleKeyDown,
    handlePaste,
  };
}
