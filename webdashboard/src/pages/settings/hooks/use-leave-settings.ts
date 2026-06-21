import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import type { Department } from "@/lib/api/types";

import { useAllEmployees } from "@/hooks/use-employees";
import { fetchLeavePolicyAssignments, fetchDepartments, deleteLeavePolicyAssignment } from "@/lib/api";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

export type TabType = "types" | "policies" | "assignments";

export interface LeaveAssignment {
  assignee_id?: string | null;
  assignee_type: string;
  created_at: string;
  effective_from: string;
  effective_to: string | null;
  id: string;
  leave_type_name: string;
  policy_id: string;
  policy_name: string;
}

export function useLeaveSettings() {
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("types");
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);
  const [editingAssignment, setEditingAssignment] = useState<LeaveAssignment | null>(null);

  const { data: assignments = [], isLoading: assignmentsLoading } = useQuery<LeaveAssignment[]>({
    queryKey: ["leave-assignments", wsId],
    queryFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return fetchLeavePolicyAssignments(wsId);
    },
    enabled: !!wsId && activeTab === "assignments"
  });

  const { employees, isLoading: employeesLoading } = useAllEmployees(wsId, {
    enabled: !!wsId && activeTab === "assignments"
  });

  const { data: departments = [], isLoading: departmentsLoading } = useQuery<Department[]>({
    queryKey: ["departments", wsId],
    queryFn: () => wsId ? fetchDepartments(wsId) : Promise.resolve([]),
    enabled: !!wsId && activeTab === "assignments"
  });

  const deleteAssignmentMutation = useMutation({
    mutationFn: (assignmentId: string) => {
      if (!wsId) throw new Error("Workspace ID is required");
      return deleteLeavePolicyAssignment(wsId, assignmentId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-assignments"] });
      void queryClient.invalidateQueries({ queryKey: ["leave-balances"] });
      void queryClient.invalidateQueries({ queryKey: ["workspace-leave-balances"] });
    },
  });

  return {
    wsId,
    activeTab,
    setActiveTab,
    isAssignModalOpen,
    setIsAssignModalOpen,
    editingAssignment,
    setEditingAssignment,
    assignments,
    assignmentsLoading: assignmentsLoading || employeesLoading || departmentsLoading,
    employees,
    departments,
    deleteAssignment: deleteAssignmentMutation.mutateAsync,
    isDeleting: deleteAssignmentMutation.isPending,
  };
}


