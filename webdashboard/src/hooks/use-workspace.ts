import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchWorkspaces,
  fetchWorkspace,
  createWorkspace,
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
  fetchAvailablePermissions,
  getCurrentWorkspaceId,
  fetchWorkspaceRoles,
  createWorkspaceRole,
  updateWorkspaceRole,
  deleteWorkspaceRole,
  fetchRolePermissions,
  updateRolePermissions,
} from "@/lib/api";

export const workspaceKeys = {
  all: ["workspaces"] as const,
  members: (wsId: string) => ["workspaces", wsId, "members"] as const,
  roles: (wsId: string, userId: string) =>
    ["workspaces", wsId, "members", userId, "roles"] as const,
  permissions: (wsId: string, userId: string) =>
    ["workspaces", wsId, "members", userId, "permissions"] as const,
  availablePermissions: (wsId: string) =>
    ["workspaces", wsId, "permissions", "available"] as const,
  workspaceRoles: (wsId: string) => ["workspaces", wsId, "roles"] as const,
  rolePermissions: (wsId: string, roleId: string) => ["workspaces", wsId, "roles", roleId, "permissions"] as const,
  permissionsMatrix: (wsId: string) => ["workspaces", wsId, "permissions", "matrix"] as const,
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
    queryFn: () => fetchWorkspace((activeWsId ?? "")),
    enabled: !!activeWsId,
  });

  return {
    workspace: workspaceQuery.data,
    isLoading: workspaceQuery.isLoading,
    isError: workspaceQuery.isError,
  };
}

export function useWorkspaceRoles(wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId ?? getCurrentWorkspaceId();

  const rolesQuery = useQuery({
    queryKey: workspaceKeys.workspaceRoles(activeWsId ?? ""),
    queryFn: () => fetchWorkspaceRoles((activeWsId ?? "")),
    enabled: !!activeWsId,
    staleTime: 5 * 60 * 1000,
  });

  const createRoleMutation = useMutation({
    mutationFn: (data: { name: string; description?: string; color: string }) => createWorkspaceRole((activeWsId ?? ""), data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: workspaceKeys.workspaceRoles((activeWsId ?? "")) });
    },
  });

  const updateRoleMutation = useMutation({
    mutationFn: ({ roleId, data }: { roleId: string; data: { name?: string; description?: string; color?: string } }) => updateWorkspaceRole((activeWsId ?? ""), roleId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: workspaceKeys.workspaceRoles((activeWsId ?? "")) });
    },
  });

  const deleteRoleMutation = useMutation({
    mutationFn: (roleId: string) => deleteWorkspaceRole((activeWsId ?? ""), roleId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: workspaceKeys.workspaceRoles((activeWsId ?? "")) });
    },
  });

  return {
    roles: rolesQuery.data ?? [],
    isLoading: rolesQuery.isLoading,
    createRole: createRoleMutation.mutateAsync,
    isCreating: createRoleMutation.isPending,
    updateRole: updateRoleMutation.mutateAsync,
    isUpdating: updateRoleMutation.isPending,
    deleteRole: deleteRoleMutation.mutateAsync,
    isDeleting: deleteRoleMutation.isPending,
  };
}

export function useWorkspaceMembers(wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId ?? getCurrentWorkspaceId();

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
  const activeWsId = wsId ?? getCurrentWorkspaceId();

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
  const activeWsId = wsId ?? getCurrentWorkspaceId();

  const permissionsQuery = useQuery({
    queryKey: workspaceKeys.permissions(activeWsId ?? "", userId),
    queryFn: () => fetchMemberPermissions((activeWsId ?? ""), userId),
    enabled: !!activeWsId && !!userId,
  });

  const availableQuery = useQuery({
    queryKey: workspaceKeys.availablePermissions(activeWsId ?? ""),
    queryFn: () => fetchAvailablePermissions((activeWsId ?? "")),
    enabled: !!activeWsId,
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
    available: availableQuery.data ?? [],
    isLoading: permissionsQuery.isLoading,
    assignPermission: assignMutation.mutateAsync,
    isAssigning: assignMutation.isPending,
    revokePermission: revokeMutation.mutateAsync,
    isRevoking: revokeMutation.isPending,
  };
}

export function useRolePermissions(roleId: string, wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId ?? getCurrentWorkspaceId();

  const permissionsQuery = useQuery({
    queryKey: workspaceKeys.rolePermissions(activeWsId ?? "", roleId),
    queryFn: () => fetchRolePermissions((activeWsId ?? ""), roleId),
    enabled: !!activeWsId && !!roleId,
  });

  const updateMutation = useMutation({
    mutationFn: (permissions: string[]) => updateRolePermissions((activeWsId ?? ""), roleId, permissions),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: workspaceKeys.rolePermissions((activeWsId ?? ""), roleId),
      });
    },
  });

  return {
    permissions: permissionsQuery.data ?? [],
    isLoading: permissionsQuery.isLoading,
    updatePermissions: updateMutation.mutateAsync,
    isUpdating: updateMutation.isPending,
  };
}

const EMPTY_MATRIX: Record<string, string[]> = {};

export function useWorkspacePermissionsMatrix(wsId?: string) {
  const activeWsId = wsId ?? getCurrentWorkspaceId();
  const { roles } = useWorkspaceRoles(activeWsId ?? undefined);

  const roleIds = roles.map(r => r.id).sort().join(',');

  const matrixQuery = useQuery({
    queryKey: [...workspaceKeys.permissionsMatrix(activeWsId ?? ""), roleIds],
    queryFn: async () => {
      const matrix: Record<string, string[]> = {};
      await Promise.all(roles.map(async (role) => {
        try {
          const perms = await fetchRolePermissions((activeWsId ?? ""), role.id);
          matrix[role.id] = perms;
        } catch {
          matrix[role.id] = [];
        }
      }));
      return matrix;
    },
    enabled: !!activeWsId && roles.length > 0,
  });

  return {
    matrix: matrixQuery.data ?? EMPTY_MATRIX,
    isLoading: matrixQuery.isLoading,
    refetch: matrixQuery.refetch,
  };
}
