import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { UserCircle, FileText, ShieldCheck, CalendarDays, Building2 } from "lucide-react";
import {
  fetchEmployees,
  fetchDepartments,
  fetchLeavePolicies,
  fetchLeaveTypes,
  assignLeavePolicy,
  getCurrentWorkspaceId,
} from "@/lib/api";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/ui/Toast";
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

  const { data: employeesData, isLoading: employeesLoading } = useQuery({ queryKey: ["employees", wsId], queryFn: () => fetchEmployees(wsId!, 1, 3000), enabled: isOpen && !!wsId });
  const employees = employeesData?.data || [];
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
    <Modal isOpen={isOpen} onClose={onClose} title={t("leave_requests.policy.assign_title")} subtitle={t("leave_requests.policy.assign_subtitle")} icon={<ShieldCheck size={24} />} maxWidth="md">
      <form onSubmit={handleAssignPolicy} className="space-y-8 pt-4 animate-in fade-in duration-500">
        <div className="space-y-2">
          <label htmlFor="assignee-type" className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.type")}</label>
          <Combobox id="assignee-type" data-testid="assign-type-select" options={[{ value: "workspace", label: t("leave_requests.policy.assignee_types.workspace"), description: t("leave_requests.policy.assignee_types.workspace_desc") }, { value: "department", label: t("leave_requests.policy.assignee_types.department"), description: t("leave_requests.policy.assignee_types.department_desc") }, { value: "employee", label: t("leave_requests.policy.assignee_types.employee"), description: t("leave_requests.policy.assignee_types.employee_desc") }]} value={assigneeType} onChange={setAssigneeType} className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" />
        </div>

        {assigneeType === 'employee' && (
          <div className="space-y-2 animate-in slide-in-from-top-2">
            <label htmlFor="assign-emp" className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1">{t("common.employee")}</label>
            <Combobox id="assign-emp" data-testid="assign-emp-select" options={employees.map(e => ({ value: e.id, label: e.full_name, description: e.employee_id_number, icon: <UserCircle size={14} /> }))} value={assignEmpId} onChange={setAssignEmpId} placeholder={employeesLoading ? t("common.loading") : t("common.select_employee")} className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" />
          </div>
        )}

        {assigneeType === 'department' && (
          <div className="space-y-2 animate-in slide-in-from-top-2">
            <label htmlFor="assign-dept" className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1">{t("common.department")}</label>
            <Combobox id="assign-dept" options={departments.map(d => ({ value: d.id, label: d.name, icon: <Building2 size={14} /> }))} value={assignDeptId} onChange={setAssignDeptId} placeholder={departmentsLoading ? t("common.loading") : t("common.select_department")} className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" />
          </div>
        )}

        <div className="space-y-2">
          <label htmlFor="assign-policy" className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.title_single")}</label>
          <Combobox id="assign-policy" data-testid="assign-policy-select" options={policies.map(p => ({ value: p.id, label: p.name, description: leaveTypes.find(t => t.id === p.leave_type_id)?.name || p.leave_type_id, icon: <ShieldCheck size={14} /> }))} value={assignPolicyId} onChange={setAssignPolicyId} placeholder={policiesLoading ? t("common.loading") : t("leave_requests.policy.select_placeholder")} className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" />
        </div>

        <div className="space-y-2">
          <label htmlFor="assign-date" className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1">{t("common.effective_from")}</label>
          <input id="assign-date" type="date" name="effective_from" required defaultValue={format(new Date(), 'yyyy-MM-dd')} className="w-full px-5 py-4 text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 border border-primary-100 dark:border-primary-800/50 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight" />
        </div>

        <div className="pt-4 flex gap-4">
          <button type="button" onClick={onClose} className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-primary-200 dark:border-primary-800/50 rounded-2xl font-bold text-xs uppercase tracking-widest text-primary-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all">{t("common.cancel")}</button>
          <button data-testid="submit-assign-policy-btn" type="submit" disabled={assignMutation.isPending} className="flex-1 bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-indigo-200 active:scale-95 disabled:opacity-50">{assignMutation.isPending ? t("common.processing").toUpperCase() : t("leave_requests.policy.assign_now_btn").toUpperCase()}</button>
        </div>
      </form>
    </Modal>
  );
}
