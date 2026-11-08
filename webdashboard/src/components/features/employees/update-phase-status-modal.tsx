import { useMutation, useQueryClient } from "@tanstack/react-query";
import { CheckCircle2, XCircle } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { updateEmployeePhaseStatus } from "@/lib/api/employees";
import { EmployeePhase } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  employeeId: string;
  isOpen: boolean;
  onClose: () => void;
  selectedPhase: EmployeePhase;
}

export default function UpdatePhaseStatusModal({
  isOpen,
  onClose,employeeId,
  selectedPhase,
}: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const [statusData, setStatusData] = useState({
    status: selectedPhase.status === "active" ? "completed" : selectedPhase.status,
    evaluation_notes: selectedPhase.evaluation_notes ?? "",
  });

  const statusMutation = useMutation({
    mutationFn: (data: { phaseId: string; status: string; evaluation_notes: string }) =>
      updateEmployeePhaseStatus(employeeId, data.phaseId, {
        status: data.status,
        evaluation_notes: data.evaluation_notes,
      }),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["employee-phases", employeeId] });
      onClose();
    },
    onError: (err: unknown) => {
      const error = err as Error;
      toast.error(error.message || t("common.error"));
    },
  });

  const handleStatusSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    statusMutation.mutate({
      phaseId: selectedPhase.id,
      ...statusData,
    });
  };

  return (
    <Modal
      icon={<CheckCircle2 size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("employees.phases.update_status_subtitle")}
      title={t("employees.phases.update_status")}
    >
      <form className="space-y-6 pt-4 animate-in fade-in" onSubmit={handleStatusSubmit}>
        <div className="space-y-3">
          <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 block">
            {t("employees.phases.status_new")}
          </span>
          <div className="flex gap-4">
            <Button
              className={cn(
                "flex-1 py-3 px-4 rounded-xl font-bold text-xs uppercase tracking-widest border transition-all flex items-center justify-center gap-2 h-auto shadow-none border-transparent",
                statusData.status === "completed"
                  ? "bg-emerald-50 text-emerald-600 border-emerald-200 hover:bg-emerald-50"
                  : "bg-white text-slate-400 border-slate-200 hover:border-emerald-200 hover:bg-slate-50"
              )}
              onClick={() => { setStatusData({ ...statusData, status: "completed" }); }}
              type="button"
              variant="ghost"
            >
              <CheckCircle2 size={16} /> Completed
            </Button>
            <Button
              className={cn(
                "flex-1 py-3 px-4 rounded-xl font-bold text-xs uppercase tracking-widest border transition-all flex items-center justify-center gap-2 h-auto shadow-none border-transparent",
                statusData.status === "terminated"
                  ? "bg-rose-50 text-rose-600 border-rose-200 hover:bg-rose-50"
                  : "bg-white text-slate-400 border-slate-200 hover:border-rose-200 hover:bg-slate-50"
              )}
              onClick={() => { setStatusData({ ...statusData, status: "terminated" }); }}
              type="button"
              variant="ghost"
            >
              <XCircle size={16} /> Terminated
            </Button>
          </div>
        </div>

        <div className="space-y-3">
          <label
            className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1"
            htmlFor="evaluation_notes"
          >
            {t("employees.phases.evaluation_notes")}
          </label>
          <textarea
            className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm min-h-[120px] resize-none"
            id="evaluation_notes"
            onChange={(e) => { setStatusData({ ...statusData, evaluation_notes: e.target.value }); }}
            placeholder={t("employees.phases.evaluation_notes_placeholder")}
            value={statusData.evaluation_notes}
          />
        </div>

        <div className="flex gap-4 pt-4 border-t border-slate-50 dark:border-slate-800">
          <Button
            className="flex-1 px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95 h-auto shadow-none"
            onClick={onClose}
            type="button"
            variant="outline"
          >
            {t("common.cancel")}
          </Button>
          <Button
            className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 h-auto"
            disabled={statusMutation.isPending}
            type="submit"
            variant="primary"
          >
            {statusMutation.isPending ? t("common.saving") : t("common.update")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
