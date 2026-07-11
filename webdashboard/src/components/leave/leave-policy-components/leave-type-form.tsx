import { Wallet, AlertTriangle } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface LeaveTypeFormProps {
  // Lower-cased codes of leave types already defined in the workspace.
  // Used for client-side duplicate detection (best-effort; the server
  // enforces the case-insensitive unique constraint as source of truth).
  existingCodes: string[];
  initialData: { code: string; name: string; is_paid: boolean };
  isSaving: boolean;
  onBackToTemplates: () => void;
  onCancel: () => void;
  onSave: (data: { code: string; name: string; is_paid: boolean }) => void;
}


export function LeaveTypeForm({
  existingCodes,
  initialData,
  isSaving,
  onSave,
  onBackToTemplates,
  onCancel,
}: LeaveTypeFormProps) {
  const { t } = useTranslation();
  const [formData, setFormData] = useState(initialData);

  const normalizedCode = formData.code.trim().toLowerCase();
  const isDuplicateCode =
    normalizedCode.length > 0 && existingCodes.includes(normalizedCode);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (isDuplicateCode) return;
    onSave(formData);
  };

  return (
    <div className="relative">
      <form className="space-y-6" onSubmit={handleSubmit}>
        <div className="space-y-1.5">
          <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest ml-1" htmlFor="type-code-input">
            {t("settings.leave.type_form.type_code", "Type Code")}
          </label>
          <input
            className="w-full rounded-2xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-4 text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all outline-none"
            id="type-code-input"
            onChange={(e) => {
              setFormData({ ...formData, code: e.target.value.toUpperCase() });
            }}
            placeholder={t("settings.leave.type_form.type_code_placeholder", "e.g. AL, SL, UL")}
            required
            type="text"
            value={formData.code}
          />
          {isDuplicateCode ? (
            <p className="mt-1.5 ml-1 text-[11px] font-semibold text-rose-500">
              {t(
                "settings.leave.type_manager.create_error_duplicate_code",
                "A leave type with this code already exists",
              )}
            </p>
          ) : null}
        </div>

        <div className="space-y-1.5">
          <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest ml-1" htmlFor="display-name-input">
            {t("settings.leave.type_form.display_name", "Display Name")}
          </label>
          <input
            className="w-full rounded-2xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-4 text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all outline-none"
            id="display-name-input"
            onChange={(e) => {
              setFormData({ ...formData, name: e.target.value });
            }}
            placeholder={t("settings.leave.type_form.display_name_placeholder", "e.g. Annual Leave")}
            required
            type="text"
            value={formData.name}
          />
        </div>

        <div className="flex items-center justify-between p-5 rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800">
          <div className="flex items-center gap-3">
            <div
              className={cn(
                "h-10 w-10 rounded-xl flex items-center justify-center transition-colors shadow-inner",
                formData.is_paid
                  ? "bg-emerald-50 dark:bg-emerald-950/50 text-emerald-600"
                  : "bg-amber-50 dark:bg-amber-950/50 text-amber-600"
              )}
            >
              {formData.is_paid ? <Wallet size={18} /> : <AlertTriangle size={18} />}
            </div>
            <div>
              <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">
                {t("settings.leave.type_form.paid_status", "Paid Status")}
              </span>
              <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">
                {formData.is_paid ? t("settings.leave.type_form.paid_time_off", "Paid time off") : t("settings.leave.type_form.unpaid_leave", "Unpaid leave")}
              </Typography>
            </div>
          </div>
          <div className="relative inline-flex items-center">
            <input
              checked={formData.is_paid}
              className="sr-only peer"
              id="paid-leave-input"
              onChange={(e) => {
                setFormData({ ...formData, is_paid: e.target.checked });
              }}
              type="checkbox"
            />
            <label
              className="w-11 h-6 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600 cursor-pointer"
              htmlFor="paid-leave-input"
            >
              <span className="sr-only">{t("settings.leave.type_form.paid_status", "Paid Status")}</span>
            </label>
          </div>
        </div>

        <div className="flex items-center justify-between pt-6 border-t border-slate-50 dark:border-slate-800">
          <Button
            className="flex items-center gap-2 text-[10px] text-slate-400 dark:text-slate-500 hover:text-primary-600"
            onClick={onBackToTemplates}
            size="sm"
            type="button"
            variant="ghost"
          >
            ← {t("settings.leave.type_form.change_template", "Change Template")}
          </Button>
          <div className="flex gap-3">
            <Button
              className="px-6 py-3 text-[10px] text-slate-400 dark:text-slate-500 hover:text-slate-600"
              onClick={onCancel}
              size="sm"
              type="button"
              variant="ghost"
            >
              {t("common.cancel", "Cancel")}
            </Button>
            <Button
              className="shadow-primary-500/20 text-[10px] tracking-[0.2em] py-3.5 px-8"
              disabled={isSaving || isDuplicateCode}
              size="sm"
              type="submit"
              variant="primary"
            >
              {isSaving ? t("settings.leave.type_form.creating", "Creating...") : t("settings.leave.type_form.confirm_create", "Confirm & Create")}
            </Button>
          </div>
        </div>
      </form>
    </div>
  );
}
