import type { TFunction } from "i18next";

import { Upload, FileText, X } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import type { ClaimItem } from "./confirm-claims-dialog";

import { ClaimRow } from "./claim-row";

interface ClaimsFormViewProps {
  claims: ClaimItem[];
  error: string | null;
  fileName: string | null;
  handleFileChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  handleMinutesChange: (index: number, val: number) => void;
  handleReasonChange: (index: number, val: string) => void;
  isSubmitting: boolean;
  onCancel: () => void;
  onSubmit: (e: React.SyntheticEvent) => void;
  photoUrl: string | null;
  setFileName: (val: string | null) => void;
  setPhotoUrl: (val: string | null) => void;
  t: TFunction;
}

export function ClaimsFormView({
  claims,
  error,
  fileName,
  handleFileChange,
  handleMinutesChange,
  handleReasonChange,
  isSubmitting,
  onCancel,
  onSubmit,
  photoUrl,
  setFileName,
  setPhotoUrl,
  t,
}: ClaimsFormViewProps) {
  return (
    <form className="mt-6 space-y-6" onSubmit={onSubmit}>
      {/* Claims Table */}
      <div className="border border-slate-100 dark:border-slate-800 rounded-2xl overflow-hidden">
        <div className="max-h-[250px] overflow-y-auto custom-scrollbar">
          <table className="w-full text-left border-collapse">
            <thead className="sticky top-0 bg-slate-50 dark:bg-slate-800/80 backdrop-blur border-b border-slate-100 dark:border-slate-800 z-10">
              <tr>
                <th className="p-3 pl-4 text-[10px] font-bold text-slate-500 uppercase tracking-wider">
                  {t("ot_approvals.delegated_claim.table.employee")}
                </th>
                <th className="p-3 text-[10px] font-bold text-slate-500 uppercase tracking-wider w-[140px]">
                  {t("ot_approvals.delegated_claim.table.computed_ot")}
                </th>
                <th className="p-3 text-[10px] font-bold text-slate-500 uppercase tracking-wider w-[120px]">
                  {t("approvals.paper_ot.minutes")}
                </th>
                <th className="p-3 pr-4 text-[10px] font-bold text-slate-500 uppercase tracking-wider w-[200px]">
                  {t("common.reason")}
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
              {claims.map((claim, idx) => (
                <ClaimRow
                  claim={claim}
                  handleMinutesChange={handleMinutesChange}
                  handleReasonChange={handleReasonChange}
                  idx={idx}
                  key={`${claim.employment_id}-${claim.date}`}
                  t={t}
                />
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Photo Evidence Uploader */}
      <div className="space-y-2">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] flex items-center justify-between">
          <span>{t("approvals.paper_ot.photo")}</span>
          {!!photoUrl && (
            <span className="text-[9px] text-amber-600 dark:text-amber-400 font-normal lowercase tracking-normal">
              *{t("ot_approvals.delegated_claim.paper_warning")}
            </span>
          )}
        </label>
        {photoUrl ? (
          <div className="border border-emerald-100 dark:border-emerald-800/50 bg-emerald-50/50 dark:bg-emerald-950/20 rounded-xl p-4 text-center relative group">
            <FileText className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
            <Typography className="text-xs font-semibold text-emerald-800 dark:text-emerald-300" variant="p">
              {fileName ?? "paper_form_evidence.png"}
            </Typography>
            <Button
              aria-label={t("common.remove_photo")}
              className="absolute top-2 right-2 p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-full h-auto w-auto shadow-none"
              onClick={() => {
                setPhotoUrl(null);
                setFileName(null);
              }}
              type="button"
              variant="ghost"
            >
              <X size={14} />
            </Button>
          </div>
        ) : (
          <div className="relative border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-xl p-5 text-center hover:border-primary-400 dark:hover:border-primary-500 transition-colors group cursor-pointer">
            <input
              accept="image/*"
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
              onChange={handleFileChange}
              type="file"
            />
            <Upload className="w-6 h-6 text-slate-400 dark:text-slate-500 mx-auto mb-2 group-hover:text-primary-500 transition-colors" />
            <Typography className="text-xs font-semibold text-slate-600 dark:text-slate-300" variant="p">
              {t("approvals.paper_ot.upload_prompt")}
            </Typography>
          </div>
        )}
      </div>

      {!!error && (
        <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3">
          <Typography className="text-xs font-semibold text-rose-600 dark:text-rose-300" variant="p">
            {error}
          </Typography>
        </div>
      )}

      {/* Form Actions */}
      <div className="flex justify-end gap-3 pt-2">
        <Button
          className="h-auto px-4 py-2 rounded-xl text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
          onClick={onCancel}
          type="button"
          variant="ghost"
        >
          {t("common.cancel")}
        </Button>
        <Button
          className="h-auto px-6 py-2 rounded-xl text-xs font-bold uppercase tracking-wider shadow-lg"
          disabled={isSubmitting}
          loading={isSubmitting}
          type="submit"
          variant="primary"
        >
          {t("common.submit")}
        </Button>
      </div>
    </form>
  );
}
