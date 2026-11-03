import { useQuery } from "@tanstack/react-query";

import { fetchWorkspace } from "@/lib/api";
import { maybeNull } from "@/types/api-overrides";
import { getCurrentWorkspaceId as getStoredWorkspaceId } from "@/lib/api/token-store";

import { workspaceKeys } from "./use-workspace-roles";

export * from "./use-workspace-roles";
export * from "./use-workspace-members";


export function useWorkspace(wsId?: string) {
  const activeWsId = wsId ?? getStoredWorkspaceId();
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
