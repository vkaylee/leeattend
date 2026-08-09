import { useQuery } from "@tanstack/react-query";

import { useDepartments } from "@/hooks/use-departments";

import { fetchWorkspace, fetchEmployers } from "@/lib/api";

interface UseReportsMetadataProps {
  wsId: string | null;
}

export function useReportsMetadata({ wsId }: UseReportsMetadataProps) {
  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchWorkspace(wsId);
    },
    enabled: !!wsId,
  });

  const { data: departments } = useDepartments(wsId);

  const { data: employers } = useQuery({
    queryKey: ["employers", wsId],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchEmployers(wsId);
    },
    enabled: !!wsId,
  });

  return { workspace, departments, employers };
}
