import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { UserCircle, ShieldCheck, Building2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/ui/Toast";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  fetchDepartments,
  fetchLeavePolicies,
  fetchLeaveTypes,
  assignLeavePolicy,
} from "@/lib/api";


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

  const { employees, isLoading: employeesLoading } = useAllEmployees(wsId, { enabled: isOpen });
  const { data: departments = [], isLoading: departmentsLoading } = useQuery({ queryKey: ["departments", wsId], queryFn: () => fetchDepartments(wsId!), enabled: isOpen && !!wsId });
    const { data: policies = [], isLoading: policiesLoading } = useQuery({ queryKey: ["leave-policies", wsId], queryFn: () => fetchLeavePolicies(wsId!), enabled: isOpen && !!wsId });
  const { data: leaveTypes = [] } = useQuery({ queryKey: ["leave-types", wsId], queryFn: () => fetchLeaveTypes(wsId!), enabled: isOpen && !!wsId });


  const assignMutation = useMutation({
    mutationFn: (data: { assignee_type: string; assignee_id: string | null; policy_id: string; effective_from: string }) => assignLeavePolicy(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-balances"] });
      queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      queryClient.invalidateQueries({ queryKey: ["leave-assignments"] });
      toast.success(t("leave_requests.policy.messages.assign_success"));
      onClose();
    },
  });

  const handleAssignPolicy = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!assignPolicyId) return;
    if (assigneeType === 'employee' && !assignEmpId) return;
    if (assigneeType === 'department' && !assignDeptId) return;

    const formData = new FormData(e.currentTarget);
    const assignee_id_raw = assigneeType === 'workspace' ? null : (assigneeType === 'department' ? assignDeptId : assignEmpId);
    const effective_from = formData.get("effective_from") as string;
    
    assignMutation.mutate({ assignee_type: assigneeType, assignee_id: (assignee_id_raw === "" || assignee_id_raw === null) ? null : assignee_id_raw, policy_id: assignPolicyId, effective_from });
  };

  return (
    <Modal icon={<ShieldCheck size={24} />} isOpen={isOpen} maxWidth="md" onClose={onClose} subtitle={t("leave_requests.policy.assign_subtitle")} title={t("leave_requests.policy.assign_title")}>
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleAssignPolicy}>
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assignee-type">{t("leave_requests.policy.table.type")}</label>
          <Combobox className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" data-testid="assign-type-select" id="assignee-type" onChange={setAssigneeType} options={[{ value: "workspace", label: t("leave_requests.policy.assignee_types.workspace"), description: t("leave_requests.policy.assignee_types.workspace_desc") }, { value: "department", label: t("leave_requests.policy.assignee_types.department"), description: t("leave_requests.policy.assignee_types.department_desc") }, { value: "employee", label: t("leave_requests.policy.assignee_types.employee"), description: t("leave_requests.policy.assignee_types.employee_desc") }]} value={assigneeType} />
        </div>

        {assigneeType === 'employee' && (
          <div className="space-y-2 animate-in slide-in-from-top-2">
            <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assign-emp">{t("common.employee")}</label>
            <Combobox className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" data-testid="assign-emp-select" id="assign-emp" onChange={setAssignEmpId} options={employees.map(e => ({ value: e.id, label: e.full_name, description: e.employee_id_number, icon: <UserCircle size={14} /> }))} placeholder={employeesLoading ? t("common.loading") : t("common.select_employee")} value={assignEmpId} />
          </div>
        )}

        {assigneeType === 'department' && (
          <div className="space-y-2 animate-in slide-in-from-top-2">
            <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assign-dept">{t("common.department")}</label>
            <Combobox className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" id="assign-dept" onChange={setAssignDeptId} options={departments.map(d => ({ value: d.id, label: d.name, icon: <Building2 size={14} /> }))} placeholder={departmentsLoading ? t("common.loading") : t("common.select_department")} value={assignDeptId} />
          </div>
        )}

        <div className="space-y-2">
          <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assign-policy">{t("leave_requests.policy.title_single")}</label>
          <Combobox className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" data-testid="assign-policy-select" id="assign-policy" onChange={setAssignPolicyId} options={policies.map(p => ({ value: p.id, label: p.name, description: leaveTypes.find(t => t.id === p.leave_type_id)?.name || p.leave_type_id, icon: <ShieldCheck size={14} /> }))} placeholder={policiesLoading ? t("common.loading") : t("leave_requests.policy.select_placeholder")} value={assignPolicyId} />
        </div>

        <div className="space-y-2">
          <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assign-date">{t("common.effective_from")}</label>
          <input className="w-full px-5 py-4 text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 border border-primary-100 dark:border-primary-800/50 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" defaultValue={format(new Date(), 'yyyy-MM-dd')} id="assign-date" name="effective_from" required type="date" />
        </div>

        <div className="pt-4 flex gap-4">
          <button className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-primary-200 dark:border-primary-800/50 rounded-2xl font-bold text-xs uppercase tracking-widest text-primary-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" onClick={onClose} type="button">{t("common.cancel")}</button>
          <button className="flex-1 bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-indigo-200 active:scale-95 disabled:opacity-50" data-testid="submit-assign-policy-btn" disabled={assignMutation.isPending} type="submit">{assignMutation.isPending ? t("common.processing").toUpperCase() : t("leave_requests.policy.assign_now_btn").toUpperCase()}</button>
        </div>
      </form>
    </Modal>
  );
}
