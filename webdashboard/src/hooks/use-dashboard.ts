import { useQuery } from "@tanstack/react-query";
import { useWorkspace } from "./use-workspace";
import { apiFetch } from "@/lib/api";

export interface WorkspaceStats {
  total_employees: number;
  present_today: number;
  pending_leave_requests: number;
  estimated_monthly_payroll: string;
  late_today: number;
  absent_today: number;
  on_leave_today: number;
  early_departure_today: number;
  avg_work_hours_today: number;
}

export function useDashboardStats() {
  const { workspace } = useWorkspace();

  return useQuery<WorkspaceStats>({
    queryKey: ["workspace-stats", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/dashboard/workspace-stats`);

      if (!response.ok) {
        throw new Error("Failed to fetch dashboard stats");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}
