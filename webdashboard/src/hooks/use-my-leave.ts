import { useQuery } from "@tanstack/react-query";

import {
  EmployeeLeaveBalance,
  LeaveRequest,
  fetchEssLeaveBalances,
  fetchEssLeaveRequests,
} from "@/lib/api";

import { essKeys } from "./use-ess-keys";

export interface UseMyLeaveReturn {
  balances: EmployeeLeaveBalance[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  refetchBalances: () => Promise<unknown>;
  refetchRequests: () => Promise<unknown>;
  requests: LeaveRequest[];
}

export function useMyLeave(): UseMyLeaveReturn {
  const balancesQuery = useQuery<EmployeeLeaveBalance[]>({
    queryKey: essKeys.leaveBalances(),
    queryFn: fetchEssLeaveBalances,
  });

  const requestsQuery = useQuery<LeaveRequest[]>({
    queryKey: essKeys.leaveRequests(),
    queryFn: fetchEssLeaveRequests,
  });

  return {
    balances: balancesQuery.data ?? [],
    requests: requestsQuery.data ?? [],
    isLoading: balancesQuery.isLoading || requestsQuery.isLoading,
    isError: balancesQuery.isError || requestsQuery.isError,
    error: balancesQuery.error ?? requestsQuery.error,
    refetchBalances: balancesQuery.refetch,
    refetchRequests: requestsQuery.refetch,
  };
}
