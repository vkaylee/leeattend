import { X } from "lucide-react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

import { FileUpload } from "@/components/files/file-upload";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useCreateOvertimeRequest } from "@/hooks/use-overtime-requests";

const otRequestSchema = z.object({
  planned_date: z.string().min(1, "Date is required"),
  planned_start_time: z.string().min(1, "Start time is required"),
  planned_end_time: z.string().min(1, "End time is required"),
  planned_minutes: z.number().min(1, "Minutes must be > 0"),
  reason: z.string().min(1, "Reason is required"),
});

type OtRequestFormValues = z.infer<typeof otRequestSchema>;

export function CreateOtRequestModal({
  isOpen,
  onClose,
  wsId,
  employeeId,
}: {
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
  employeeId: string;
}) {
  const { t } = useTranslation();
  const toast = useToast();
  const { mutateAsync: createRequest, isPending } = useCreateOvertimeRequest();

  const [ownerId] = useState<string>(() => crypto.randomUUID());
  const [attachmentFileIds, setAttachmentFileIds] = useState<string[]>([]);

  const form = useForm<OtRequestFormValues>({
    resolver: zodResolver(otRequestSchema),
    defaultValues: {
      planned_date: new Date().toISOString().split("T")[0],
      planned_start_time: "17:00",
      planned_end_time: "19:00",
      planned_minutes: 120,
      reason: "",
    },
  });

  const handleClose = () => {
    setAttachmentFileIds([]);
    form.reset();
    onClose();
  };

  const onSubmit = async (data: OtRequestFormValues) => {
    try {
      await createRequest({
        planned_date: data.planned_date,
        planned_start_time: data.planned_start_time,
        planned_end_time: data.planned_end_time,
        planned_minutes: data.planned_minutes,
        reason: data.reason,
        employees: [employeeId],
        attachment_file_ids: attachmentFileIds,
      });
      toast.success(t("ess.ot.request_success", { defaultValue: "Overtime request created successfully" }));
      handleClose();
    } catch (error) {
      console.error(error);
      toast.error(t("common.error"));
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title={t("ess.ot.request_title", { defaultValue: "Request Overtime" })}>
      <form className="space-y-6" onSubmit={form.handleSubmit(onSubmit)}>
        <div className="space-y-4">
          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="planned_date">Date</label>
            <input
              {...form.register("planned_date")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              id="planned_date"
              type="date"
            />
            {!!form.formState.errors.planned_date && <Typography className="text-rose-500 text-xs mt-1" variant="p">{form.formState.errors.planned_date.message}</Typography>}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="planned_start_time">Start Time</label>
              <input
                {...form.register("planned_start_time")}
                className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                id="planned_start_time"
                type="time"
              />
            </div>
            <div>
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="planned_end_time">End Time</label>
              <input
                {...form.register("planned_end_time")}
                className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                id="planned_end_time"
                type="time"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="planned_minutes">Total Minutes</label>
            <input
              {...form.register("planned_minutes", { valueAsNumber: true })}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              id="planned_minutes"
              type="number"
            />
            {!!form.formState.errors.planned_minutes && <Typography className="text-rose-500 text-xs mt-1" variant="p">{form.formState.errors.planned_minutes.message}</Typography>}
          </div>

          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="reason">Reason</label>
            <textarea
              {...form.register("reason")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              id="reason"
              placeholder="Why do you need to work overtime?"
              rows={3}
            />
            {!!form.formState.errors.reason && <Typography className="text-rose-500 text-xs mt-1" variant="p">{form.formState.errors.reason.message}</Typography>}
          </div>

          <div className="space-y-3">
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">
              {t("ess.ot.attachment", { defaultValue: "Attachment (PDF ONLY)" })}
            </label>
            <FileUpload
              accept="application/pdf"
              category="OvertimeRequestAttachment"
              maxSizeMB={20}
              onUploadSuccess={(file) => {
                setAttachmentFileIds(prev => [...prev, file.id]);
              }}
              ownerId={ownerId}
              ownerType="OvertimeRequest"
              workspaceId={wsId}
            />
            {attachmentFileIds.length > 0 && (
              <div className="mt-3 space-y-2">
                {attachmentFileIds.map((id, index) => (
                  <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/50 rounded-xl border border-slate-200 dark:border-slate-700 text-xs" key={id}>
                    <span className="font-mono text-slate-700 dark:text-slate-350">Document #{index + 1} ({id})</span>
                    <Button
                      className="p-1 h-6 w-6 text-rose-650 hover:bg-rose-100 dark:hover:bg-rose-950/50 rounded-lg transition-colors border-none shadow-none"
                      onClick={() => {
                        setAttachmentFileIds(prev => prev.filter(fId => fId !== id));
                      }}
                      type="button"
                      variant="ghost"
                    >
                      <X className="w-4 h-4" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={handleClose} type="button" variant="ghost">
            {t("common.cancel")}
          </Button>
          <Button loading={isPending} type="submit" variant="primary">
            {t("common.submit")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
