import { fetchDepartments, getCurrentWorkspaceId } from "@/lib/api";
import { departmentKeys } from "@/lib/query-keys";

import { useWorkspaceQuery } from "./common/use-workspace-query";

export function useDepartments(
  wsId: string | null | undefined = getCurrentWorkspaceId(),
  options?: { enabled?: boolean },
) {

  return useWorkspaceQuery({
    workspaceId: wsId,
    queryKey: departmentKeys.all(wsId),
    queryFn: (workspaceId) => fetchDepartments(workspaceId),
    enabled: options?.enabled,
  });
}
