import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import Button from "@/components/ui/button";
import FormField, { INPUT_BASE, LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { updateEmployeeEmployment } from "@/lib/api";
import { cn } from "@/lib/utils";

interface TerminateEmployeeModalProps {
  employeeId: string | null;
  activeEmploymentId: string | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
  wsId: string;
}

const TERMINATION_REASONS = [
  "voluntary_resignation",
  "involuntary_termination",
  "end_of_contract",
  "retirement",
  "mutual_agreement",
  "abandonment",
  "death",
  "other",
] as const;

export default function TerminateEmployeeModal({
  isOpen,
  onClose,
  employeeId,
  activeEmploymentId,
  wsId,
  onSuccess,
}: TerminateEmployeeModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const terminateMutation = useMutation({
    mutationFn: ({ endDate, reason, notes }: { endDate: string; reason: string; notes: string }) => {
      if (!employeeId) throw new Error("No employee selected");
      if (!activeEmploymentId) throw new Error("No active employment found");
      return updateEmployeeEmployment(wsId, employeeId, activeEmploymentId, {
        status: "terminated",
        end_date: endDate,
        termination_reason: reason,
        notes: notes || null,
      });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["deptHistory", employeeId] });
      void queryClient.invalidateQueries({ queryKey: ["employee-employments", wsId, employeeId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  const handleTerminateSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    terminateMutation.mutate({
      endDate: formData.get("endDate") as string,
      reason: formData.get("reason") as string,
      notes: formData.get("notes") as string,
    });
  };

  return (
    <Modal
      icon={<Trash2 className="text-rose-600 dark:text-rose-400" size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("employees.terminate.subtitle")}
      title={t("employees.terminate.title")}
    >
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <div className="p-8 bg-rose-50/50 dark:bg-rose-950/20 rounded-[2.5rem] border border-rose-100/50 dark:border-rose-950/30 shadow-inner text-center italic">
          <Typography className="text-sm font-bold text-rose-700 dark:text-rose-300 uppercase tracking-tight leading-relaxed" variant="p">
            &quot;{t("employees.terminate.confirm_msg")}&quot;
          </Typography>
        </div>
        <form className="space-y-8" onSubmit={handleTerminateSubmit}>
          <FormField
            defaultValue={format(new Date(), "yyyy-MM-dd")}
            label={t("employees.terminate.terminate_date")}
            name="endDate"
            required
            type="date"
          />

          <div className="space-y-3 px-1">
            <label className={LABEL_BASE} htmlFor="reason">
              {t("employees.terminate.termination_reason_label")}
            </label>
            <select
              className={cn(
                "w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm appearance-none cursor-pointer"
              )}
              id="reason"
              name="reason"
              required
              defaultValue=""
            >
              <option value="" disabled>
                {t("employees.terminate.termination_reason_placeholder")}
              </option>
              {TERMINATION_REASONS.map((key) => (
                <option key={key} value={key}>
                  {t(`employees.terminate.reason.${key}`)}
                </option>
              ))}
            </select>
          </div>

          <FormField
            label={t("employees.terminate.termination_notes_label")}
            placeholder={t("employees.terminate.termination_notes_placeholder")}
            name="notes"
            type="textarea"
            rows={3}
          />

          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button onClick={onClose} size="lg" type="button" variant="outline">
              {t("employees.terminate.cancel").toUpperCase()}
            </Button>
            <Button
              data-testid="confirm-terminate-btn"
              loading={terminateMutation.isPending}
              size="lg"
              type="submit"
              variant="destructive"
            >
              {terminateMutation.isPending
                ? t("employees.terminate.processing").toUpperCase()
                : t("employees.terminate.confirm").toUpperCase()}
            </Button>
          </div>
        </form>
      </div>
    </Modal>
  );
}
