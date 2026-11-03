import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchWorkspaceMembers,
  addWorkspaceMember,
  removeWorkspaceMember,
  updateWorkspaceMemberRole,
  fetchMemberRoles,
  assignExtraRole,
  revokeExtraRole,
  fetchMemberPermissions,
  assignExtraPermission,
  revokeExtraPermission,
  fetchPermissionCatalog,
  getStandaloneOrganizationId,
} from "@/lib/api";

import { workspaceKeys } from "./use-workspace-roles";

export function useWorkspaceMembers(wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId ?? getStandaloneOrganizationId();

  const membersQuery = useQuery({
    queryKey: workspaceKeys.members(activeWsId ?? ""),
    queryFn: () => fetchWorkspaceMembers((activeWsId ?? "")),
    enabled: !!activeWsId,
  });

  const addMemberMutation = useMutation({
    mutationFn: ({ email, roleId }: { email: string; roleId: string }) =>
      addWorkspaceMember((activeWsId ?? ""), email, roleId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.members((activeWsId ?? "")),
      });
    },
  });

  const removeMemberMutation = useMutation({
    mutationFn: (userId: string) => removeWorkspaceMember((activeWsId ?? ""), userId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.members((activeWsId ?? "")),
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
  const activeWsId = wsId ?? getStandaloneOrganizationId();

  const rolesQuery = useQuery({
    queryKey: workspaceKeys.roles(activeWsId ?? "", userId),
    queryFn: () => fetchMemberRoles((activeWsId ?? ""), userId),
    enabled: !!activeWsId && !!userId,
  });

  const updatePrimaryRoleMutation = useMutation({
    mutationFn: (role: string) =>
      updateWorkspaceMemberRole((activeWsId ?? ""), userId, role),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.roles((activeWsId ?? ""), userId),
      });
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.members((activeWsId ?? "")),
      });
    },
  });

  const assignRoleMutation = useMutation({
    mutationFn: ({ role, expiresAt }: { role: string; expiresAt: string }) =>
      assignExtraRole((activeWsId ?? ""), userId, role, expiresAt),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.roles((activeWsId ?? ""), userId),
      });
    },
  });

  const revokeRoleMutation = useMutation({
    mutationFn: (roleId: string) =>
      revokeExtraRole((activeWsId ?? ""), userId, roleId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.roles((activeWsId ?? ""), userId),
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

export function useMemberPermissions(userId: string, wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId ?? getStandaloneOrganizationId();

  const permissionsQuery = useQuery({
    queryKey: workspaceKeys.permissions(activeWsId ?? "", userId),
    queryFn: () => fetchMemberPermissions((activeWsId ?? ""), userId),
    enabled: !!activeWsId && !!userId,
  });

  const catalogQuery = useQuery({
    queryKey: workspaceKeys.permissionCatalog(activeWsId ?? ""),
    queryFn: () => fetchPermissionCatalog(activeWsId ?? ""),
    enabled: !!activeWsId,
    staleTime: 5 * 60 * 1000,
  });

  const assignMutation = useMutation({
    mutationFn: ({ permission, expiresAt }: { permission: string; expiresAt: string }) =>
      assignExtraPermission((activeWsId ?? ""), userId, permission, expiresAt),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.permissions((activeWsId ?? ""), userId),
      });
    },
  });

  const revokeMutation = useMutation({
    mutationFn: (permId: string) =>
      revokeExtraPermission((activeWsId ?? ""), userId, permId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.permissions((activeWsId ?? ""), userId),
      });
    },
  });

  return {
    permissions: permissionsQuery.data ?? [],
    catalog: catalogQuery.data ?? [],
    isLoading: permissionsQuery.isLoading || catalogQuery.isLoading,
    isCatalogError: catalogQuery.isError,
    assignPermission: assignMutation.mutateAsync,
    isAssigning: assignMutation.isPending,
    revokePermission: revokeMutation.mutateAsync,
    isRevoking: revokeMutation.isPending,
  };
}
