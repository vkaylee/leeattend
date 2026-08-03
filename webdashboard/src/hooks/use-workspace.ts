import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchWorkspaces,
  fetchWorkspace,
  createWorkspace,
  getCurrentWorkspaceId,
} from "@/lib/api";

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
  const activeWsId = wsId ?? getCurrentWorkspaceId();

  const workspaceQuery = useQuery({
    queryKey: ["workspace", activeWsId],
    queryFn: async () => {
      const res = await fetchWorkspace(activeWsId ?? "");
      return res ?? null;
    },
    enabled: !!activeWsId,
  });

  return {
    workspace: workspaceQuery.data,
    isLoading: workspaceQuery.isLoading,
    isError: workspaceQuery.isError,
  };
}
