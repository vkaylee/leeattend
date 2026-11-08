import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { employeeKeys } from "@/lib/query-keys";
import { useMemo } from "react";

import { searchEmployees } from "@/lib/api";

interface UseReportsEmployeeSearchProps {
  debouncedSearch: string;
}

export function useReportsEmployeeSearch({ debouncedSearch }: UseReportsEmployeeSearchProps) {
  const { data: searchResults } = useOrganizationQuery({ queryKey: employeeKeys.search({ search: debouncedSearch }),
  queryFn: () => searchEmployees({ query: debouncedSearch }),
  enabled: debouncedSearch.trim().length > 0, });

  const employeeIdsFilter = useMemo(() => {
    if (!debouncedSearch.trim()) return undefined;
    if (!searchResults?.data.length) return ["__none__"];
    return searchResults.data.map((emp) => emp.id);
  }, [debouncedSearch, searchResults]);

  return { employeeIdsFilter };
}
