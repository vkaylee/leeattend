import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  fetchWorkspaces,
  createWorkspace,
  fetchWorkspaceMembers,
  addWorkspaceMember,
  removeWorkspaceMember,
  updateWorkspaceMemberRole,
  fetchMemberRoles,
  assignExtraRole,
  revokeExtraRole,
  getCurrentWorkspaceId,
} from "@/lib/api";

export const workspaceKeys = {
  all: ["workspaces"] as const,
  members: (wsId: string) => ["workspaces", wsId, "members"] as const,
  roles: (wsId: string, userId: string) =>
    ["workspaces", wsId, "members", userId, "roles"] as const,
};

export function useWorkspaces() {
  const queryClient = useQueryClient();

  const workspacesQuery = useQuery({
    queryKey: workspaceKeys.all,
    queryFn: fetchWorkspaces,
  });

  const createMutation = useMutation({
    mutationFn: createWorkspace,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
  });

  return {
    workspaces: workspacesQuery.data ?? [],
    isLoading: workspacesQuery.isLoading,
    isError: workspacesQuery.isError,
    createWorkspace: createMutation.mutateAsync,
  };
}

export function useWorkspaceMembers(wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId || getCurrentWorkspaceId();

  const membersQuery = useQuery({
    queryKey: workspaceKeys.members(activeWsId || ""),
    queryFn: () => fetchWorkspaceMembers(activeWsId!),
    enabled: !!activeWsId,
  });

  const addMemberMutation = useMutation({
    mutationFn: ({ email, role }: { email: string; role: string }) =>
      addWorkspaceMember(activeWsId!, email, role),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: workspaceKeys.members(activeWsId!),
      });
    },
  });

  const removeMemberMutation = useMutation({
    mutationFn: (userId: string) => removeWorkspaceMember(activeWsId!, userId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: workspaceKeys.members(activeWsId!),
      });
    },
  });

  return {
    members: membersQuery.data ?? [],
    isLoading: membersQuery.isLoading,
    addMember: addMemberMutation.mutateAsync,
    isAdding: addMemberMutation.isPending,
    removeMember: removeMemberMutation.mutateAsync,
    isRemoving: removeMemberMutation.isPending,
  };
}

export function useMemberRoles(userId: string, wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId || getCurrentWorkspaceId();

  const rolesQuery = useQuery({
    queryKey: workspaceKeys.roles(activeWsId || "", userId),
    queryFn: () => fetchMemberRoles(activeWsId!, userId),
    enabled: !!activeWsId && !!userId,
  });

  const updatePrimaryRoleMutation = useMutation({
    mutationFn: (role: string) =>
      updateWorkspaceMemberRole(activeWsId!, userId, role),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: workspaceKeys.roles(activeWsId!, userId),
      });
      queryClient.invalidateQueries({
        queryKey: workspaceKeys.members(activeWsId!),
      });
    },
  });

  const assignRoleMutation = useMutation({
    mutationFn: ({ role, expiresAt }: { role: string; expiresAt: string }) =>
      assignExtraRole(activeWsId!, userId, role, expiresAt),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: workspaceKeys.roles(activeWsId!, userId),
      });
    },
  });

  const revokeRoleMutation = useMutation({
    mutationFn: (roleId: string) =>
      revokeExtraRole(activeWsId!, userId, roleId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: workspaceKeys.roles(activeWsId!, userId),
      });
    },
  });

  return {
    roles: rolesQuery.data,
    isLoading: rolesQuery.isLoading,
    updatePrimaryRole: updatePrimaryRoleMutation.mutateAsync,
    assignRole: assignRoleMutation.mutateAsync,
    revokeRole: revokeRoleMutation.mutateAsync,
  };
}
