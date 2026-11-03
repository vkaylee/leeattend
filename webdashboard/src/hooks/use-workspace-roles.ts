import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchWorkspaceRoles,
  createWorkspaceRole,
  updateWorkspaceRole,
  deleteWorkspaceRole,
  fetchRolePermissions,
  fetchPermissionCatalog,
  updateRolePermissions,
  getStandaloneOrganizationId,
} from "@/lib/api";
import { logError } from "@/lib/logger";

export const workspaceKeys = {
  all: ["workspaces"] as const,
  members: (wsId: string) => ["workspaces", wsId, "members"] as const,
  roles: (wsId: string, userId: string) =>
    ["workspaces", wsId, "members", userId, "roles"] as const,
  permissions: (wsId: string, userId: string) =>
    ["workspaces", wsId, "members", userId, "permissions"] as const,
  availablePermissions: (wsId: string) =>
    ["workspaces", wsId, "permissions", "available"] as const,
  permissionCatalog: (wsId: string) =>
    ["workspaces", wsId, "permissions", "catalog"] as const,
  workspaceRoles: (wsId: string) => ["workspaces", wsId, "roles"] as const,
  rolePermissions: (wsId: string, roleId: string) => ["workspaces", wsId, "roles", roleId, "permissions"] as const,
  permissionsMatrix: (wsId: string) => ["workspaces", wsId, "permissions", "matrix"] as const,
};

export function usePermissionCatalog(wsId?: string) {
  const activeWsId = wsId ?? getStandaloneOrganizationId();
  const catalogQuery = useQuery({
    queryKey: workspaceKeys.permissionCatalog(activeWsId ?? ""),
    queryFn: () => fetchPermissionCatalog(activeWsId ?? ""),
    enabled: !!activeWsId,
    staleTime: 5 * 60 * 1000,
  });

  return {
    catalog: catalogQuery.data ?? [],
    isLoading: catalogQuery.isLoading,
    isError: catalogQuery.isError,
  };
}

export function useWorkspaceRoles(wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId ?? getStandaloneOrganizationId();

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

export function useRolePermissions(roleId: string, wsId?: string) {
  const queryClient = useQueryClient();
  const activeWsId = wsId ?? getStandaloneOrganizationId();

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
  const activeWsId = wsId ?? getStandaloneOrganizationId();
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
        } catch (e: unknown) {
          logError(e, { component: 'useWorkspacePermissionsMatrix' });
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
