import type { TFunction } from "i18next";

import { Briefcase, FileText, MapPin, Pencil, Trash2, Users } from "lucide-react";
import React from "react";

import type { Employer, Employee } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";

export interface EmployerCardProps {
  employees: Employee[];
  employer: Employer;
  onDelete: (id: string) => Promise<void>;
  onEdit: (employer: Employer) => void;
  t: TFunction;
}

export function EmployerCard({ employer, employees, onEdit, onDelete, t }: EmployerCardProps) {
  const { confirm, ConfirmationDialog } = useConfirm();

  const linkedEmployeeCount = employees.filter((e) => e.employer_id === employer.id).length;

  const handleDelete = async () => {
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Cảnh báo" }),
      message: t("settings.employers.messages.confirm_delete", {
        defaultValue: "Bạn có chắc chắn muốn xóa pháp nhân này? Hành động này không thể hoàn tác.",
      }),
      variant: "danger",
    });
    if (ok) {
      await onDelete(employer.id);
    }
  };

  return (
    <>
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col min-h-[300px]">
        {/* Actions overlay visible on hover */}
        <div className="absolute top-0 right-0 p-8 flex gap-3 translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-500">
          <Button
            className="hover:text-primary-600 dark:hover:text-primary-400"
            icon={<Pencil size={18} />}
            onClick={() => { onEdit(employer); }}
            size="icon"
            title={t("common.edit", { defaultValue: "Chỉnh sửa" })}
            variant="outline"
          />
          <Button
            className="hover:text-rose-600 dark:hover:text-rose-300"
            icon={<Trash2 size={18} />}
            onClick={() => { void handleDelete(); }}
            size="icon"
            title={t("common.delete", { defaultValue: "Xóa" })}
            variant="outline"
          />
        </div>

        {/* Icon Header */}
        <div className="bg-primary-50 dark:bg-primary-950 w-16 h-16 rounded-[1.5rem] flex items-center justify-center text-primary-600 dark:text-primary-400 mb-8 shadow-inner border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 group-hover:rotate-3 transition-all duration-500">
          <Briefcase size={32} />
        </div>

        {/* Employer details */}
        <Typography
          className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight mb-6 uppercase leading-tight line-clamp-2 min-h-[3.5rem]"
          variant="h2"
        >
          {employer.name}
        </Typography>

        <div className="space-y-4 mb-8">
          {/* Tax Code */}
          <div className="flex items-center gap-3 text-slate-500 dark:text-slate-400">
            <FileText className="w-4 h-4 text-slate-400 shrink-0" />
            <span className="text-xs font-semibold tracking-tight">
              {t("settings.employers.tax_code", { defaultValue: "Mã số thuế" })}:{" "}
              <span className="font-bold text-slate-700 dark:text-slate-300">
                {employer.tax_code ?? t("common.not_assigned", { defaultValue: "Chưa thiết lập" })}
              </span>
            </span>
          </div>

          {/* Legal Address */}
          <div className="flex items-start gap-3 text-slate-500 dark:text-slate-400">
            <MapPin className="w-4 h-4 text-slate-400 shrink-0 mt-0.5" />
            <span className="text-xs font-semibold tracking-tight leading-relaxed line-clamp-2">
              {t("settings.employers.legal_address", { defaultValue: "Địa chỉ pháp lý" })}:{" "}
              <span className="font-medium text-slate-700 dark:text-slate-300">
                {employer.legal_address ?? t("common.not_assigned", { defaultValue: "Chưa thiết lập" })}
              </span>
            </span>
          </div>
        </div>

        {/* Linked Employee Count */}
        <div className="mt-auto pt-8 border-t border-slate-50 dark:border-slate-800">
          <div className="flex items-center gap-5 group/info">
            <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner group-hover/info:text-primary-600 dark:text-primary-400 group-hover/info:bg-primary-50 dark:bg-primary-950 group-hover/info:border-primary-100 dark:border-primary-800 transition-all">
              <Users size={20} />
            </div>
            <div>
              <span className="block text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] text-[9px] mb-1">
                {t("settings.employers.active_employees_label", { defaultValue: "Nhân viên trực thuộc" })}
              </span>
              <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">
                {linkedEmployeeCount} {t("settings.employers.employee_unit", { defaultValue: "nhân sự" })}
              </span>
            </div>
          </div>
        </div>
      </div>
      <ConfirmationDialog />
    </>
  );
}
