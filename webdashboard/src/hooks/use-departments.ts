import { fetchDepartments, getStandaloneOrganizationId } from "@/lib/api";
import { departmentKeys } from "@/lib/query-keys";

import { useOrganizationQuery } from "./common/use-workspace-query";

export function useDepartments(
  wsId: string | null | undefined = getStandaloneOrganizationId(),
  options?: { enabled?: boolean },
) {

  return useOrganizationQuery({
    workspaceId: wsId,
    queryKey: departmentKeys.all(wsId),
    queryFn: (workspaceId) => fetchDepartments(workspaceId),
    enabled: options?.enabled,
  });
}
