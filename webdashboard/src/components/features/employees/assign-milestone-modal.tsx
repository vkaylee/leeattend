import { useMutation, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle, Milestone } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { assignEmployeePhase } from "@/lib/api/employees";
import { EmployeeContract, EmploymentPhaseDefinition } from "@/lib/api/types";

interface Props {
  contracts: EmployeeContract[];
  definitions?: EmploymentPhaseDefinition[];
  employeeId: string;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

interface WarningProps {
  title: string;
  warning: string | null;
}

function WarningAlert({ warning, title }: WarningProps) {
  if (!warning) return null;
  return (
    <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800/50 p-4 rounded-2xl flex gap-3 items-start animate-in fade-in slide-in-from-top-2">
      <AlertTriangle className="text-amber-500 shrink-0 mt-0.5" size={18} />
      <div>
        <Typography
          className="text-[11px] font-bold text-amber-800 dark:text-amber-500 uppercase tracking-widest mb-1"
          variant="p"
        >
          {title}
        </Typography>
        <Typography
          className="text-xs text-amber-700 dark:text-amber-600 font-medium leading-relaxed"
          variant="p"
        >
          {warning}
        </Typography>
      </div>
    </div>
  );
}

  // eslint-disable-next-line max-lines-per-function
export default function AssignMilestoneModal({
  isOpen,
  onClose,
  wsId,
  employeeId,
  definitions,
  contracts,
}: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const [assignData, setAssignData] = useState({
    phase_definition_id: "",
    start_date: "",
    end_date: "",
  });

  const assignMutation = useMutation({
    mutationFn: (data: typeof assignData) => assignEmployeePhase(wsId, employeeId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["employee-phases", employeeId] });
      onClose();
      setAssignData({ phase_definition_id: "", start_date: "", end_date: "" });
    },
    onError: (err: unknown) => {
      const error = err as Error;
      toast.error(error.message || t("common.error"));
    },
  });

  const isOutOfContractBounds = useMemo(() => {
    if (!assignData.start_date || !assignData.end_date || contracts.length === 0) {
      return contracts.length === 0 ? t("employees.phases.no_contracts_warning") : null;
    }
    const startDates = contracts.map((c) => new Date(c.start_date).getTime());
    const endDates = contracts.map((c) => (c.end_date ? new Date(c.end_date).getTime() : Infinity));
    const minStart = Math.min(...startDates);
    const maxEnd = Math.max(...endDates);
    const phaseStart = new Date(assignData.start_date).getTime();
    const phaseEnd = new Date(assignData.end_date).getTime();

    if (phaseStart < minStart || phaseEnd > maxEnd) {
      return t("employees.phases.out_of_bounds_warning");
    }
    return null;
  }, [assignData.start_date, assignData.end_date, contracts, t]);

  const handleAssignSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!assignData.phase_definition_id || !assignData.start_date || !assignData.end_date) {
      toast.error(t("common.required_fields"));
      return;
    }
    assignMutation.mutate(assignData);
  };

  const handlePhaseChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newPhaseId = e.target.value;
    const def = definitions?.find((d) => d.id === newPhaseId);
    let newEndDate = assignData.end_date;

    if (def?.rule_config?.duration_days && assignData.start_date) {
      const startDate = new Date(assignData.start_date);
      startDate.setDate(startDate.getDate() + def.rule_config.duration_days);
      newEndDate = startDate.toISOString().split("T")[0];
    }

    setAssignData({ ...assignData, phase_definition_id: newPhaseId, end_date: newEndDate });
  };

  const handleStartDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newStartDate = e.target.value;
    const def = definitions?.find((d) => d.id === assignData.phase_definition_id);
    let newEndDate = assignData.end_date;

    if (def?.rule_config?.duration_days && newStartDate) {
      const startDate = new Date(newStartDate);
      startDate.setDate(startDate.getDate() + def.rule_config.duration_days);
      newEndDate = startDate.toISOString().split("T")[0];
    }

    setAssignData({ ...assignData, start_date: newStartDate, end_date: newEndDate });
  };

  return (
    <Modal
      icon={<Milestone size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("employees.phases.add_milestone_subtitle")}
      title={t("employees.phases.add_milestone")}
    >
      <form className="space-y-6 pt-4 animate-in fade-in" onSubmit={handleAssignSubmit}>
        <div className="space-y-3">
          <label
            className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1"
            htmlFor="phase_definition_id"
          >
            {t("employees.phases.milestone_type")}
          </label>
          <select
            className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
            id="phase_definition_id"
            onChange={handlePhaseChange}
            required
            value={assignData.phase_definition_id}
          >
            <option value="">{t("employees.phases.select_milestone")}</option>
            {definitions
              ?.filter((d) => d.is_active)
              .map((def) => (
                <option key={def.id} value={def.id}>
                  {def.name} ({def.phase_type})
                </option>
              ))}
          </select>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-3">
            <label
              className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1"
              htmlFor="start_date"
            >
              {t("common.startDate")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
              id="start_date"
              onChange={handleStartDateChange}
              required
              type="date"
              value={assignData.start_date}
            />
          </div>
          <div className="space-y-3">
            <label
              className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1"
              htmlFor="end_date"
            >
              {t("common.endDate")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
              id="end_date"
              onChange={(e) => { setAssignData({ ...assignData, end_date: e.target.value }); }}
              required
              type="date"
              value={assignData.end_date}
            />
          </div>
        </div>

        <WarningAlert title={t("employees.phases.soft_warning")} warning={isOutOfContractBounds} />

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
            disabled={assignMutation.isPending}
            type="submit"
            variant="primary"
          >
            {assignMutation.isPending ? t("common.saving") : t("employees.phases.add_milestone")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
