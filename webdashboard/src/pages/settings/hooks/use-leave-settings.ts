import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { useAllEmployees } from "@/hooks/use-employees";
import { fetchLeavePolicyAssignments, fetchDepartments } from "@/lib/api";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";
import type { Employee, Department } from "@/lib/api/types";

export type TabType = "types" | "policies" | "assignments";

export interface LeaveAssignment {
  assignee_type: string;
  assignee_id?: string | null;
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

  const { employees = [], isLoading: employeesLoading } = useAllEmployees(wsId, {
    enabled: !!wsId && activeTab === "assignments"
  });

  const { data: departments = [], isLoading: departmentsLoading } = useQuery<Department[]>({
    queryKey: ["departments", wsId],
    queryFn: () => wsId ? fetchDepartments(wsId) : Promise.resolve([]),
    enabled: !!wsId && activeTab === "assignments"
  });

  return {
    wsId,
    activeTab,
    setActiveTab,
    isAssignModalOpen,
    setIsAssignModalOpen,
    assignments,
    assignmentsLoading: assignmentsLoading || employeesLoading || departmentsLoading,
    employees,
    departments,
  };
}

