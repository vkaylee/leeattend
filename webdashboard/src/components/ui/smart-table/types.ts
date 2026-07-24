/**
 * SmartTable shared types.
 *
 * Central type definitions used by the SmartTable component tree
 * (smart-table.tsx, hooks, row/header renderers).
 * Extracted to a standalone module to avoid circular imports between
 * the root component and its hook dependencies.
 */

// ── Cell value types ──────────────────────────────────────────────

/** Union of all value types supported by inline cell editing. */
export type EditableCellValue = string | number | boolean | null | undefined;

/** Payload for a single cell save operation. */
export interface CellEditPayload {
  columnId: string;
  rowId: string;
  value: EditableCellValue;
}

/** Grid coordinate used by keyboard navigation, drag-select, and focus. */
export interface CellCoordinate {
  colIndex: number;
  rowIndex: number;
}

// ── Column metadata ───────────────────────────────────────────────

/**
 * Per-column metadata consumed by SmartTable's inline-editing,
 * clipboard, and sticky-column features.
 *
 * This is the **single source of truth** — hooks and row renderers
 * import it from here instead of duplicating a local copy.
 */
export interface SmartTableColumnMeta {
  cellClassName?: string;
  editable?: boolean;
  editOptions?: { value: string; label: string }[];
  editType?: "text" | "number" | "select";
  headerClassName?: string;
  sticky?: "left" | "right";
}

// ── Context menu ──────────────────────────────────────────────────

export interface ContextMenuItem {
  disabled?: boolean;
  icon?: React.ReactNode;
  label: string;
  onClick: () => void;
}
