import { useQuery } from "@tanstack/react-query";

import { fetchWorkspace } from "@/lib/api";
import { getStandaloneOrganizationId } from "@/lib/api/config";
import { maybeNull } from "@/types/api-overrides";

import { workspaceKeys } from "./use-workspace-roles";

export * from "./use-workspace-roles";
export * from "./use-workspace-members";


export function useWorkspace(wsId?: string) {
  const activeWsId = wsId ?? getStandaloneOrganizationId();
  const workspaceQuery = useQuery({
    queryKey: ["workspace", activeWsId],
    queryFn: async () => {
      if (!activeWsId) throw new Error("No workspace ID");
      const workspace = await fetchWorkspace(activeWsId);
      return maybeNull(workspace) ?? null;
    },
    enabled: !!activeWsId,
  });

  return {
    workspace: workspaceQuery.data,
    isLoading: workspaceQuery.isLoading,
    isError: workspaceQuery.isError,
  };
}
