import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchOrganizationRoles,
  createOrganizationRole,
  updateOrganizationRole,
  deleteOrganizationRole,
  fetchRolePermissions,
  fetchPermissionCatalog,
  updateRolePermissions,
} from "@/lib/api";
import { logError } from "@/lib/logger";

import { organizationKeys } from "./use-organization-queries";
import { createQueryKey } from "@/lib/query-keys";


export function usePermissionCatalog() {
  const catalogQuery = useQuery({
    queryKey: organizationKeys.permissionCatalog(),
    queryFn: () => fetchPermissionCatalog(),
    staleTime: 5 * 60 * 1000,
  });

  return {
    catalog: catalogQuery.data ?? [],
    isLoading: catalogQuery.isLoading,
    isError: catalogQuery.isError,
  };
}

export function useOrganizationRoles() {
  const queryClient = useQueryClient();

  const rolesQuery = useQuery({
    queryKey: organizationKeys.roles(),
    queryFn: () => fetchOrganizationRoles(),
    staleTime: 5 * 60 * 1000,
  });

  const createRoleMutation = useMutation({
    mutationFn: (data: { name: string; description?: string; color: string }) => createOrganizationRole(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: organizationKeys.roles() });
    },
  });

  const updateRoleMutation = useMutation({
    mutationFn: ({ roleId, data }: { roleId: string; data: { name?: string; description?: string; color?: string } }) => updateOrganizationRole(roleId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: organizationKeys.roles() });
    },
  });

  const deleteRoleMutation = useMutation({
    mutationFn: (roleId: string) => deleteOrganizationRole(roleId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: organizationKeys.roles() });
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

export function useRolePermissions(roleId: string) {
  const queryClient = useQueryClient();

  const permissionsQuery = useQuery({
    queryKey: organizationKeys.rolePermissions(roleId),
    queryFn: () => fetchRolePermissions(roleId),
    enabled: !!roleId,
  });

  const updateMutation = useMutation({
    mutationFn: (permissions: string[]) => updateRolePermissions(roleId, permissions),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.rolePermissions(roleId),
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

export function useOrganizationPermissionsMatrix() {
  const { roles, isLoading: rolesLoading } = useOrganizationRoles();

  const roleIds = roles.map((role) => role.id).sort().join(",");

  const matrixQuery = useQuery({
    queryKey: createQueryKey(...organizationKeys.permissionsMatrix(), roleIds),
    queryFn: async () => {
      const matrix: Record<string, string[]> = {};
      await Promise.all(roles.map(async (role) => {
        try {
          const perms = await fetchRolePermissions(role.id);
          matrix[role.id] = perms;
        } catch (e: unknown) {
          logError(e, { component: "useOrganizationPermissionsMatrix" });
          matrix[role.id] = [];
        }
      }));
      return matrix;
    },
    enabled: roles.length > 0,
  });

  return {
    matrix: matrixQuery.data ?? EMPTY_MATRIX,
    isLoading: rolesLoading || matrixQuery.isLoading,
    refetch: matrixQuery.refetch,
  };
}
