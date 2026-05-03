import { ShieldCheck } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import Modal from "@/components/ui/modal";

import { 
  AssigneeTypeSelect, 
  EmployeePicker, 
  DepartmentPicker, 
  PolicyPicker 
} from "./assign-policy-components/assign-policy-fields";
import { useAssignPolicy } from "./assign-policy-components/use-assign-policy";

interface AssignPolicyModalProps {
  isOpen: boolean;
  onClose: () => void;
  wsId: string | null;
}

export function AssignPolicyModal({ isOpen, onClose, wsId }: AssignPolicyModalProps) {
  const { t } = useTranslation();
  
  const {
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
  } = useAssignPolicy(wsId, isOpen, onClose);

  return (
    <Modal icon={<ShieldCheck size={24} />} isOpen={isOpen} maxWidth="md" onClose={onClose} subtitle={t("leave_requests.policy.assign_subtitle")} title={t("leave_requests.policy.assign_title")}>
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleAssignPolicy}>
        <AssigneeTypeSelect 
          assigneeType={assigneeType}
          setAssigneeType={setAssigneeType}
          t={t}
        />

        {assigneeType === 'employee' && (
          <EmployeePicker 
            assignEmpId={assignEmpId}
            employees={employees}
            employeesLoading={employeesLoading}
            setAssignEmpId={setAssignEmpId}
            t={t}
          />
        )}

        {assigneeType === 'department' && (
          <DepartmentPicker 
            assignDeptId={assignDeptId}
            departments={departments}
            departmentsLoading={departmentsLoading}
            setAssignDeptId={setAssignDeptId}
            t={t}
          />
        )}

        <PolicyPicker 
          assignPolicyId={assignPolicyId}
          leaveTypes={leaveTypes}
          policies={policies}
          policiesLoading={policiesLoading}
          setAssignPolicyId={setAssignPolicyId}
          t={t}
        />

        <div className="space-y-2">
          <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assign-date">{t("common.effective_from")}</label>
          <input className="w-full px-5 py-4 text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 border border-primary-100 dark:border-primary-800/50 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" defaultValue={format(new Date(), 'yyyy-MM-dd')} id="assign-date" name="effective_from" required type="date" />
        </div>

        <div className="pt-4 flex gap-4">
          <button className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-primary-200 dark:border-primary-800/50 rounded-2xl font-bold text-xs uppercase tracking-widest text-primary-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" onClick={onClose} type="button">{t("common.cancel")}</button>
          <button className="flex-1 bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-200 active:scale-95 disabled:opacity-50" data-testid="submit-assign-policy-btn" disabled={assignMutation.isPending} type="submit">{assignMutation.isPending ? t("common.processing").toUpperCase() : t("leave_requests.policy.assign_now_btn").toUpperCase()}</button>
        </div>
      </form>
    </Modal>
  );
}
