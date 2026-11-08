import { useQuery } from "@tanstack/react-query";

import { fetchDepartments } from "@/lib/api";
import { departmentKeys } from "@/lib/query-keys";

export function useDepartments(options?: { enabled?: boolean }) {
  return useQuery({
    queryKey: departmentKeys.all(),
    queryFn: () => fetchDepartments(),
    enabled: options?.enabled,
  });
}
