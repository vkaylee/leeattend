import React from "react";

import Button from "@/components/ui/button";

interface ModalActionsProps {
  isDisabled: boolean;
  isSubmitting: boolean;
  onClose: () => void;
  submitLabel: string;
  t: (key: string, defaultValue?: string) => string;
}

export function ModalActions({
  onClose,
  isSubmitting,
  isDisabled,
  submitLabel,
  t
}: ModalActionsProps) {
  return (
    <div className="flex gap-4 pt-2 border-t border-slate-50 dark:border-slate-800">
      <Button className="flex-1 px-6 py-3.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
        onClick={onClose}
        type="button"
        variant="ghost"
      >
        {t("common.cancel", "Cancel")}
      </Button>
      <Button className="flex-[1.5] bg-teal-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-teal-700 transition-all shadow-xl shadow-teal-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed py-3.5"
        disabled={isDisabled}
        type="submit"
        variant="primary"
      >
        {isSubmitting
          ? t("common.saving", "Saving") + "…"
          : submitLabel}
      </Button>
    </div>
  );
}
