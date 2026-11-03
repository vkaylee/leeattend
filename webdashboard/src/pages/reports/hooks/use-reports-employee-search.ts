import { useOrganizationQuery } from "@/hooks/common/use-workspace-query";
import { employeeKeys } from "@/lib/query-keys";
import { useMemo } from "react";

import { searchEmployees } from "@/lib/api";

interface UseReportsEmployeeSearchProps {
  debouncedSearch: string;
  wsId: string | null;
}

export function useReportsEmployeeSearch({ wsId, debouncedSearch }: UseReportsEmployeeSearchProps) {
  const { data: searchResults } = useOrganizationQuery({
    workspaceId: wsId,
    queryKey: employeeKeys.search(wsId, { search: debouncedSearch }),
    queryFn: (workspaceId) => searchEmployees({ wsId: workspaceId, query: debouncedSearch }),
    enabled: debouncedSearch.trim().length > 0,
  });

  const employeeIdsFilter = useMemo(() => {
    if (!debouncedSearch.trim()) return undefined;
    if (!searchResults?.data.length) return ["__none__"];
    return searchResults.data.map((emp) => emp.id);
  }, [debouncedSearch, searchResults]);

  return { employeeIdsFilter };
}
