import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";
 
import { useToast } from "@/components/ui/toast";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  fetchDepartments,
  fetchLeavePolicies,
  fetchLeaveTypes,
  assignLeavePolicy,
} from "@/lib/api";

export function useAssignPolicy(wsId: string | null, isOpen: boolean, onClose: () => void) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const [assigneeType, setAssigneeType] = useState<string>("employee");
  const [assignEmpId, setAssignEmpId] = useState<string>("");
  const [assignDeptId, setAssignDeptId] = useState<string>("");
  const [assignPolicyId, setAssignPolicyId] = useState<string>("");

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
    mutationFn: (data: { assignee_type: string; assignee_id: string | null; policy_id: string; effective_from: string }) => {
      if (!wsId) throw new Error("Workspace ID is required");
      return assignLeavePolicy(wsId, data);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-balances"] });
      void queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      void queryClient.invalidateQueries({ queryKey: ["leave-assignments"] });
      toast.success(t("leave_requests.policy.messages.assign_success"));
      onClose();
    },
  });

  const handleAssignPolicy = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!assignPolicyId) return;
    if (assigneeType === 'employee' && !assignEmpId) return;
    if (assigneeType === 'department' && !assignDeptId) return;

    const formData = new FormData(e.currentTarget);
    const assignee_id_raw = assigneeType === 'workspace' ? null : (assigneeType === 'department' ? assignDeptId : assignEmpId);
    const effective_from = formData.get("effective_from") as string;
    
    assignMutation.mutate({ 
      assignee_type: assigneeType, 
      assignee_id: (assignee_id_raw === "" || assignee_id_raw === null) ? null : assignee_id_raw, 
      policy_id: assignPolicyId, 
      effective_from 
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
