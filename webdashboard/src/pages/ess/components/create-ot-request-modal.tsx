import React from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
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

  const onSubmit = async (data: OtRequestFormValues) => {
    try {
      await createRequest({
        planned_date: data.planned_date,
        planned_start_time: data.planned_start_time,
        planned_end_time: data.planned_end_time,
        planned_minutes: data.planned_minutes,
        reason: data.reason,
        employees: [employeeId],
      });
      toast.success(t("ess.ot.request_success", { defaultValue: "Overtime request created successfully" }));
      onClose();
    } catch (error) {
      console.error(error);
      toast.error(t("common.error"));
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={t("ess.ot.request_title", { defaultValue: "Request Overtime" })}>
      <form className="space-y-6" onSubmit={form.handleSubmit(onSubmit)}>
        <div className="space-y-4">
          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Date</label>
            <input
              {...form.register("planned_date")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              type="date"
            />
            {!!form.formState.errors.planned_date && <p className="text-rose-500 text-xs mt-1">{form.formState.errors.planned_date.message}</p>}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Start Time</label>
              <input
                {...form.register("planned_start_time")}
                className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                type="time"
              />
            </div>
            <div>
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">End Time</label>
              <input
                {...form.register("planned_end_time")}
                className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                type="time"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Total Minutes</label>
            <input
              {...form.register("planned_minutes", { valueAsNumber: true })}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              type="number"
            />
            {!!form.formState.errors.planned_minutes && <p className="text-rose-500 text-xs mt-1">{form.formState.errors.planned_minutes.message}</p>}
          </div>

          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Reason</label>
            <textarea
              {...form.register("reason")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              placeholder="Why do you need to work overtime?"
              rows={3}
            />
            {!!form.formState.errors.reason && <p className="text-rose-500 text-xs mt-1">{form.formState.errors.reason.message}</p>}
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <button
            className="px-5 py-2.5 rounded-xl font-bold text-sm text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            onClick={onClose}
            type="button"
          >
            {t("common.cancel")}
          </button>
          <button
            className="px-5 py-2.5 rounded-xl font-bold text-sm bg-indigo-600 hover:bg-indigo-700 text-white shadow-sm transition-all disabled:opacity-50"
            disabled={isPending}
            type="submit"
          >
            {isPending ? t("common.submitting") : t("common.submit")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
