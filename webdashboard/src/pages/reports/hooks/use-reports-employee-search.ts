import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { searchEmployees } from "@/lib/api";

interface UseReportsEmployeeSearchProps {
  debouncedSearch: string;
  wsId: string | null;
}

export function useReportsEmployeeSearch({ wsId, debouncedSearch }: UseReportsEmployeeSearchProps) {
  const { data: searchResults } = useQuery({
    queryKey: ["employees", wsId, "search", debouncedSearch],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return searchEmployees({ wsId, query: debouncedSearch });
    },
    enabled: !!wsId && debouncedSearch.trim().length > 0,
  });

  const employeeIdsFilter = useMemo(() => {
    if (!debouncedSearch.trim()) return undefined;
    if (!searchResults?.data.length) return ["__none__"];
    return searchResults.data.map((emp) => emp.id);
  }, [debouncedSearch, searchResults]);

  return { employeeIdsFilter };
}
