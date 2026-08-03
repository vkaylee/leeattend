import type { ColumnDef, ColumnPinningState } from "@tanstack/react-table";

import type { SmartTableColumnMeta } from "../smart-table/types";

/** Compute column id from a column definition, handling both accessorKey and id variants. */
export function computeColumnId<T>(col: ColumnDef<T>, index: number): string {
  if (col.id) return col.id;
  if ("accessorKey" in col) return col.accessorKey as string;
  return `col_${index}`;
}

/** Determine if a column should be pinned left based on meta or sticky count. */
export function shouldPinLeft(col: { meta?: SmartTableColumnMeta }, index: number, stickyLeftCount: number): boolean {
  if (col.meta?.sticky === "left") return true;
  return stickyLeftCount > 0 && index < stickyLeftCount;
}

/** Determine if a column should be pinned right based on meta or sticky count. */
export function shouldPinRight(col: { meta?: SmartTableColumnMeta }, index: number, totalCols: number, stickyRightCount: number): boolean {
  if (col.meta?.sticky === "right") return true;
  return stickyRightCount > 0 && index >= totalCols - stickyRightCount;
}

/** Build column pinning state from column definitions and sticky counts. */
export function buildColumnPinning<T>(columns: (ColumnDef<T> & { meta?: SmartTableColumnMeta })[], stickyLeftColumns?: number, stickyRightColumns?: number): ColumnPinningState {
  const left: string[] = [];
  const right: string[] = [];
  const sLC = stickyLeftColumns ?? 0;
  const sRC = stickyRightColumns ?? 0;
  columns.forEach((col, index) => {
    const id = computeColumnId(col, index);
    if (shouldPinLeft(col, index, sLC)) left.push(id);
    if (shouldPinRight(col, index, columns.length, sRC)) right.push(id);
  });
  return { left, right };
}
