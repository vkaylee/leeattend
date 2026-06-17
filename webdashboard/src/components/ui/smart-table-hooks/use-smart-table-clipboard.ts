import React from "react";

interface LocalSmartTableColumnMeta {
  editable?: boolean;
  editType?: "text" | "number" | "select";
  editOptions?: { value: string; label: string }[];
  sticky?: "left" | "right";
  headerClassName?: string;
  cellClassName?: string;
}

interface UseSmartTableClipboardProps {
  focusedCell: { rowIndex: number; colIndex: number } | null;
  editingCell: { rowIndex: number; colIndex: number } | null;
  selectedRowIds?: Set<string>;
  rows: any[];
  rowCount: number;
  colCount: number;
  validateCell?: (rowId: string, columnId: string, value: any) => string | null | undefined;
  onCellSave?: (rowId: string, columnId: string, value: any) => Promise<void> | void;
  onBulkCellSave?: (changes: { rowId: string; columnId: string; value: any }[]) => Promise<void> | void;
  cellErrors: Record<string, string>;
  setCellErrors: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  localCellValues: Record<string, any>;
  setLocalCellValues: React.Dispatch<React.SetStateAction<Record<string, any>>>;
}

export function useSmartTableClipboard({
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
}: UseSmartTableClipboardProps) {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (editingCell) return;

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "c") {
      if (!focusedCell && (!selectedRowIds || selectedRowIds.size === 0)) return;
      e.preventDefault();

      const selectedRows = rows.filter((r: any) => selectedRowIds?.has(r.id));
      let tsvText = "";

      if (selectedRows.length > 0) {
        tsvText = selectedRows
          .map((r: any) =>
            r
              .getVisibleCells()
              .map((cell: any) => {
                const val = cell.getValue();
                return val != null ? String(val) : "";
              })
              .join("\t")
          )
          .join("\n");
      } else if (focusedCell) {
        const r: any = rows[focusedCell.rowIndex];
        const cell = r?.getVisibleCells()[focusedCell.colIndex];
        if (cell) {
          const val = cell.getValue();
          tsvText = val != null ? String(val) : "";
        }
      }

      if (tsvText) {
        navigator.clipboard
          .writeText(tsvText)
          .catch((err) => console.error("Clipboard copy failed: ", err));
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

    const changes: { rowId: string; columnId: string; value: any }[] = [];
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
        const meta = column.columnDef.meta as LocalSmartTableColumnMeta | undefined;

        if (meta?.editable) {
          const rawValue = grid[i][j];
          let parsedValue: any = rawValue;
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
              rowId: row.id,
              columnId: column.id,
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
          result.catch((err) => console.error("Error saving bulk cells:", err));
        }
      } else if (onCellSave) {
        for (const change of changes) {
          const result = onCellSave(change.rowId, change.columnId, change.value);
          if (result instanceof Promise) {
            await result.catch((err) => console.error("Error saving cell:", err));
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
