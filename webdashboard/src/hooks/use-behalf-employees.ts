/**
 * useBehalfEmployees — Reusable hook for fetching the list of employees
 * that the current user is authorized to act on behalf of.
 *
 * This hook calls the backend `/behalf-assignments/my-targets` endpoint
 * which resolves both employee-level and department-level assignments into
 * a concrete Employee[] list.
 *
 * Usage:
 *   const { employees, hasAssignments, isLoading } = useBehalfEmployees("leave");
 */

import { useQuery } from "@tanstack/react-query";

import type { BehalfDomain } from "@/lib/api/behalf-targets";
import type { Employee } from "@/lib/api/types";

import { fetchMyBehalfTargets } from "@/lib/api/behalf-targets";

export interface UseBehalfEmployeesReturn {
  /** List of employees the current user can act on behalf of */
  employees: Employee[];
  /** Whether the user has any behalf assignments configured */
  hasAssignments: boolean;
  /** Whether the data is still loading */
  /** Whether there was an error fetching */
  isError: boolean;
  /** Whether the data is still loading */
  isLoading: boolean;
}

export const behalfEmployeesKeys = {
  all: ["behalf-employees"] as const,
  targets: (domain: BehalfDomain) =>
    [...behalfEmployeesKeys.all, "targets", domain] as const,
};

export function useBehalfEmployees(
  domain: BehalfDomain,
): UseBehalfEmployeesReturn {
  const query = useQuery<Employee[]>({
    queryKey: behalfEmployeesKeys.targets(domain),
    queryFn: async () => fetchMyBehalfTargets(domain),
    staleTime: 5 * 60 * 1000, // Cache for 5 minutes
  });

  const employees = query.data ?? [];

  return {
    employees,
    hasAssignments: query.isSuccess && employees.length > 0,
    isLoading: query.isLoading,
    isError: query.isError,
  };
}
