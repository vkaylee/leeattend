import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { CreateContractRequest, UpdateContractRequest } from "@/lib/api";

import {
  fetchEmployeeContracts,
  createEmployeeContract,
  updateEmployeeContract,
  deleteEmployeeContract,
} from "@/lib/api";

export function useEmployeeContracts(employeeId: string) {
  const queryClient = useQueryClient();
  const queryKey = ["employee-contracts", employeeId];

  const contractsQuery = useQuery({
    queryKey,
    queryFn: () => fetchEmployeeContracts(employeeId),
    enabled: !!employeeId,
  });

  const createContract = useMutation({
    mutationFn: (data: CreateContractRequest) => createEmployeeContract(employeeId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey });
      void queryClient.invalidateQueries({ queryKey: ["organization-contracts"] });
    },
  });

  const updateContract = useMutation({
    mutationFn: ({ contractId, data }: { contractId: string; data: UpdateContractRequest }) =>
      updateEmployeeContract(employeeId, contractId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey });
      void queryClient.invalidateQueries({ queryKey: ["organization-contracts"] });
    },
  });

  const deleteContract = useMutation({
    mutationFn: (contractId: string) => deleteEmployeeContract(employeeId, contractId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey });
      void queryClient.invalidateQueries({ queryKey: ["organization-contracts"] });
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
