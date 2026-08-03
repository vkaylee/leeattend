export interface CellNavigationOptions {
  colCount: number;
  colIndex: number;
  key: string;
  rowCount: number;
  rowIndex: number;
  shiftKey: boolean;
}

export interface CellNavigationResult {
  /** Whether the key was recognized and handled. */
  handled: boolean;
  /** Target column index (unchanged if `handled` is false). */
  nextCol: number;
  /** Target row index (unchanged if `handled` is false). */
  nextRow: number;
}

/**
 * Pure reducer: compute the next cell coordinate for keyboard navigation.
 *
 * Handles ArrowUp/Down/Left/Right (clamped to grid bounds) and Tab (with
 * row-wrapping: after the last column, Tab moves to the next row's first
 * column; Shift+Tab wraps to the previous row's last column).
 *
 * Returns `handled: false` for unrecognized keys — the caller should pass the
 * event through instead of consuming it.
 */
export function computeCellNavigation({
  key,
  rowIndex,
  colIndex,
  shiftKey,
  rowCount,
  colCount,
}: CellNavigationOptions): CellNavigationResult {
  let nextRow = rowIndex;
  let nextCol = colIndex;
  let handled = false;

  switch (key) {
    case "ArrowUp":
      nextRow = Math.max(0, rowIndex - 1);
      handled = true;
      break;
    case "ArrowDown":
      nextRow = Math.min(rowCount - 1, rowIndex + 1);
      handled = true;
      break;
    case "ArrowLeft":
      nextCol = Math.max(0, colIndex - 1);
      handled = true;
      break;
    case "ArrowRight":
      nextCol = Math.min(colCount - 1, colIndex + 1);
      handled = true;
      break;
    case "Tab":
      if (shiftKey) {
        nextCol = colIndex - 1;
        if (nextCol < 0) {
          nextCol = colCount - 1;
          nextRow = Math.max(0, rowIndex - 1);
        }
      } else {
        nextCol = colIndex + 1;
        if (nextCol >= colCount) {
          nextCol = 0;
          nextRow = Math.min(rowCount - 1, rowIndex + 1);
        }
      }
      handled = true;
      break;
    default:
      break;
  }
  return { nextCol, nextRow, handled };
}
