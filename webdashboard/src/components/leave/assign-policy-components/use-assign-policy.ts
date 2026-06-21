import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";
 
import type { LeaveAssignment } from "@/pages/settings/hooks/use-leave-settings";

import { useToast } from "@/components/ui/toast";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  fetchDepartments,
  fetchLeavePolicies,
  fetchLeaveTypes,
  assignLeavePolicy,
  deleteLeavePolicyAssignment,
} from "@/lib/api";

export function useAssignPolicy(
  wsId: string | null,
  isOpen: boolean,
  onClose: () => void,
  editingAssignment?: LeaveAssignment | null
) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const [assigneeType, setAssigneeType] = useState<string>("employee");
  const [assignEmpId, setAssignEmpId] = useState<string>("");
  const [assignDeptId, setAssignDeptId] = useState<string>("");
  const [assignPolicyId, setAssignPolicyId] = useState<string>("");
  const [effectiveFrom, setEffectiveFrom] = useState<string>(format(new Date(), "yyyy-MM-dd"));

  // Track props for resetting state during render to avoid useEffect cascading renders
  const [prevIsOpen, setPrevIsOpen] = useState(isOpen);
  const [prevEditingAssignment, setPrevEditingAssignment] = useState(editingAssignment);

  if (isOpen !== prevIsOpen || editingAssignment !== prevEditingAssignment) {
    setPrevIsOpen(isOpen);
    setPrevEditingAssignment(editingAssignment);
    if (isOpen) {
      if (editingAssignment) {
        setAssigneeType(editingAssignment.assignee_type);
        setAssignEmpId(editingAssignment.assignee_type === "employee" ? (editingAssignment.assignee_id ?? "") : "");
        setAssignDeptId(editingAssignment.assignee_type === "department" ? (editingAssignment.assignee_id ?? "") : "");
        setAssignPolicyId(editingAssignment.policy_id);
        setEffectiveFrom(format(new Date(editingAssignment.effective_from), "yyyy-MM-dd"));
      } else {
        setAssigneeType("employee");
        setAssignEmpId("");
        setAssignDeptId("");
        setAssignPolicyId("");
        setEffectiveFrom(format(new Date(), "yyyy-MM-dd"));
      }
    }
  }

  const { employees, isLoading: employeesLoading } = useAllEmployees(wsId, { enabled: isOpen });
  
  const { data: departments = [], isLoading: departmentsLoading } = useQuery({ 
    queryKey: ["departments", wsId], 
    queryFn: () => wsId ? fetchDepartments(wsId) : Promise.resolve([]), 
    enabled: isOpen && !!wsId 
  });
  
  const { data: policies = [], isLoading: policiesLoading } = useQuery({ 
    queryKey: ["leave-policies", wsId], 
    queryFn: () => wsId ? fetchLeavePolicies(wsId) : Promise.resolve([]), 
    enabled: isOpen && !!wsId 
  });
  
  const { data: leaveTypes = [] } = useQuery({ 
    queryKey: ["leave-types", wsId], 
    queryFn: () => wsId ? fetchLeaveTypes(wsId) : Promise.resolve([]), 
    enabled: isOpen && !!wsId 
  });

  const assignMutation = useMutation({
    mutationFn: async (data: { assignee_type: string; assignee_id: string | null; policy_id: string; effective_from: string }) => {
      if (!wsId) throw new Error("Workspace ID is required");
      if (editingAssignment) {
        // Sequential Delete-then-Post flow to update
        await deleteLeavePolicyAssignment(wsId, editingAssignment.id);
      }
      return assignLeavePolicy(wsId, data);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-balances"] });
      void queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      void queryClient.invalidateQueries({ queryKey: ["leave-assignments"] });
      toast.success(editingAssignment 
        ? t("leave_requests.policy.messages.update_assignment_success", { defaultValue: "Policy assignment updated successfully" }) 
        : t("leave_requests.policy.messages.assign_success")
      );
      onClose();
    },
  });

  const handleAssignPolicy = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!assignPolicyId) return;
    if (assigneeType === 'employee' && !assignEmpId) return;
    if (assigneeType === 'department' && !assignDeptId) return;

    const assignee_id_raw = assigneeType === 'workspace' ? null : (assigneeType === 'department' ? assignDeptId : assignEmpId);
    
    assignMutation.mutate({ 
      assignee_type: assigneeType, 
      assignee_id: (assignee_id_raw === "" || assignee_id_raw === null) ? null : assignee_id_raw, 
      policy_id: assignPolicyId, 
      effective_from: effectiveFrom 
    });
  };

  return {
    assigneeType,
    setAssigneeType,
    assignEmpId,
    setAssignEmpId,
    assignDeptId,
    setAssignDeptId,
    assignPolicyId,
    setAssignPolicyId,
    effectiveFrom,
    setEffectiveFrom,
    employees,
    employeesLoading,
    departments,
    departmentsLoading,
    policies,
    policiesLoading,
    leaveTypes,
    assignMutation,
    handleAssignPolicy
  };
}
