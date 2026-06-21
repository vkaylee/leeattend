import * as Dialog from "@radix-ui/react-dialog";
import { Upload, X, FileText } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { FileUpload } from "@/components/files/file-upload";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useEnterPaperOt } from "@/hooks/use-attendance";
import { useWorkspace } from "@/hooks/use-workspace";

import type { OtSummary } from "./ot-approvals-table";

interface AddPhotoDialogProps {
  employeeId?: string;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  open: boolean;
  summary: OtSummary | null;
}

export function AddPhotoDialog({
  open,
  onOpenChange,
  summary,
  onSuccess,
  employeeId,
}: AddPhotoDialogProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const { workspace } = useWorkspace();
  const { enterPaperOt, isSubmitting } = useEnterPaperOt();
  const [photoUrl, setPhotoUrl] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const wsId = workspace?.id ?? "";

  const handleSubmit = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!summary) return;
    if (!photoUrl) {
      setError("Please select and upload a PDF document");
      return;
    }
    if (!employeeId) {
      setError("Không tìm thấy thông tin nhân viên");
      return;
    }
    setError(null);
    try {
      await enterPaperOt({
        employeeId,
        date: summary.date,
        claimed_minutes: summary.claimed_ot_minutes,
        reason: summary.ot_reason ?? "Paper OT Claim",
        paper_form_photo_url: photoUrl,
      });
      setPhotoUrl(null);
      setFileName(null);
      onOpenChange(false);
      onSuccess?.();
      toast.success("Document evidence attached successfully");
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to upload document");
    }
  };

  return (
    <Dialog.Root onOpenChange={(o) => { if (!o) { setPhotoUrl(null); setFileName(null); setError(null); } onOpenChange(o); }} open={open}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 p-6 shadow-xl rounded-[2rem]">
          <Dialog.Title className="text-xl font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight flex items-center gap-2">
            <Upload className="w-5 h-5" />
            {t("ot_approvals.evidence.add", { defaultValue: "Add Document Evidence (PDF)" })}
          </Dialog.Title>
          <Dialog.Description className="text-sm text-slate-500 dark:text-slate-400 mt-1">
            {t("ot_approvals.evidence.description", { defaultValue: "Upload a PDF of the signed paper form for this claim." })}
          </Dialog.Description>

          <Button
            className="absolute right-4 top-4 h-auto w-auto p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
            icon={<X className="h-4 w-4" />}
            onClick={() => { onOpenChange(false); }}
            size="icon"
            variant="ghost"
          />

          <form className="mt-6 space-y-5" onSubmit={handleSubmit}>
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                {t("approvals.paper_ot.photo") || "Paper Form Document (PDF)"}
              </label>
              {photoUrl ? (
                <div className="border border-emerald-100 dark:border-emerald-800/50 bg-emerald-50/50 dark:bg-emerald-950/20 rounded-xl p-4 text-center relative group">
                  <FileText className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
                  <Typography className="text-xs font-semibold text-emerald-800 dark:text-emerald-300 animate-in fade-in" variant="p">
                    {fileName ?? "paper_form_evidence.pdf"}
                  </Typography>
                  <Button
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
                <div className="w-full">
                  <FileUpload
                    accept="application/pdf"
                    category="OtClaimAttachment"
                    maxSizeMB={20}
                    onUploadSuccess={(file) => {
                      setFileName(file.file_name);
                      setPhotoUrl(`/api/v1/workspaces/${wsId}/files/${file.id}/download`);
                    }}
                    ownerId={summary?.id ?? ""}
                    ownerType="AttendanceDailySummary"
                    workspaceId={wsId}
                  />
                </div>
              )}
            </div>

            {!!error && (
              <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3">
                <Typography className="text-[10px] font-bold text-rose-600 dark:text-rose-300 uppercase tracking-widest" variant="p">{error}</Typography>
              </div>
            )}

            <div className="flex justify-end gap-3 pt-2">
              <Button
                className="h-auto px-4 py-2 rounded-xl text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
                onClick={() => { onOpenChange(false); }}
                type="button"
                variant="ghost"
              >
                {t("common.cancel") || "Cancel"}
              </Button>
              <Button
                className="h-auto px-6 py-2 rounded-xl text-xs font-bold uppercase tracking-wider shadow-lg"
                disabled={isSubmitting}
                loading={isSubmitting}
                type="submit"
                variant="primary"
              >
                {t("common.submit") || "Submit"}
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
