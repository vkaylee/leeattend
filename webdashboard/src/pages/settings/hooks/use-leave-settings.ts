import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import type { Department } from "@/lib/api/types";

import { useAllEmployees } from "@/hooks/use-employees";
import { useDepartments } from "@/hooks/use-departments";
import { fetchLeavePolicyAssignments, deleteLeavePolicyAssignment } from "@/lib/api";

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
  
  const [activeTab, setActiveTab] = useState<TabType>("types");
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);
  const [editingAssignment, setEditingAssignment] = useState<LeaveAssignment | null>(null);

  const { data: assignments = [], isLoading: assignmentsLoading } = useQuery<LeaveAssignment[]>({
    queryKey: ["leave-assignments"],
    queryFn: () => {
      return fetchLeavePolicyAssignments();
    },
    enabled: activeTab === "assignments"
  });

  const { employees, isLoading: employeesLoading } = useAllEmployees({
    enabled: activeTab === "assignments"
  });

  const { data: departments = [], isLoading: departmentsLoading } = useDepartments({
    enabled: activeTab === "assignments",
  });

  const deleteAssignmentMutation = useMutation({
    mutationFn: (assignmentId: string) => {
      
      return deleteLeavePolicyAssignment(assignmentId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-assignments"] });
      void queryClient.invalidateQueries({ queryKey: ["leave-balances"] });
      void queryClient.invalidateQueries({ queryKey: ["workspace-leave-balances"] });
    },
  });

  return {
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


