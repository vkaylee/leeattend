import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2, ShieldCheck, Plus, Trash2, Clock, ArrowRight } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import {
  fetchPolicyAssignments,
  upsertPolicyAssignment,
  deletePolicyAssignment,
  fetchDefaultAttendanceRules,
} from "@/lib/api/attendance-rules";

interface Props {
  employeeId: string;
}

function formatRules(rules: { work_unit_minutes: number; rounding_policy: string; early_arrival_strategy?: string | null; grace_period_minutes?: number | null }) {
  const parts: string[] = [];
  parts.push(`${rules.work_unit_minutes}min`);
  if (rules.rounding_policy && rules.rounding_policy !== "none") parts.push(`round:${rules.rounding_policy}`);
  if (rules.grace_period_minutes) parts.push(`grace:${rules.grace_period_minutes}m`);
  return parts.join(" | ");
}

  // eslint-disable-next-line complexity, max-lines-per-function
export default function EmployeeAttendancePolicyTab({ employeeId}: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { confirm, ConfirmationDialog } = useConfirm();

  const { data: defaultRules } = useQuery({ queryKey: ["attendance-rules"], queryFn: () => fetchDefaultAttendanceRules(),
 });

  const { data: assignments = [], isLoading } = useQuery({ queryKey: ["policy-assignments"], queryFn: () => fetchPolicyAssignments(),
 });

  const employeeAssignment = assignments.find((a) => a.target_type === "USER" && a.target_id === employeeId);

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deletePolicyAssignment(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["policy-assignments"] });
    },
  });

  const createMutation = useMutation({
    mutationFn: () =>
      upsertPolicyAssignment({
        target_id: employeeId,
        target_type: "USER",
        work_unit_minutes: defaultRules?.work_unit_minutes ?? 15,
        rounding_policy: defaultRules?.rounding_policy ?? "none",
        early_arrival_strategy: defaultRules?.early_arrival_strategy ?? "block",
        grace_period_minutes: defaultRules?.grace_period_minutes ?? null,
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["policy-assignments"] });
    },
  });

  const handleDelete = async () => {
    if (!employeeAssignment) return;
    const ok = await confirm({
      title: t("common.delete_confirm"),
      message: t("employees.policies.delete_assign_warning") || "Remove this policy override? Employee will fall back to default rules.",
      variant: "danger",
    });
    if (ok) deleteMutation.mutate(employeeAssignment.id);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-20">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-8 animate-in fade-in duration-500" data-testid="attendance-policy-tab">
      {/* Header */}
      <div className="flex items-center justify-between px-2">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
            <ShieldCheck size={20} />
          </div>
          <div>
            <Typography className="text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">
              {t("employees.details.special_policies")}
            </Typography>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5" variant="p">
              {t("employees.policies.policy_desc") || "Attendance policy overrides for this employee"}
            </Typography>
          </div>
        </div>
        {!employeeAssignment && (
          <Button
            className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95"
            disabled={createMutation.isPending}
            onClick={() => { createMutation.mutate(); }}
            variant="ghost"
          >
            {createMutation.isPending ? <Loader2 className="animate-spin" size={14} /> : <Plus size={14} />}
            {t("common.add_new").toUpperCase()}
          </Button>
        )}
      </div>

      {/* Default rules (read-only) */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-8">
        <div className="flex items-center gap-3 mb-4">
          <Clock className="text-slate-400" size={16} />
          <Typography className="text-[10px] font-bold text-slate-500 uppercase tracking-widest" variant="h4">
            {t("common.payroll.overtime_policies") ? "Default organization rules" : "Default organization rules"}
          </Typography>
        </div>
        {defaultRules ? (
          <div className="text-sm font-mono text-slate-600 dark:text-slate-400">
            {formatRules(defaultRules)}
          </div>
        ) : (
          <div className="text-sm text-slate-400 italic">{t("common.no_data")}</div>
        )}
      </div>

      {/* Employee override */}
      <div className="space-y-4">
        <div className="flex items-center gap-3 px-2">
          <ArrowRight className="text-primary-500" size={14} />
          <Typography className="text-[10px] font-bold text-primary-600 uppercase tracking-widest" variant="h4">
            Override
          </Typography>
        </div>

        {employeeAssignment ? (
          <div className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 rounded-[2rem] p-8 hover:shadow-premium-xl transition-all group relative overflow-hidden">
            <div className="flex items-start justify-between relative z-10">
              <div>
                <div className="flex items-center gap-3 mb-2">
                  <Typography className="font-bold text-slate-700 dark:text-slate-200 text-lg tracking-tight uppercase leading-none" variant="h4">
                    {t("common.payroll.traceability.individual_policy") || "Individual Policy"}
                  </Typography>
                  <span className="px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 border-primary-100 dark:border-primary-800">
                    USER
                  </span>
                </div>
                <div className="text-sm font-mono text-slate-600 dark:text-slate-400 space-y-1 mt-3">
                  <div>Work unit: {employeeAssignment.rules.work_unit_minutes}min · Round: {employeeAssignment.rules.rounding_policy}</div>
                  {employeeAssignment.rules.grace_period_minutes != null && <div>Grace period: {employeeAssignment.rules.grace_period_minutes}m</div>}
                  {!!employeeAssignment.rules.early_arrival_strategy && <div>Early arrival: {employeeAssignment.rules.early_arrival_strategy}</div>}
                </div>
                {!!employeeAssignment.valid_from && (
                  <div className="text-[10px] text-slate-400 font-bold uppercase tracking-widest mt-3">
                    <Clock className="inline mr-1" size={10} />
                    {t("common.effective_date")}: {employeeAssignment.valid_from}
                  </div>
                )}
              </div>
              <Button
                className="p-4 rounded-2xl bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 hover:bg-rose-600 hover:text-white transition-all shadow-sm opacity-0 group-hover:opacity-100 -translate-y-2 group-hover:translate-y-0"
                disabled={deleteMutation.isPending}
                onClick={handleDelete}
                variant="ghost"
              >
                {deleteMutation.isPending ? <Loader2 className="animate-spin" size={20} /> : <Trash2 size={20} />}
              </Button>
            </div>
            <div className="absolute top-0 right-0 w-24 h-24 bg-slate-50 dark:bg-slate-800/50 -rotate-45 translate-x-12 -translate-y-12 opacity-50 transition-all group-hover:bg-primary-50 dark:group-hover:bg-primary-950/20" />
          </div>
        ) : (
          <div className="bg-slate-50 dark:bg-slate-800/50 border border-dashed border-slate-200 dark:border-slate-700 rounded-[2rem] p-16 text-center">
            <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-3xl flex items-center justify-center text-slate-300 dark:text-slate-600 mx-auto mb-6 shadow-sm border border-slate-100 dark:border-slate-800">
              <ShieldCheck size={32} />
            </div>
            <Typography className="text-sm font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="h4">
              {t("employees.policies.no_assignments") || "No policy override"}
            </Typography>
            <Typography className="text-xs text-slate-400 dark:text-slate-500 mt-2 max-w-md mx-auto">
              {t("employees.policies.no_assignments_desc") || "Employee uses default organization rules. Click \"Add New\" to set custom attendance policy."}
            </Typography>
          </div>
        )}
      </div>

      <ConfirmationDialog />
    </div>
  );
}
