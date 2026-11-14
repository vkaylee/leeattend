import { useQuery } from "@tanstack/react-query";

import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { fetchEmployees, fetchWorkingConditions, fetchWorkingConditionsForAllEmployees, type WorkingConditionWithEmployee } from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";

/**
 * Maximum number of employees fetched in a single "all employees" request.
 * Used across pages that need a full employee roster for dropdowns/search.
 */
export const MAX_EMPLOYEE_FETCH = 3000;

export { employeeKeys } from "@/lib/query-keys";

/**
 * Shared hook to fetch the full employee list for the organization.
 *
 * Replaces duplicated full-roster fetch options across pages.
 * found across multiple pages. Centralizes query key, enabled guard,
 * and the fetch limit into one reusable hook.
 *
 * @param options - Optional overrides (e.g., enabled for lazy-loaded modals).
 */
export function useAllEmployees(
  options?: { enabled?: boolean; permission?: string },
) {
  const enabledOverride = options?.enabled ?? true;
  const permission = options?.permission;

  const query = useOrganizationQuery({
    queryKey: employeeKeys.list({ permission }),
    queryFn: () => fetchEmployees(1, MAX_EMPLOYEE_FETCH, permission),
    enabled: enabledOverride,
  });

  return {
    employees: (query.data?.data ?? []),
    employeesData: query.data,
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    refetch: query.refetch,
  };
}

export const workingConditionKeys = {
  all: ["working-conditions"] as const,
  list: (employeeId: string) => [...workingConditionKeys.all, employeeId] as const,
  organization: () => [...workingConditionKeys.all, "organization"] as const,
};

export function useOrganizationWorkingConditions() {
  return useQuery<WorkingConditionWithEmployee[]>({
    queryKey: workingConditionKeys.organization(),
    queryFn: () => fetchWorkingConditionsForAllEmployees(),
  });
}

export function useWorkingConditions(employeeId?: string) {
  return useQuery({
    queryKey: employeeId ? workingConditionKeys.list(employeeId) : workingConditionKeys.organization(),
    queryFn: () => {
      if (employeeId) {
        return fetchWorkingConditions(employeeId);
      }
      return fetchWorkingConditionsForAllEmployees();
    },
  });
}
