import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchOrganizationMembers,
  addOrganizationMember,
  removeOrganizationMember,
  updateOrganizationMemberRole,
  fetchMemberRoles,
  assignExtraRole,
  revokeExtraRole,
  fetchMemberPermissions,
  assignExtraPermission,
  revokeExtraPermission,
  fetchPermissionCatalog,
} from "@/lib/api";

import { organizationKeys } from "./use-organization-queries";

export function useOrganizationMembers() {
  const queryClient = useQueryClient();

  const membersQuery = useQuery({
    queryKey: organizationKeys.members(),
    queryFn: () => fetchOrganizationMembers(),
  });

  const addMemberMutation = useMutation({
    mutationFn: ({ email, roleId }: { email: string; roleId: string }) =>
      addOrganizationMember(email, roleId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.members(),
      });
    },
  });

  const removeMemberMutation = useMutation({
    mutationFn: (userId: string) => removeOrganizationMember(userId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.members(),
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

export function useMemberRoles(userId: string) {
  const queryClient = useQueryClient();

  const rolesQuery = useQuery({
    queryKey: organizationKeys.memberRoles(userId),
    queryFn: () => fetchMemberRoles(userId),
    enabled: !!userId,
  });

  const updatePrimaryRoleMutation = useMutation({
    mutationFn: (role: string) =>
      updateOrganizationMemberRole(userId, role),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.memberRoles(userId),
      });
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.members(),
      });
    },
  });

  const assignRoleMutation = useMutation({
    mutationFn: ({ role, expiresAt }: { role: string; expiresAt: string }) =>
      assignExtraRole(userId, role, expiresAt),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.memberRoles(userId),
      });
    },
  });

  const revokeRoleMutation = useMutation({
    mutationFn: (roleId: string) =>
      revokeExtraRole(userId, roleId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.memberRoles(userId),
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

export function useMemberPermissions(userId: string) {
  const queryClient = useQueryClient();

  const permissionsQuery = useQuery({
    queryKey: organizationKeys.memberPermissions(userId),
    queryFn: () => fetchMemberPermissions(userId),
    enabled: !!userId,
  });

  const catalogQuery = useQuery({
    queryKey: organizationKeys.permissionCatalog(),
    queryFn: () => fetchPermissionCatalog(),
    staleTime: 5 * 60 * 1000,
  });

  const assignMutation = useMutation({
    mutationFn: ({ permission, expiresAt }: { permission: string; expiresAt: string }) =>
      assignExtraPermission(userId, permission, expiresAt),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.memberPermissions(userId),
      });
    },
  });

  const revokeMutation = useMutation({
    mutationFn: (permId: string) =>
      revokeExtraPermission(userId, permId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: organizationKeys.memberPermissions(userId),
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
