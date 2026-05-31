import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { fetchLeavePolicyAssignments } from "@/lib/api";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

export type TabType = "types" | "policies" | "assignments" | "workflows";

export interface LeaveAssignment {
  assignee_type: string;
  created_at: string;
  effective_from: string;
  effective_to: string | null;
  id: string;
  leave_type_name: string;
  policy_name: string;
}

export function useLeaveSettings() {
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("types");
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);

  const { data: assignments = [], isLoading: assignmentsLoading } = useQuery<LeaveAssignment[]>({
    queryKey: ["leave-assignments", wsId],
    queryFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return fetchLeavePolicyAssignments(wsId);
    },
    enabled: !!wsId && activeTab === "assignments"
  });

  return {
    wsId,
    activeTab,
    setActiveTab,
    isAssignModalOpen,
    setIsAssignModalOpen,
    assignments,
    assignmentsLoading,
  };
}
