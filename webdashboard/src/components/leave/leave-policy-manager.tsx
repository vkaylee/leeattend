import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, ShieldCheck } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import { fetchLeavePolicies, createLeavePolicy, deleteLeavePolicy, updateLeavePolicy, fetchLeaveTypes } from "@/lib/api/leaves";
import { LeavePolicy } from "@/lib/api/types";

import { PolicyCard } from "./policy-card";
import { PolicyModal } from "./policy-modal";

interface LeavePolicyManagerProps {
  wsId: string;
}

export function LeavePolicyManager({ wsId }: LeavePolicyManagerProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [isCreating, setIsCreating] = useState(false);
  const [editingPolicy, setEditingPolicy] = useState<LeavePolicy | null>(null);
  const { confirm, ConfirmationDialog } = useConfirm();

  const { data: policies = [], isLoading } = useQuery({
    queryKey: ["leave-policies", wsId],
    queryFn: () => fetchLeavePolicies(wsId),
  });

  const { data: leaveTypes = [] } = useQuery({
    queryKey: ["leave-types", wsId],
    queryFn: () => fetchLeaveTypes(wsId),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteLeavePolicy(wsId, id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      toast.success("Policy deleted successfully");
    },
  });

  const handleDelete = async (id: string) => {
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Warning" }),
      message: t("leave.policy.confirm_delete", { defaultValue: "Delete this policy?" }),
      variant: "danger"
    });
    if (ok) deleteMutation.mutate(id);
  };

  const mutation = useMutation({
    mutationFn: (data: Partial<LeavePolicy>) => 
      editingPolicy ? updateLeavePolicy(wsId, editingPolicy.id, data) : createLeavePolicy(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      toast.success(editingPolicy ? "Policy updated" : "Policy created");
      setIsCreating(false);
      setEditingPolicy(null);
    },
    onError: (error: Error) => { toast.error(error.message); },
  });

  if (isLoading) return <div className="p-12 text-center text-slate-400 dark:text-slate-500 animate-pulse font-bold uppercase tracking-widest text-[10px]">Loading policies...</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-end px-1">
         <button className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100" data-testid="add-policy-btn" onClick={() => { setIsCreating(true); }}><Plus size={14} /> Create Policy</button>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {policies.length === 0 ? (
          <div className="col-span-full py-20 bg-slate-50 dark:bg-slate-800 border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-[2.5rem] flex flex-col items-center justify-center text-slate-400 dark:text-slate-500">
            <ShieldCheck className="mb-4 opacity-20" size={40} />
            <Typography className="text-xs font-bold uppercase tracking-widest" variant="p">No policies defined yet</Typography>
          </div>
        ) : policies.map((policy) => (
          <PolicyCard 
            bonusCount={policy.rule_config.seniority_bonus.length}
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
