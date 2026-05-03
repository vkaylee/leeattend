import { useState } from "react";

import { Employee } from "@/lib/api";

export function usePayrollSelection(employees: Employee[]) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const toggleSelectAll = (ids?: string[]) => {
    if (ids) {
      const next = new Set(selectedIds);
      const allIncluded = ids.every(id => next.has(id));
      if (allIncluded) {
        ids.forEach(id => next.delete(id));
      } else {
        ids.forEach(id => next.add(id));
      }
      setSelectedIds(next);
      return;
    }

    if (selectedIds.size === employees.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(employees.map(e => e.id)));
    }
  };

  const toggleSelect = (id: string) => {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    setSelectedIds(next);
  };

  return {
    selectedIds,
    setSelectedIds,
    toggleSelect,
    toggleSelectAll,
  };
}
