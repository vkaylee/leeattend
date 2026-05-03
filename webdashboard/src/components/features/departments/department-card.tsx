import type { TFunction } from "i18next";

import { Pencil, Trash2, Building2, UserCircle, Users, CalendarDays, Calculator } from "lucide-react";
import React from "react";

import type { Department, User, WorkingPattern } from "@/lib/api";

import { useConfirm } from "@/hooks/use-confirm";

export interface DepartmentCardProps {
  deleteMutationMutate: (id: string) => void;
  dept: Department;
  onConfig: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  patterns: WorkingPattern[];
  t: TFunction;
  users: User[];
}

export function DepartmentCard({ dept, users, patterns, onEdit, onConfig, deleteMutationMutate, t }: DepartmentCardProps) {
  const { confirm, ConfirmationDialog } = useConfirm();

  const handleDelete = async () => {
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Warning" }),
      message: t("departments.messages.confirm_delete"),
      variant: "danger"
    });
    if(ok) deleteMutationMutate(dept.id);
  };

  return (
    <>
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col">
        <div className="absolute top-0 right-0 p-8 flex gap-3 translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-500">
          <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-emerald-600 dark:text-emerald-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onConfig(dept); }} title={t("payroll.settings.title")}><Calculator size={18} /></button>
          <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onEdit(dept); }} title={t("common.edit")}><Pencil size={18} /></button>
          <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { void handleDelete(); }} title={t("common.delete")}><Trash2 size={18} /></button>
        </div>

        <div className="bg-primary-50 dark:bg-primary-950 w-16 h-16 rounded-[1.5rem] flex items-center justify-center text-primary-600 dark:text-primary-400 mb-8 shadow-inner border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 group-hover:rotate-3 transition-all duration-500">
          <Building2 size={32} />
        </div>
        
        <h2 className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight mb-3 uppercase leading-none">{dept.name}</h2>
        <p className="text-slate-500 dark:text-slate-400 text-xs mb-10 line-clamp-2 min-h-[3rem] font-bold uppercase tracking-tight opacity-60 leading-relaxed italic">{dept.description ?? t("common.no_description")}</p>
        
        <div className="mt-auto pt-8 border-t border-slate-50 dark:border-slate-800 flex flex-col gap-6">
          <div className="flex items-center gap-5 group/info">
            <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner group-hover/info:text-primary-600 dark:text-primary-400 group-hover/info:bg-primary-50 dark:bg-primary-950 group-hover/info:border-primary-100 dark:border-primary-800 transition-all"><Users size={20} /></div>
            <div>
              <span className="block text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] text-[9px] mb-1">{t("departments.active_employees_label")}</span>
              <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{dept.active_employee_count}</span>
            </div>
          </div>

          <div className="flex items-center gap-5 group/info">
            <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner group-hover/info:text-primary-600 dark:text-primary-400 group-hover/info:bg-primary-50 dark:bg-primary-950 group-hover/info:border-primary-100 dark:border-primary-800 transition-all"><UserCircle size={20} /></div>
            <div>
              <span className="block text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] text-[9px] mb-1">{t("departments.manager_label")}</span>
              <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{users.find(u => u.id === dept.manager_id)?.full_name ?? t("departments.not_assigned").toUpperCase()}</span>
            </div>
          </div>

          <div className="flex items-center gap-5 group/info">
            <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner group-hover/info:text-primary-600 dark:text-primary-400 group-hover/info:bg-primary-50 dark:bg-primary-950 group-hover/info:border-primary-100 dark:border-primary-800 transition-all"><CalendarDays size={20} /></div>
            <div>
              <span className="block text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] text-[9px] mb-1">{t("departments.default_pattern_label")}</span>
              <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{patterns.find(p => p.id === dept.default_working_pattern_id)?.name ?? t("departments.not_assigned").toUpperCase()}</span>
            </div>
          </div>
        </div>
      </div>
      <ConfirmationDialog />
    </>
  );
}
