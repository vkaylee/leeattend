import { useQuery } from "@tanstack/react-query";

import { useDepartments } from "@/hooks/use-departments";
import { useMyPermissions } from "@/hooks/use-permissions";
import {
  fetchAssignmentRules,
  fetchWorkingPatterns,
  fetchEmployers,
  fetchLocations,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";


export function useAssignmentRulesData() {
  const { hasPermission } = useMyPermissions();

  const { data: rules, isLoading: rulesLoading } = useQuery({
    queryKey: createQueryKey("assignment-rules"),
    queryFn: () => fetchAssignmentRules(),
    enabled: hasPermission("assignment_rules:manage"),
  });

  const { data: patterns } = useQuery({
    queryKey: createQueryKey("working-patterns"),
    queryFn: () => fetchWorkingPatterns(),
  });

  const { data: departments } = useDepartments();

  const { data: employers } = useQuery({
    queryKey: createQueryKey("employers"),
    queryFn: () => fetchEmployers(),
  });

  const { data: locations } = useQuery({
    queryKey: createQueryKey("locations"),
    queryFn: fetchLocations,
    retry: false,
    meta: { suppressErrorToast: [403] },
  });

  return {
    rules,
    rulesLoading,
    patterns,
    departments,
    employers,
    locations
  };
}
