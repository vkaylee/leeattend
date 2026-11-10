import { Globe } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Department } from "@/lib/api/types";

import { EmployeeSearch } from "@/components/common/employee-search";
import { Typography } from "@/components/ui/typography";

import type { ScopeType } from "./scope-selector";

interface AssigneePickerProps {
  assigneeId: string;
  assigneeType: ScopeType;
  departments?: Department[];
  setAssigneeId: (id: string) => void;
}

export function AssigneePicker({
  assigneeId,
  assigneeType,
  departments,
  setAssigneeId,}: AssigneePickerProps) {
  const { t } = useTranslation();

  if (assigneeType === "employee") {
    return (
      <div className="space-y-3">
        <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 mb-1">
          {t("payroll.select_employee", "Select Employee")}
        </div>
        <EmployeeSearch departments={departments}
        onChange={(value) => {
          setAssigneeId(Array.isArray(value) ? value[0] : value);
        }}
        placeholder={t("payroll.search_employees", "Search employees…")}
        value={assigneeId}  />
      </div>
    );
  }

  if (assigneeType === "department") {
    return (
      <div className="space-y-3">
        <label
          className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1"
          htmlFor="dept-select"
        >
          {t("payroll.select_department", "Select Department")}
        </label>
        <select
          className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-sky-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm"
          id="dept-select"
          onChange={(e) => {
            setAssigneeId(e.target.value);
          }}
          required
          value={assigneeId}
        >
          <option value="">
            {t("payroll.select_department_placeholder", "— Select department —")}
          </option>
          {departments?.map((d) => (
            <option key={d.id} value={d.id}>
              {d.name}
            </option>
          ))}
        </select>
      </div>
    );
  }

  return (
    <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-100 dark:border-amber-800 rounded-2xl p-5 flex items-center gap-4">
      <Globe className="text-amber-600 dark:text-amber-400 shrink-0" size={20} />
      <Typography className="text-xs font-bold text-amber-800 dark:text-amber-200 uppercase tracking-tight" variant="p">
        {t(
          "payroll.organization_scope_info",
          "This rule will apply to all employees in the organization"
        )}
      </Typography>
    </div>
  );
}
