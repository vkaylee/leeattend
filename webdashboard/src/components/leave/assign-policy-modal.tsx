import { ShieldCheck } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { LeaveAssignment } from "@/pages/settings/hooks/use-leave-settings";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";

import { 
  AssigneeTypeSelect, 
  EmployeePicker, 
  DepartmentPicker, 
  PolicyPicker 
} from "./assign-policy-components/assign-policy-fields";
import { useAssignPolicy } from "./assign-policy-components/use-assign-policy";

interface AssignPolicyModalProps {
  editingAssignment?: LeaveAssignment | null;
  isOpen: boolean;
  onClose: () => void;
}

export function AssignPolicyModal({ isOpen, onClose,editingAssignment }: AssignPolicyModalProps) {
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
  } = useAssignPolicy(isOpen, onClose, editingAssignment);

  return (
    <Modal 
      icon={<ShieldCheck size={24} />} 
      isOpen={isOpen} 
      maxWidth="md" 
      onClose={onClose} 
      subtitle={editingAssignment ? t("leave_requests.policy.edit_assignment_subtitle", { defaultValue: "Modify assignment rules and effective dates." }) : t("leave_requests.policy.assign_subtitle")} 
      title={editingAssignment ? t("leave_requests.policy.edit_assignment_title", { defaultValue: "EDIT POLICY ASSIGNMENT" }) : t("leave_requests.policy.assign_title")}
    >
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
          <input className="w-full px-5 py-4 text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 border border-primary-100 dark:border-primary-800/50 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" id="assign-date" name="effective_from" onChange={(e) => { setEffectiveFrom(e.target.value); }} required type="date" value={effectiveFrom} />
        </div>

        <div className="pt-4 flex gap-4">
          <Button
            className="flex-1 h-auto py-4 px-8 border-primary-200 text-primary-400 dark:border-primary-800/50"
            onClick={onClose}
            size="md"
            type="button"
            variant="outline"
          >
            {t("common.cancel")}
          </Button>
          <Button
            className="flex-1 shadow-primary-200"
            data-testid="submit-assign-policy-btn"
            disabled={assignMutation.isPending}
            size="md"
            type="submit"
            variant="primary"
          >
            {assignMutation.isPending 
              ? t("common.processing").toUpperCase() 
              : (editingAssignment 
                  ? t("common.update", { defaultValue: "UPDATE" }).toUpperCase() 
                  : t("leave_requests.policy.assign_now_btn").toUpperCase()
                )
            }
          </Button>
        </div>
      </form>
    </Modal>
  );
}
