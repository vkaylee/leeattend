import { useQuery } from "@tanstack/react-query";
import { useWorkspace } from "./use-workspace";

export interface WorkspaceStats {
  total_employees: number;
  present_today: number;
  pending_leave_requests: number;
  estimated_monthly_payroll: string;
}

export function useDashboardStats() {
  const { workspace } = useWorkspace();

  return useQuery<WorkspaceStats>({
    queryKey: ["workspace-stats", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");
      
      const response = await fetch(`/api/dashboard/workspace-stats`, {
        headers: {
          "X-Workspace-Id": workspace.id,
          "Authorization": `Bearer ${localStorage.getItem("access_token")}`,
        },
      });

      if (!response.ok) {
        throw new Error("Failed to fetch dashboard stats");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}
