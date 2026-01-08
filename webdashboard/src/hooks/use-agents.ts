import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  fetchAuthorizedAgents,
  createAuthorizedAgent,
  updateAgentTrust,
  deleteAuthorizedAgent,
  updateAuthorizedAgent,
  fetchAgents,
  AuthorizedAgent,
} from "@/lib/api";

export const agentsKeys = {
  authorized: (includeInactive: boolean) =>
    ["authorized-agents", { includeInactive }] as const,
  status: ["agents-status"] as const,
};

export function useAuthorizedAgents() {
  const queryClient = useQueryClient();
  const [includeInactive, setIncludeInactive] = useState(false);

  const agentsQuery = useQuery({
    queryKey: agentsKeys.authorized(includeInactive),
    queryFn: () => fetchAuthorizedAgents(includeInactive),
  });

  const createMutation = useMutation({
    mutationFn: createAuthorizedAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["authorized-agents"] });
    },
  });

  const updateTrustMutation = useMutation({
    mutationFn: ({ id, is_trusted }: { id: string; is_trusted: boolean }) =>
      updateAgentTrust(id, is_trusted),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["authorized-agents"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteAuthorizedAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["authorized-agents"] });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({
      id,
      agent,
    }: {
      id: string;
      agent: Partial<AuthorizedAgent>;
    }) => updateAuthorizedAgent(id, agent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["authorized-agents"] });
    },
  });

  return {
    agents: agentsQuery.data ?? [],
    isLoading: agentsQuery.isLoading,
    isFetching: agentsQuery.isFetching,
    error: agentsQuery.error,
    refetch: agentsQuery.refetch,
    includeInactive,
    setIncludeInactive,

    createAgent: createMutation.mutateAsync,
    updateTrust: updateTrustMutation.mutateAsync,
    deleteAgent: deleteMutation.mutateAsync,
    updateAgent: updateMutation.mutateAsync,

    isCreating: createMutation.isPending,
    isUpdatingTrust: updateTrustMutation.isPending,
    isDeleting: deleteMutation.isPending,
    isUpdating: updateMutation.isPending,
  };
}

export function useAgentsStatus() {
  const agentsStatusQuery = useQuery({
    queryKey: agentsKeys.status,
    queryFn: fetchAgents,
    refetchInterval: 30000, // Tự động làm mới mỗi 30s
  });

  return {
    agents: agentsStatusQuery.data ?? [],
    isLoading: agentsStatusQuery.isLoading,
    isFetching: agentsStatusQuery.isFetching,
    error: agentsStatusQuery.error,
    refetch: agentsStatusQuery.refetch,
  };
}
