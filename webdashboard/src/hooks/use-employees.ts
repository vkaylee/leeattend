import { useQuery } from "@tanstack/react-query";

import { fetchEmployees } from "@/lib/api";
import { apiFetch } from "@/lib/api/helpers";
import { WorkingCondition } from "@/lib/api/types";

/**
 * Maximum number of employees fetched in a single "all employees" request.
 * Used across pages that need a full employee roster for dropdowns/search.
 */
export const MAX_EMPLOYEE_FETCH = 3_000;

export const employeeKeys = {
  all: ["employees"] as const,
  list: (wsId: string | null | undefined) => [...employeeKeys.all, wsId] as const,
};

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
  options?: { enabled?: boolean },
) {
  const enabledOverride = options?.enabled ?? true;

  const query = useQuery({
    queryKey: employeeKeys.list(wsId),
    queryFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return fetchEmployees(wsId, 1, MAX_EMPLOYEE_FETCH);
    },
    enabled: !!wsId && enabledOverride,
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
  return useQuery({
    queryKey: ["working-conditions", employeeId],
    queryFn: async () => {
      const response = await apiFetch(`/v1/employees/${employeeId}/working-conditions`);
      if (!response.ok) throw new Error("Failed to fetch");
      return response.json() as Promise<WorkingCondition[]>;
    },
    enabled: !!employeeId,
  });
}
