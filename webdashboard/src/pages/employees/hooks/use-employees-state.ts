import { useState } from "react";

import { useDebounce } from "@/hooks/use-debounce";

export type SortKey = "name" | "id" | "department" | "status";
export type SortDir = "asc" | "desc";

export const useEmployeesState = () => {
  const [searchTerm, setSearchTerm] = useState("");
  const [filterDeptIds, setFilterDeptIds] = useState<string[]>([]);
  const [sortKey, setSortKey] = useState<SortKey | null>(null);
  const [sortDir, setSortDir] = useState<SortDir>("asc");
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

  return {
    searchTerm,
    setSearchTerm,
    filterDeptIds,
    setFilterDeptIds,
    sortKey,
    setSortKey,
    sortDir,
    setSortDir,
    debouncedSearch,
    toggleSort,
    toggleDeptFilter,
  };
};
