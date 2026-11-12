import { ShieldCheck, Building2 } from "lucide-react";
import React from "react";

import { EmployeeSearch } from "@/components/common/employee-search";
import { Combobox } from "@/components/ui/combobox";
import { Department, LeavePolicy, LeaveType } from "@/lib/api/types";

interface FieldProps {
  t: (key: string) => string;
}

export function AssigneeTypeSelect({ assigneeType, setAssigneeType, t }: { assigneeType: string; setAssigneeType: (val: string) => void } & FieldProps) {
  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assignee-type">{t("leave_requests.policy.table.type")}</label>
      <Combobox className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" data-testid="assign-type-select" id="assignee-type" onChange={setAssigneeType} options={[{ value: "organization", label: t("leave_requests.policy.assignee_types.organization"), description: t("leave_requests.policy.assignee_types.organization_desc") }, { value: "department", label: t("leave_requests.policy.assignee_types.department"), description: t("leave_requests.policy.assignee_types.department_desc") }, { value: "employee", label: t("leave_requests.policy.assignee_types.employee"), description: t("leave_requests.policy.assignee_types.employee_desc") }]} value={assigneeType} />
    </div>
  );
}

export function EmployeePicker({ assignEmpId, setAssignEmpId, t }: { assignEmpId: string; setAssignEmpId: (val: string) => void } & FieldProps) {
  return (
    <div className="space-y-2 animate-in slide-in-from-top-2">
      <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assign-emp">{t("common.employee")}</label>
      <EmployeeSearch
        data-testid="assign-emp-select"
        id="assign-emp"
        onChange={(value) => { setAssignEmpId(typeof value === "string" ? value : value[0] ?? ""); }}
        placeholder={t("common.select_employee")}
        value={assignEmpId}
      />
    </div>
  );
}

export function DepartmentPicker({ assignDeptId, setAssignDeptId, departments, departmentsLoading, t }: { assignDeptId: string; setAssignDeptId: (val: string) => void; departments: Department[]; departmentsLoading: boolean } & FieldProps) {
  return (
    <div className="space-y-2 animate-in slide-in-from-top-2">
      <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assign-dept">{t("common.department")}</label>
      <Combobox className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" id="assign-dept" onChange={setAssignDeptId} options={departments.map(d => ({ value: d.id, label: d.name, icon: <Building2 size={14} /> }))} placeholder={departmentsLoading ? t("common.loading") : t("common.select_department")} value={assignDeptId} />
    </div>
  );
}

export function PolicyPicker({ assignPolicyId, setAssignPolicyId, policies, leaveTypes, policiesLoading, t }: { assignPolicyId: string; setAssignPolicyId: (val: string) => void; policies: LeavePolicy[]; leaveTypes: LeaveType[]; policiesLoading: boolean } & FieldProps) {
  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold text-primary-400 uppercase tracking-widest ml-1" htmlFor="assign-policy">{t("leave_requests.policy.title_single")}</label>
      <Combobox className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" data-testid="assign-policy-select" id="assign-policy" onChange={setAssignPolicyId} options={policies.map(p => ({ value: p.id, label: p.name, description: leaveTypes.find(type => type.id === p.leave_type_id)?.name ?? p.leave_type_id, icon: <ShieldCheck size={14} /> }))} placeholder={policiesLoading ? t("common.loading") : t("leave_requests.policy.select_placeholder")} value={assignPolicyId} />
    </div>
  );
}
