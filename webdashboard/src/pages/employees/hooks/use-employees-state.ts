import { useState } from "react";

import { useDebounce } from "@/hooks/use-debounce";

export type SortKey = "name" | "id" | "department";
export type SortDir = "asc" | "desc";

export const useEmployeesState = () => {
  const [searchTerm, setSearchTerm] = useState("");
  const [filterDeptIds, setFilterDeptIds] = useState<string[]>([]);
  const [filterStatus, setFilterStatus] = useState<string>("active");
  const [sortKey, setSortKey] = useState<SortKey | null>(null);
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [customFieldFilters, setCustomFieldFilters] = useState<Record<string, string>>({});
  const debouncedSearch = useDebounce(searchTerm, 500);

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const toggleDeptFilter = (deptId: string) => {
    setFilterDeptIds(previous =>
      previous.includes(deptId) ? previous.filter(id => id !== deptId) : [...previous, deptId]
    );
  };

  const updateCustomFieldFilter = (key: string, val: string) => {
    setCustomFieldFilters((prev) => {
      const next = { ...prev };
      if (!val.trim()) {
        delete next[key];
      } else {
        next[key] = val;
      }
      return next;
    });
  };

  return {
    searchTerm,
    setSearchTerm,
    filterDeptIds,
    setFilterDeptIds,
    filterStatus,
    setFilterStatus,
    sortKey,
    setSortKey,
    sortDir,
    setSortDir,
    customFieldFilters,
    setCustomFieldFilters,
    updateCustomFieldFilter,
    debouncedSearch,
    toggleSort,
    toggleDeptFilter,
  };
};
