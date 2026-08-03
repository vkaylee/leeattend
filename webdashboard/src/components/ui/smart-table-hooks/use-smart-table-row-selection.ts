import type { RowSelectionState } from "@tanstack/react-table";

import { useMemo, useCallback } from "react";

interface UseSmartTableRowSelectionProps<T> {
  data: T[];
  getRowId: (row: T) => string;
  onSelectionChange?: (selectedIds: Set<string>) => void;
  selectedRowIds?: Set<string>;
}

export function useSmartTableRowSelection<T>({
  selectedRowIds,
  data,
  getRowId,
  onSelectionChange,
}: UseSmartTableRowSelectionProps<T>) {
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

  const handleRowSelectionChange = useCallback((
    updaterOrValue: RowSelectionState | ((old: RowSelectionState) => RowSelectionState)
  ) => {
    if (!onSelectionChange) return;
    const newSelection = typeof updaterOrValue === "function" ? updaterOrValue(rowSelection) : updaterOrValue;
    const nextSelectedSet = new Set<string>();
    Object.keys(newSelection).forEach((key) => {
      if (newSelection[key]) {
        nextSelectedSet.add(key);
      }
    });
    onSelectionChange(nextSelectedSet);
  }, [onSelectionChange, rowSelection]);

  return { rowSelection, handleRowSelectionChange };
}
