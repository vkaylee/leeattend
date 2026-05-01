import React, { useState } from "react";

import { Shield, Plus, Users } from "lucide-react";
import { useTranslation } from "react-i18next";

import { PayrollRuleAssignment } from "@/lib/api/types";

import { AssignmentFormModal } from "./rule-assignment/AssignmentFormModal";
import { AssignmentList } from "./rule-assignment/AssignmentList";
import { useRuleAssignments } from "./rule-assignment/useRuleAssignments";

interface Props {
  wsId: string;
  ruleId: string;
  calculationMethod?: "percentage" | "fixed" | "tiered";
}

export default function RuleAssignmentManager({
  wsId,
  ruleId,
  calculationMethod,
}: Props) {
  const { t } = useTranslation();
  const [isModalOpen, setIsModalOpen] = useState(false);

  const {
    isLoading,
    departments,
    deptMap,
    grouped,
    totalCount,
    createMutation,
    deleteMutation,
  } = useRuleAssignments(wsId, ruleId);

  const handleCreate = (data: Partial<PayrollRuleAssignment>) => {
    createMutation.mutate(data, {
      onSuccess: () => setIsModalOpen(false),
    });
  };

  return (
    <div className="space-y-5 mt-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-teal-50 dark:bg-teal-950/40 text-teal-600 dark:text-teal-400 flex items-center justify-center shadow-sm border border-teal-100 dark:border-teal-800">
            <Shield size={16} />
          </div>
          <div>
            <h3 className="font-bold text-sm uppercase tracking-widest text-slate-700 dark:text-slate-200 leading-none">
              {t("payroll_rules.rule_assignments", "Rule Assignments")}
            </h3>
            <p className="text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70">
              {totalCount} active{" "}
              {totalCount === 1 ? "assignment" : "assignments"} across all scopes
            </p>
          </div>
        </div>
        <button
          onClick={() => setIsModalOpen(true)}
          className="flex items-center gap-2 bg-teal-600 hover:bg-teal-700 text-white px-5 py-2.5 rounded-xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-lg shadow-teal-100 dark:shadow-black/30 active:scale-95"
        >
          <Plus size={14} /> {t("common.assign", "Assign")}
        </button>
      </div>

      {/* Loading state */}
      {isLoading && (
        <div className="p-12 flex justify-center">
          <div className="w-8 h-8 border-[3px] border-teal-600 border-t-transparent rounded-full animate-spin" />
        </div>
      )}

      {/* Empty state */}
      {!isLoading && totalCount === 0 && (
        <div className="p-12 bg-slate-50 dark:bg-slate-800/30 rounded-2xl border-2 border-dashed border-slate-100 dark:border-slate-800 flex flex-col items-center gap-3">
          <div className="w-14 h-14 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center shadow-sm border border-slate-100 dark:border-slate-800">
            <Users size={24} className="text-slate-200 dark:text-slate-700" />
          </div>
          <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
            {t(
              "payroll.no_assignments_yet",
              "No assignments yet — click 'Assign' to get started"
            )}
          </p>
        </div>
      )}

      {/* Assignment cards by scope */}
      {!isLoading && totalCount > 0 && (
        <AssignmentList
          grouped={grouped}
          deptMap={deptMap}
          onDelete={(id) => deleteMutation.mutate(id)}
          isDeleting={deleteMutation.isPending}
        />
      )}

      {/* Create Assignment Modal */}
      <AssignmentFormModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSubmit={handleCreate}
        isSubmitting={createMutation.isPending}
        wsId={wsId}
        departments={departments}
        calculationMethod={calculationMethod}
      />
    </div>
  );
}
