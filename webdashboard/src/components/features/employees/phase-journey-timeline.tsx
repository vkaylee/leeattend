import { Milestone, Pencil } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { EmployeePhase, EmploymentPhaseDefinition } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  definitions?: EmploymentPhaseDefinition[];
  formatDate: (date: string) => string;
  isLoading: boolean;
  onOpenStatusModal: (phase: EmployeePhase) => void;
  phases?: EmployeePhase[];
}

interface ItemProps {
  def?: EmploymentPhaseDefinition;
  formatDate: (date: string) => string;
  idx: number;
  onOpenStatusModal: (phase: EmployeePhase) => void;
  phase: EmployeePhase;
  t: (key: string) => string;
}

function PhaseTimelineItem({
  phase,
  def,
  idx,
  formatDate,
  onOpenStatusModal,
  t,
}: ItemProps) {
  const isCompleted = phase.status === "completed";
  const isTerminated = phase.status === "terminated";

  return (
    <div
      className="relative animate-in slide-in-from-left duration-500"
      style={{ animationDelay: `${idx * 100}ms` }}
    >
      <div
        className={cn(
          "absolute -left-[2.15rem] top-4 w-5 h-5 rounded-full border-4 border-white dark:border-slate-900 shadow-sm z-10",
          isCompleted
            ? "bg-emerald-500"
            : isTerminated
              ? "bg-rose-500"
              : "bg-primary-500 ring-4 ring-primary-50 dark:ring-primary-900/30"
        )}
      ></div>

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm group hover:shadow-md transition-all">
        <div className="flex justify-between items-start mb-4">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <Typography
                className="font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight"
                variant="h4"
              >
                {def?.name ?? "Unknown Phase"}
              </Typography>
              <span
                className={cn(
                  "text-[8px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full",
                  isCompleted
                    ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400"
                    : isTerminated
                      ? "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-400"
                      : "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400"
                )}
              >
                {phase.status}
              </span>
            </div>
            <Typography
              className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold tracking-widest"
              variant="p"
            >
              {formatDate(phase.start_date)} — {formatDate(phase.end_date)}
            </Typography>
          </div>

          <PermissionGate permission="employment_phases:manage">
            <Button
              className="p-2 text-slate-300 dark:text-slate-600 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-xl transition-all h-auto w-auto shadow-none"
              onClick={() => onOpenStatusModal(phase)}
              size="icon"
              title={t("employees.phases.update_status")}
              type="button"
              variant="ghost"
            >
              <Pencil size={14} />
            </Button>
          </PermissionGate>
        </div>

        {!!phase.evaluation_notes && (
          <div className="mt-4 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-2xl border border-slate-100 dark:border-slate-700/50">
            <Typography
              className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-1"
              variant="p"
            >
              {t("employees.phases.evaluation")}
            </Typography>
            <Typography className="text-sm text-slate-600 dark:text-slate-300" variant="p">
              {phase.evaluation_notes}
            </Typography>
          </div>
        )}
      </div>
    </div>
  );
}

export default function PhaseJourneyTimeline({
  phases,
  definitions,
  isLoading,
  formatDate,
  onOpenStatusModal,
}: Props) {
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <div className="p-10 text-center">
        <div className="w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div>
      </div>
    );
  }

  if (!phases || phases.length === 0) {
    return (
      <div className="p-10 text-center bg-slate-50 dark:bg-slate-800/30 rounded-[2rem] border border-dashed border-slate-200 dark:border-slate-700">
        <Milestone className="w-10 h-10 text-slate-300 dark:text-slate-600 mx-auto mb-3" />
        <Typography
          className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest"
          variant="p"
        >
          {t("employees.phases.no_milestones")}
        </Typography>
      </div>
    );
  }

  return (
    <div className="relative pl-10 space-y-6 before:absolute before:left-[19px] before:top-4 before:bottom-4 before:w-px before:bg-slate-200 dark:before:bg-slate-700">
      {phases.map((phase, idx) => {
        const def = definitions?.find((d) => d.id === phase.phase_definition_id);
        return (
          <PhaseTimelineItem
            def={def}
            formatDate={formatDate}
            idx={idx}
            key={phase.id}
            onOpenStatusModal={onOpenStatusModal}
            phase={phase}
            t={t}
          />
        );
      })}
    </div>
  );
}
