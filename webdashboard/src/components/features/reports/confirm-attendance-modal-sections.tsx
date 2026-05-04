import { RefreshCw, CheckCircle } from "lucide-react";
import React from "react";

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
      <p className="text-xs font-bold text-orange-600 dark:text-orange-300 uppercase tracking-wider">
        {t("reports.confirm.existing_request")}
      </p>
      {existingRequest.reason ? (
        <p className="text-sm text-orange-700 dark:text-orange-200 mt-2 italic">
          &ldquo;{existingRequest.reason}&rdquo;
        </p>
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
      <button
        className="flex-1 px-6 py-3.5 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
        disabled={isPending}
        onClick={onClose}
      >
        {t("common.cancel")}
      </button>
      <button
        className="flex-[1.5] bg-emerald-600 hover:bg-emerald-700 text-white font-bold px-6 py-3.5 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-xl text-xs uppercase tracking-widest"
        disabled={isPending || !canSubmit}
        onClick={onSubmit}
      >
        {isPending ? (
          <RefreshCw className="w-4 h-4 animate-spin" />
        ) : (
          <CheckCircle size={18} />
        )}
        {isEdit ? t("reports.confirm.approve") : t("reports.confirm.submit")}
      </button>
    </div>
  );
}
