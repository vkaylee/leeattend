import { useQuery } from "@tanstack/react-query";

import { 
  fetchAssignmentRules, 
  fetchWorkingPatterns,
  fetchDepartments,
  fetchEmployers,
  fetchLocations,
  getCurrentWorkspaceId
} from "@/lib/api";

export function useAssignmentRulesData() {
  const wsId = getCurrentWorkspaceId() ?? "";

  const { data: rules, isLoading: rulesLoading } = useQuery({ 
    queryKey: ["assignment-rules", wsId], 
    queryFn: () => fetchAssignmentRules(wsId), 
    enabled: !!wsId 
  });

  const { data: patterns } = useQuery({ 
    queryKey: ["working-patterns", wsId], 
    queryFn: () => fetchWorkingPatterns(wsId), 
    enabled: !!wsId 
  });

  const { data: departments } = useQuery({ 
    queryKey: ["departments", wsId], 
    queryFn: () => fetchDepartments(wsId), 
    enabled: !!wsId 
  });

  const { data: employers } = useQuery({ 
    queryKey: ["employers", wsId], 
    queryFn: () => fetchEmployers(wsId), 
    enabled: !!wsId 
  });

  const { data: locations } = useQuery({ 
    queryKey: ["locations", wsId], 
    queryFn: () => fetchLocations(wsId), 
    enabled: !!wsId 
  });

  return {
    wsId,
    rules,
    rulesLoading,
    patterns,
    departments,
    employers,
    locations
  };
}
