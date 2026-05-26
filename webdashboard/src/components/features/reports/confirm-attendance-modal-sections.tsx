import { RefreshCw, CheckCircle } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { AttendanceAdjustmentRequest } from "@/lib/api";

export function ExistingRequestBanner({ 
  existingRequest, 
  t 
}: { 
  existingRequest: AttendanceAdjustmentRequest; 
  t: (k: string) => string 
}) {
  return (
    <div className="p-4 bg-orange-50 dark:bg-orange-950 rounded-2xl border border-orange-200 dark:border-orange-800">
      <Typography className="text-xs font-bold text-orange-600 dark:text-orange-300 uppercase tracking-wider" variant="p">
        {t("reports.confirm.existing_request")}
      </Typography>
      {existingRequest.reason ? (
        <Typography className="text-sm text-orange-700 dark:text-orange-200 mt-2 italic" variant="p">
          &ldquo;{existingRequest.reason}&rdquo;
        </Typography>
      ) : null}
    </div>
  );
}

export function ReasonInput({ 
  reason, 
  setReason, 
  t 
}: { 
  reason: string; 
  setReason: (val: string) => void; 
  t: (k: string) => string 
}) {
  return (
    <div>
      <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">
        {t("reports.confirm.reason")}
      </label>
      <input
        className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent"
        onChange={(e) => { setReason(e.target.value); }}
        placeholder={t("reports.confirm.reason_placeholder")}
        type="text"
        value={reason}
      />
    </div>
  );
}

export function ActionButtons({ 
  isPending, 
  onClose, 
  onSubmit, 
  canSubmit, 
  isEdit, 
  t 
}: { 
  canSubmit: boolean;
  isEdit: boolean;
  isPending: boolean; 
  onClose: () => void; 
  onSubmit: () => void; 
  t: (k: string) => string;
}) {
  return (
    <div className="flex gap-4 border-t border-slate-100 dark:border-slate-800 pt-4">
      <Button
        className="flex-1 h-auto py-3.5 px-6 rounded-2xl font-bold text-xs uppercase tracking-widest"
        disabled={isPending}
        onClick={onClose}
        variant="ghost"
      >
        {t("common.cancel")}
      </Button>
      <Button
        className="flex-[1.5] h-auto py-3.5 px-6 bg-emerald-600 hover:bg-emerald-700 text-white font-bold rounded-2xl shadow-xl text-xs uppercase tracking-widest"
        disabled={isPending || !canSubmit}
        icon={isPending ? <RefreshCw className="w-4 h-4 animate-spin" /> : <CheckCircle size={18} />}
        onClick={onSubmit}
        variant="primary"
      >
        {isEdit ? t("reports.confirm.approve") : t("reports.confirm.submit")}
      </Button>
    </div>
  );
}
