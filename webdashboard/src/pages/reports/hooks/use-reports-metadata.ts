import { useQuery } from "@tanstack/react-query";

import { fetchDepartments, fetchWorkspace } from "@/lib/api";

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

  const { data: departments } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchDepartments(wsId);
    },
    enabled: !!wsId,
  });

  return { workspace, departments };
}
