import type { TFunction } from "i18next";

import { Pencil, Trash2, Building2, UserCircle, Users, Calculator, MoreHorizontal } from "lucide-react";
import React from "react";

import type { Department, Employee } from "@/lib/api";

import Button from "@/components/ui/button";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";

export interface DepartmentCardProps {
  deleteMutationMutate: (id: string) => void;
  dept: Department;
  employees: Employee[];
  onClick: () => void;
  onConfig: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  t: TFunction;
}

export function DepartmentCard({ dept, employees, onEdit, onConfig, deleteMutationMutate, onClick, t }: DepartmentCardProps) {
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
      <div 
        className="cursor-pointer text-left bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 sm:p-10 rounded-2xl sm:rounded-[2.5rem] shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500"
        onClick={(e) => {
          if ((e.target as HTMLElement).closest("[data-actions-menu]")) {
            return;
          }
          onClick();
        }}
        onKeyDown={(e) => {
          if ((e.target as HTMLElement).closest("[data-actions-menu]")) {
            return;
          }
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onClick();
          }
        }}
        role="button"
        tabIndex={0}
      >
        <div className="absolute top-0 right-0 p-8 z-10" data-actions-menu>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                aria-label={t("departments.columns.actions")}
                className="hover:text-primary-600 dark:hover:text-primary-400"
                icon={<MoreHorizontal size={18} />}
                size="icon"
                variant="outline"
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => { onConfig(dept); }}>
                <Calculator className="mr-2 h-4 w-4" />
                {t("payroll.settings.title")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => { onEdit(dept); }}>
                <Pencil className="mr-2 h-4 w-4" />
                {t("common.edit")}
              </DropdownMenuItem>
              <DropdownMenuItem className="text-rose-600 dark:text-rose-400" onClick={() => { void handleDelete(); }}>
                <Trash2 className="mr-2 h-4 w-4" />
                {t("common.delete")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        <div className="bg-primary-50 dark:bg-primary-950 w-16 h-16 rounded-[1.5rem] flex items-center justify-center text-primary-600 dark:text-primary-400 mb-8 shadow-inner border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 group-hover:rotate-3 transition-all duration-500">
          <Building2 size={32} />
        </div>
        
        <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight mb-3 uppercase leading-none" variant="h2">{dept.name}</Typography>
        <Typography className="text-slate-500 dark:text-slate-400 text-xs mb-10 line-clamp-2 min-h-[3rem] font-bold uppercase tracking-tight opacity-60 leading-relaxed italic" variant="p">{dept.description ?? t("common.no_description")}</Typography>
        
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
              <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{employees.find(e => e.id === dept.manager_id)?.full_name ?? t("departments.not_assigned").toUpperCase()}</span>
            </div>
          </div>
        </div>
      </div>
      <ConfirmationDialog />
    </>
  );
}
