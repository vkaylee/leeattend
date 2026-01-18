import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { UserCircle, FileText, ShieldCheck, CalendarDays } from "lucide-react";
import {
  fetchEmployees,
  fetchDepartments,
  fetchLeavePolicies,
  assignLeavePolicy,
  getCurrentWorkspaceId,
} from "../../lib/api";
import { Combobox } from "../ui/Combobox";
import Modal from "../ui/Modal";
import { useToast } from "../Toast";
import { format } from "date-fns";

interface AssignPolicyModalProps {
  isOpen: boolean;
  onClose: () => void;
  wsId: string | null;
}

export function AssignPolicyModal({ isOpen, onClose, wsId }: AssignPolicyModalProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const [assigneeType, setAssigneeType] = useState<string>("employee");
  const [assignEmpId, setAssignEmpId] = useState<string>("");
  const [assignDeptId, setAssignDeptId] = useState<string>("");
  const [assignPolicyId, setAssignPolicyId] = useState<string>("");

  // Isolated queries for the modal to ensure data is fresh and available
  const { data: employees = [], isLoading: employeesLoading } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: isOpen && !!wsId,
  });

  const { data: departments = [], isLoading: departmentsLoading } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId!),
    enabled: isOpen && !!wsId,
  });

  const { data: policies = [], isLoading: policiesLoading } = useQuery({
    queryKey: ["leave-policies", wsId],
    queryFn: () => fetchLeavePolicies(wsId!),
    enabled: isOpen && !!wsId,
  });

  const assignMutation = useMutation({
    mutationFn: (data: { 
      assignee_type: string; 
      assignee_id: string | null; 
      policy_id: string; 
      effective_from: string 
    }) => assignLeavePolicy(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-balances", wsId] });
      queryClient.invalidateQueries({ queryKey: ["leave-policies", wsId] });
      toast.success(t("leave_requests.policy.messages.assign_success"));
      onClose();
    },
    onError: (error: any) => {
      toast.error(error.message || "Failed to assign policy");
    }
  });

  const handleAssignPolicy = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    console.log('[AssignPolicyModal] Submitting with:', { assigneeType, assignEmpId, assignDeptId, assignPolicyId });
    
    if (!assignPolicyId) {
      console.warn('[AssignPolicyModal] Missing policy ID');
      toast.error(t("leave_requests.policy.select_placeholder"));
      return;
    }

    if (assigneeType === 'employee' && !assignEmpId) {
      console.warn('[AssignPolicyModal] Missing employee ID');
      toast.error(t("common.select_employee"));
      return;
    }

    if (assigneeType === 'department' && !assignDeptId) {
      console.warn('[AssignPolicyModal] Missing department ID');
      toast.error(t("common.select_department"));
      return;
    }

    const formData = new FormData(e.currentTarget);
    const assignee_id_raw = assigneeType === 'workspace' ? null : (assigneeType === 'department' ? assignDeptId : assignEmpId);
    const effective_from = formData.get("effective_from") as string;

    console.log('[AssignPolicyModal] Mutating with:', { 
      assignee_type: assigneeType, 
      assignee_id: (assignee_id_raw === "" || assignee_id_raw === null) ? null : assignee_id_raw,
      policy_id: assignPolicyId,
      effective_from
    });
    
    assignMutation.mutate({
      assignee_type: assigneeType,
      assignee_id: (assignee_id_raw === "" || assignee_id_raw === null) ? null : assignee_id_raw,
      policy_id: assignPolicyId,
      effective_from,
    });
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("leave_requests.policy.assign_title")}
      subtitle={t("leave_requests.policy.assign_subtitle")}
      icon={<UserCircle className="w-6 h-6" />}
    >
      <form onSubmit={handleAssignPolicy} className="p-2 space-y-6">
        <div className="space-y-2">
          <label 
            htmlFor="assignee-type"
            className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1"
          >
            {t("leave_requests.policy.table.type")}
          </label>
          <Combobox
            id="assignee-type"
            options={[
              { value: "workspace", label: t("leave_requests.policy.assignee_types.workspace"), description: t("leave_requests.policy.assignee_types.workspace_desc") },
              { value: "department", label: t("leave_requests.policy.assignee_types.department"), description: t("leave_requests.policy.assignee_types.department_desc") },
              { value: "employee", label: t("leave_requests.policy.assignee_types.employee"), description: t("leave_requests.policy.assignee_types.employee_desc") },
            ]}
            value={assigneeType}
            onChange={setAssigneeType}
            className="bg-slate-50 border-slate-200"
          />
        </div>

        {assigneeType === 'employee' && (
          <div className="space-y-2">
            <label 
              htmlFor="assign-emp"
              className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1"
            >
              {t("common.employee")}
            </label>
            <Combobox
              id="assign-emp"
              options={employees.map(e => ({
                value: e.id,
                label: `${e.first_name} ${e.last_name}`,
                description: e.employee_id_number,
                icon: <UserCircle className="w-4 h-4" />
              }))}
              value={assignEmpId}
              onChange={setAssignEmpId}
              placeholder={employeesLoading ? t("common.loading") : t("common.select_employee")}
              className="bg-slate-50 border-slate-200"
            />
          </div>
        )}

        {assigneeType === 'department' && (
          <div className="space-y-2">
            <label 
              htmlFor="assign-dept"
              className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1"
            >
              {t("common.department")}
            </label>
            <Combobox
              id="assign-dept"
              options={departments.map(d => ({
                value: d.id,
                label: d.name,
                icon: <FileText className="w-4 h-4" />
              }))}
              value={assignDeptId}
              onChange={setAssignDeptId}
              placeholder={departmentsLoading ? t("common.loading") : t("common.select_department")}
              className="bg-slate-50 border-slate-200"
            />
          </div>
        )}

        <div className="space-y-2">
          <label 
            htmlFor="assign-policy"
            className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1"
          >
            {t("leave_requests.policy.title_single")}
          </label>
          <Combobox
            id="assign-policy"
            options={policies.map(p => ({
              value: p.id,
              label: p.name,
              description: p.leave_type === 'vacation' ? t("leave_requests.types.vacation") : t("leave_requests.types.sick"),
              icon: <ShieldCheck className="w-4 h-4" />
            }))}
            value={assignPolicyId}
            onChange={setAssignPolicyId}
            placeholder={policiesLoading ? t("common.loading") : t("leave_requests.policy.select_placeholder")}
            className="bg-slate-50 border-slate-200"
          />
        </div>
        <div className="space-y-1">
          <label htmlFor="assign-date" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.effective_from")}</label>
          <input id="assign-date" type="date" name="effective_from" required defaultValue={format(new Date(), 'yyyy-MM-dd')} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none font-bold" />
        </div>
        <div className="pt-4 flex gap-3">
          <button type="button" onClick={onClose} className="flex-1 px-6 py-3 border border-slate-200 rounded-xl font-bold text-slate-500 hover:bg-slate-50 transition-colors">{t("common.cancel")}</button>
          <button type="submit" data-testid="submit-assign-policy-btn" disabled={assignMutation.isPending} className="flex-1 px-6 py-3 bg-slate-900 text-white rounded-xl font-bold hover:bg-slate-800 transition-colors shadow-lg shadow-slate-200 disabled:opacity-50">
            {assignMutation.isPending ? t("common.processing") : t("leave_requests.policy.assign_now_btn")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
