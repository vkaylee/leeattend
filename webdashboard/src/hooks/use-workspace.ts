import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchWorkspaces,
  fetchWorkspace,
  createWorkspace,
  getCurrentWorkspaceId,
} from "@/lib/api";
import { maybeNull } from "@/types/api-overrides";
import { getCurrentWorkspaceId as getStoredWorkspaceId } from "@/lib/api/token-store";

import { workspaceKeys } from "./use-workspace-roles";

export * from "./use-workspace-roles";
export * from "./use-workspace-members";

export function useWorkspaces() {
  const queryClient = useQueryClient();

  const workspacesQuery = useQuery({
    queryKey: workspaceKeys.all,
    queryFn: fetchWorkspaces,
  });

  const createMutation = useMutation({
    mutationFn: createWorkspace,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
  });

  return {
    workspaces: workspacesQuery.data ?? [],
    isLoading: workspacesQuery.isLoading,
    isError: workspacesQuery.isError,
    createWorkspace: createMutation.mutateAsync,
  };
}

export function useWorkspace(wsId?: string) {
  let activeWsId: string | null | undefined = wsId;
  if (!activeWsId) {
    try {
      activeWsId = getCurrentWorkspaceId();
    } catch {
      // Partial API mocks may omit workspace selection in isolated hook tests.
      activeWsId = getStoredWorkspaceId();
    }
  }

  const workspaceQuery = useQuery({
    queryKey: ["workspace", activeWsId],
    queryFn: async () => {
      if (!activeWsId) throw new Error("No workspace ID");
      const res = await fetchWorkspace(activeWsId);
      return maybeNull(res) ?? null;
    },
    enabled: !!activeWsId,
  });

  return {
    workspace: workspaceQuery.data,
    isLoading: workspaceQuery.isLoading,
    isError: workspaceQuery.isError,
  };
}
