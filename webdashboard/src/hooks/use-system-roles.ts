import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchSystemRoles,
  fetchUserSystemRoles,
  assignSystemRole,
  revokeSystemRole,
  AssignSystemRoleRequest,
} from "@/lib/api";

export const systemRoleKeys = {
  all: ["system-roles"] as const,
  user: (userId: string) => ["system-roles", "user", userId] as const,
};

export function useSystemRoles() {
  const queryClient = useQueryClient();

  const rolesQuery = useQuery({
    queryKey: systemRoleKeys.all,
    queryFn: fetchSystemRoles,
  });

  const assignMutation = useMutation({
    mutationFn: ({ userId, request }: { userId: string; request: AssignSystemRoleRequest }) =>
      assignSystemRole(userId, request),
    onSuccess: (_, { userId }) => {
      void queryClient.invalidateQueries({ queryKey: systemRoleKeys.all });
      void queryClient.invalidateQueries({ queryKey: systemRoleKeys.user(userId) });
    },
  });

  const revokeMutation = useMutation({
    mutationFn: ({ userId, roleName }: { userId: string; roleName: string }) =>
      revokeSystemRole(userId, roleName),
    onSuccess: (_, { userId }) => {
      void queryClient.invalidateQueries({ queryKey: systemRoleKeys.all });
      void queryClient.invalidateQueries({ queryKey: systemRoleKeys.user(userId) });
    },
  });

  return {
    roles: rolesQuery.data ?? [],
    isLoading: rolesQuery.isLoading,
    isError: rolesQuery.isError,
    error: rolesQuery.error,
    refetch: rolesQuery.refetch,
    assignRole: assignMutation.mutateAsync,
    isAssigning: assignMutation.isPending,
    revokeRole: revokeMutation.mutateAsync,
    isRevoking: revokeMutation.isPending,
  };
}

export function useUserSystemRoles(userId: string) {
  const query = useQuery({
    queryKey: systemRoleKeys.user(userId),
    queryFn: () => fetchUserSystemRoles(userId),
    enabled: !!userId,
  });

  return {
    roles: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    refetch: query.refetch,
  };
}
