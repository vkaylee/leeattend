import { useState, useCallback, useMemo } from "react";

import type { Employee } from "@/lib/api/types";

export function useEmployeeBulkSelection(employees: Employee[]) {
  const [selectedIds, setSelectedIds] = useState<string[]>([]);

  const toggleSelect = useCallback((id: string) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id]
    );
  }, []);

  const selectAll = useCallback(() => {
    setSelectedIds(employees.map((e) => e.id));
  }, [employees]);

  const clearSelection = useCallback(() => {
    setSelectedIds([]);
  }, []);

  const toggleSelectAll = useCallback(() => {
    if (selectedIds.length === employees.length && employees.length > 0) {
      clearSelection();
    } else {
      selectAll();
    }
  }, [selectedIds.length, employees.length, clearSelection, selectAll]);

  const isAllSelected = useMemo(
    () => employees.length > 0 && selectedIds.length === employees.length,
    [employees.length, selectedIds.length]
  );

  const isIndeterminate = useMemo(
    () => selectedIds.length > 0 && selectedIds.length < employees.length,
    [selectedIds.length, employees.length]
  );

  return {
    clearSelection,
    isAllSelected,
    isIndeterminate,
    selectAll,
    selectedIds,
    toggleSelect,
    toggleSelectAll,
  };
}
