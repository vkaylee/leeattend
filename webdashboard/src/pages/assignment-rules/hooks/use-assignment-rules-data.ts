import { useQuery } from "@tanstack/react-query";

import {
  fetchAssignmentRules,
  fetchWorkingPatterns,
  fetchEmployers,
  fetchLocations,
} from "@/lib/api";
import { useDepartments } from "@/hooks/use-departments";

export function useAssignmentRulesData() {
  

  const { data: rules, isLoading: rulesLoading } = useQuery({ 
    queryKey: ["assignment-rules"], 
    queryFn: () => fetchAssignmentRules(), 
  });

  const { data: patterns } = useQuery({ 
    queryKey: ["working-patterns"], 
    queryFn: () => fetchWorkingPatterns(), 
  });

  const { data: departments } = useDepartments();

  const { data: employers } = useQuery({ 
    queryKey: ["employers"], 
    queryFn: () => fetchEmployers(), 
  });

  const { data: locations } = useQuery({ 
    queryKey: ["locations"], 
    queryFn: () => fetchLocations(), 
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
