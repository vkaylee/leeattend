import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { CreateContractRequest, UpdateContractRequest } from "@/lib/api";

import {
  fetchEmployeeContracts,
  createEmployeeContract,
  updateEmployeeContract,
  deleteEmployeeContract,
} from "@/lib/api";

export function useEmployeeContracts(wsId: string, employeeId: string) {
  const queryClient = useQueryClient();
  const queryKey = ["employee-contracts", wsId, employeeId];

  const contractsQuery = useQuery({
    queryKey,
    queryFn: () => fetchEmployeeContracts(wsId, employeeId),
    enabled: !!wsId && !!employeeId,
  });

  const createContract = useMutation({
    mutationFn: (data: CreateContractRequest) => createEmployeeContract(wsId, employeeId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey });
      void queryClient.invalidateQueries({ queryKey: ["workspace-contracts", wsId] });
    },
  });

  const updateContract = useMutation({
    mutationFn: ({ contractId, data }: { contractId: string; data: UpdateContractRequest }) =>
      updateEmployeeContract(wsId, employeeId, contractId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey });
      void queryClient.invalidateQueries({ queryKey: ["workspace-contracts", wsId] });
    },
  });

  const deleteContract = useMutation({
    mutationFn: (contractId: string) => deleteEmployeeContract(wsId, employeeId, contractId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey });
      void queryClient.invalidateQueries({ queryKey: ["workspace-contracts", wsId] });
    },
  });

  return {
    contracts: contractsQuery.data ?? [],
    isLoading: contractsQuery.isLoading,
    isError: contractsQuery.isError,
    createContract,
    updateContract,
    deleteContract,
  };
}
