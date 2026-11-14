import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, ShieldCheck } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import { fetchLeavePolicies, createLeavePolicy, deleteLeavePolicy, updateLeavePolicy, fetchLeaveTypes, type CreateLeavePolicyRequestBody, type UpdateLeavePolicyRequestBody } from "@/lib/api/leaves";
import { LeavePolicy } from "@/lib/api/types";

import { PolicyCard } from "./policy-card";
import { PolicyModal } from "./policy-modal";
import { createQueryKey } from "@/lib/query-keys";


export function LeavePolicyManager() {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [isCreating, setIsCreating] = useState(false);
  const [editingPolicy, setEditingPolicy] = useState<LeavePolicy | null>(null);
  const { confirm, ConfirmationDialog } = useConfirm();

  const { data: policies = [], isLoading } = useQuery({ queryKey: createQueryKey("leave-policies"), queryFn: () => fetchLeavePolicies(), });

  const { data: leaveTypes = [] } = useQuery({ queryKey: createQueryKey("leave-types"), queryFn: () => fetchLeaveTypes(), });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteLeavePolicy(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("leave-policies") });
      toast.success(t("settings.leave.policy_manager.success_delete", "Policy deleted successfully"));
    },
  });

  const handleDelete = async (id: string) => {
    const ok = await confirm({
      title: t("common.warning", "Warning"),
      message: t("leave.policy.confirm_delete", "Delete this policy?"),
      variant: "danger"
    });
    if (ok) deleteMutation.mutate(id);
  };

  const mutation = useMutation({
    mutationFn: (data: Partial<LeavePolicy>) =>
      editingPolicy
        ? updateLeavePolicy(editingPolicy.id, data as Record<string, unknown> as UpdateLeavePolicyRequestBody)
        : createLeavePolicy(data as Record<string, unknown> as CreateLeavePolicyRequestBody),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("leave-policies") });
      toast.success(editingPolicy ? t("settings.leave.policy_manager.success_update", "Policy updated") : t("settings.leave.policy_manager.success_create", "Policy created"));
      setIsCreating(false);
      setEditingPolicy(null);
    },
    onError: (error: Error) => { toast.error(error.message); },
  });

  if (isLoading) return <div className="p-12 text-center text-slate-400 dark:text-slate-500 animate-pulse font-bold uppercase tracking-widest text-[10px]">{t("settings.leave.policy_manager.loading", "Loading policies...")}</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-end px-1">
         <Button
           className="shadow-primary-100 text-[10px]"
           data-testid="add-policy-btn"
           onClick={() => { setIsCreating(true); }}
           size="sm"
           variant="primary"
         >
           <Plus size={14} /> {t("settings.leave.policy_manager.create_btn", "Create Policy")}
         </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {policies.length === 0 ? (
          <div className="col-span-full py-20 bg-slate-50 dark:bg-slate-800 border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-[2.5rem] flex flex-col items-center justify-center text-slate-400 dark:text-slate-500">
            <ShieldCheck className="mb-4 opacity-20" size={40} />
            <Typography className="text-xs font-bold uppercase tracking-widest" variant="p">{t("settings.leave.policy_manager.empty_title", "No policies defined yet")}</Typography>
          </div>
        ) : policies.map((policy) => (
          <PolicyCard
            bonusCount={policy.rule_config.seniority_bonus?.length ?? 0}
            key={policy.id}
            onDelete={handleDelete}
            onEdit={setEditingPolicy}
            policy={policy}
            typeName={leaveTypes.find(t => t.id === policy.leave_type_id)?.name ?? "Unknown"}
          />
        ))}
      </div>

      {!!(isCreating || editingPolicy) && (
        <PolicyModal
          isOpen={true}
          isSaving={mutation.isPending}
          leaveTypes={leaveTypes}
          onClose={() => { setIsCreating(false); setEditingPolicy(null); }}
          onSave={(data: Partial<LeavePolicy>) => { mutation.mutate(data); }}
          policy={editingPolicy}
        />
      )}
      <ConfirmationDialog />
    </div>
  );
}
