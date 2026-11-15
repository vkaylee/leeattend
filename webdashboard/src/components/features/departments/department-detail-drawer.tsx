import type { TFunction } from "i18next";

import { Pencil, Trash2, UserCircle, Users } from "lucide-react";
import React from "react";

import type { Department, Employee } from "@/lib/api";

import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";

export interface DepartmentDetailDrawerProps {
  dept: Department | null;
  employees: Employee[];
  isOpen: boolean;
  onClose: () => void;
  onDelete: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  t: TFunction;
}

interface EmployeeGridProps {
  deptEmployees: Employee[];
  t: TFunction;
}

function EmployeeGrid({ deptEmployees, t }: EmployeeGridProps) {
  if (deptEmployees.length === 0) {
    return (
      <div className="py-20 text-center text-[10px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">
        {t("departments.drawer.empty_employees")}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
      {deptEmployees.map((emp) => (
        <div
          className="group relative flex items-center gap-4 overflow-hidden rounded-[2rem] border border-slate-100 bg-slate-50 p-6 transition-all duration-300 hover:bg-white hover:shadow-xl hover:shadow-primary-100/10 dark:border-slate-800/80 dark:bg-slate-800/20 dark:hover:bg-slate-900"
          key={emp.id}
        >
          <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border border-primary-100 bg-primary-50 text-base font-bold text-primary-600 dark:border-primary-800/30 dark:bg-primary-950/50 dark:text-primary-400">
            {emp.full_name[0].toUpperCase()}
          </div>
          <div className="min-w-0">
            <div className="truncate text-sm font-bold uppercase leading-tight tracking-tight text-slate-700 dark:text-slate-200">
              {emp.full_name}
            </div>
            <div className="mt-1 truncate font-mono text-[10px] font-bold uppercase tracking-widest text-slate-500 opacity-75 dark:text-slate-400">
              {emp.employee_id_number}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

function MetadataSection({ activeEmployeeCount, managerName, t }: { activeEmployeeCount: number; managerName: string; t: TFunction }) {
  return (
    <div className="grid grid-cols-1 gap-6 border-b border-slate-100 pb-6 dark:border-slate-800 sm:grid-cols-2">
      <div className="flex items-center gap-4">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-slate-100 bg-slate-50 text-slate-400 dark:border-slate-800 dark:bg-slate-800 dark:text-slate-500">
          <UserCircle size={20} />
        </div>
        <div className="min-w-0">
          <span className="block text-[8px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("departments.manager_label")}</span>
          <span className="block truncate text-xs font-bold uppercase text-slate-700 dark:text-slate-200">{managerName}</span>
        </div>
      </div>
      <div className="flex items-center gap-4">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-slate-100 bg-slate-50 text-slate-400 dark:border-slate-800 dark:bg-slate-800 dark:text-slate-500">
          <Users size={20} />
        </div>
        <div>
          <span className="block text-[8px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("departments.active_employees_label")}</span>
          <span className="text-xs font-bold text-slate-700 dark:text-slate-200">{activeEmployeeCount}</span>
        </div>
      </div>
    </div>
  );
}

export function DepartmentDetailDrawer({ isOpen, onClose, dept, employees, onEdit, onDelete, t }: DepartmentDetailDrawerProps) {
  if (!dept) return null;

  const deptEmployees = employees.filter((e) => e.department_name === dept.name);
  const manager = employees.find((e) => e.id === dept.manager_id);
  const managerName = manager ? manager.full_name : t("departments.not_assigned").toUpperCase();

  return (
    <Sheet
      className="!max-w-4xl"
      headerExtra={(
        <div className="flex gap-2">
          <Button aria-label={t("common.edit")} className="hover:text-primary-600 dark:hover:text-primary-400" icon={<Pencil size={16} />} onClick={() => { onEdit(dept); }} size="sm" title={t("common.edit")} variant="outline" />
          <Button aria-label={t("common.delete")} className="hover:text-rose-600 dark:hover:text-rose-300" icon={<Trash2 size={16} />} onClick={() => { onDelete(dept); }} size="sm" title={t("common.delete")} variant="outline" />
        </div>
      )}
      isOpen={isOpen}
      onClose={onClose}
      subtitle={dept.description ?? t("common.no_description")}
      title={dept.name}
    >
      <div className="space-y-8">
        <MetadataSection activeEmployeeCount={dept.active_employee_count} managerName={managerName} t={t} />
        <EmployeeGrid deptEmployees={deptEmployees} t={t} />
      </div>
    </Sheet>
  );
}
