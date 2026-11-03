import { useQuery } from "@tanstack/react-query";

import { fetchEmployees, fetchWorkingConditions, getStandaloneOrganizationId } from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";
import { useOrganizationQuery } from "@/hooks/common/use-workspace-query";

/**
 * Maximum number of employees fetched in a single "all employees" request.
 * Used across pages that need a full employee roster for dropdowns/search.
 */
export const MAX_EMPLOYEE_FETCH = 3000;

export { employeeKeys } from "@/lib/query-keys";

/**
 * Shared hook to fetch the full employee list for a workspace.
 *
 * Replaces the duplicated `fetchEmployees(wsId!, 1, 3000)` pattern
 * found across multiple pages. Centralizes query key, enabled guard,
 * and the fetch limit into one reusable hook.
 *
 * @param wsId - Active workspace ID. Query is disabled when falsy.
 * @param options - Optional overrides (e.g., enabled for lazy-loaded modals).
 */
export function useAllEmployees(
  wsId: string | null | undefined,
  options?: { enabled?: boolean; permission?: string },
) {
  const enabledOverride = options?.enabled ?? true;
  const permission = options?.permission;

  const query = useOrganizationQuery({
    workspaceId: wsId,
    queryKey: employeeKeys.list(wsId, { permission }),
    queryFn: (workspaceId) => fetchEmployees(workspaceId, 1, MAX_EMPLOYEE_FETCH, permission),
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

export function useWorkingConditions(employeeId: string) {
  const wsId = getStandaloneOrganizationId();
  return useQuery({
    queryKey: ["working-conditions", employeeId],
    queryFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return fetchWorkingConditions(wsId, employeeId);
    },
    enabled: !!employeeId && !!wsId,
  });
}
