import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { fetchLeavePolicies, createLeavePolicy, deleteLeavePolicy, fetchLeaveTypes } from "@/lib/api/leaves";
import { LeavePolicy, LeaveType } from "@/lib/api/types";
import { Plus, Trash2, CalendarDays, Clock, Users, Building2 } from "lucide-react";
import { useToast } from "@/components/Toast";
import Modal from "@/components/ui/Modal";
import { Combobox } from "@/components/ui/Combobox";

interface LeavePolicyManagerProps {
  wsId: string;
}

export function LeavePolicyManager({ wsId }: LeavePolicyManagerProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [isCreating, setIsCreating] = useState(false);

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
      queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      toast.success("Policy deleted successfully");
    },
  });

  const createMutation = useMutation({
    mutationFn: (data: Partial<LeavePolicy>) => createLeavePolicy(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      toast.success("Policy created successfully");
      setIsCreating(false);
    },
  });

  if (isLoading) return <div className="p-8 text-center text-primary-400">Loading policies...</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-end">
        <button
          onClick={() => setIsCreating(true)}
          className="flex items-center gap-2 rounded-lg bg-primary-600 px-6 py-2.5 text-sm font-bold text-white uppercase tracking-widest text-xs hover:bg-primary-700"
        >
          <Plus className="h-4 w-4" />
          {t("common.create")}
        </button>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {policies.map((policy) => {
          const type = leaveTypes.find(t => t.id === policy.leave_type_id);
          return (
            <div
              key={policy.id}
              className="relative flex flex-col justify-between rounded-xl border border-primary-200/50 bg-white p-6 shadow-sm transition-all hover:shadow-md dark:border-slate-800 dark:bg-slate-800"
            >
              <div>
                <div className="flex items-center justify-between">
                  <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 ring-1 ring-inset ring-blue-700/10 dark:bg-blue-400/10 dark:text-blue-400 dark:ring-blue-400/30">
                    {type?.name || t("common.unknown")}
                  </span>
                  <button
                    onClick={() => {
                        if (confirm("Are you sure you want to delete this policy?")) {
                            deleteMutation.mutate(policy.id);
                        }
                    }}
                    className="rounded-full p-2 text-primary-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20 dark:hover:text-red-400"
                  >
                    <Trash2 className="h-4 w-4" />
                  </button>
                </div>
                <h3 className="mt-4 text-lg font-bold text-primary-600 tracking-tight">{policy.name}</h3>
                <p className="text-sm text-primary-400 dark:text-primary-400">{policy.description || t("common.no_description")}</p>
                
                <div className="mt-4 space-y-2 text-sm text-primary-400 dark:text-primary-400">
                  <div className="flex items-center gap-2">
                    <Clock className="h-4 w-4 text-primary-400" />
                    <span className="capitalize">{policy.rule_config.frequency} Accrual</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <CalendarDays className="h-4 w-4 text-primary-400" />
                    <span>{policy.rule_config.accrual_amount} {t("settings.leave.fields.days_per_period", "days / period")}</span>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {isCreating && (
        <CreatePolicyModal
          isOpen={isCreating}
          onClose={() => setIsCreating(false)}
          leaveTypes={leaveTypes}
          onSave={(data) => createMutation.mutate(data)}
          isSaving={createMutation.isPending}
        />
      )}
    </div>
  );
}

function CreatePolicyModal({ isOpen, onClose, leaveTypes, onSave, isSaving }: { isOpen: boolean; onClose: () => void; leaveTypes: LeaveType[]; onSave: (data: Partial<LeavePolicy>) => void; isSaving: boolean; }) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [typeId, setTypeId] = useState("");
  const [amount, setAmount] = useState(1.0);
  const [frequency, setFrequency] = useState("monthly");
  const [timing, setTiming] = useState("start_of_period");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave({
      name,
      description,
      leave_type_id: typeId,
      rule_config: {
        frequency: frequency as any,
        accrual_amount: amount,
        accrual_timing: timing as any,
        seniority_bonus: [],
      },
    });
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Create Leave Policy">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-primary-400 dark:text-primary-400">Policy Name</label>
          <input
            type="text"
            required
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="mt-1 block w-full rounded-md border border-primary-200 text-primary-700 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800"
            placeholder="e.g. Standard Vacation"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-primary-400 dark:text-primary-400">Leave Type</label>
          <Combobox
            options={leaveTypes.map(t => ({ value: t.id, label: t.name }))}
            value={typeId}
            onChange={setTypeId}
            placeholder="Select leave type..."
            className="w-full"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-primary-400 dark:text-primary-400">Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="mt-1 block w-full rounded-md border border-primary-200 text-primary-700 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800"
            rows={2}
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-primary-400 dark:text-primary-400">Accrual Amount</label>
            <input
              type="number"
              step="0.1"
              required
              value={amount}
              onChange={(e) => setAmount(parseFloat(e.target.value))}
              className="mt-1 block w-full rounded-md border border-primary-200 text-primary-700 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-primary-400 dark:text-primary-400">Frequency</label>
            <select
              value={frequency}
              onChange={(e) => setFrequency(e.target.value)}
              className="mt-1 block w-full rounded-md border border-primary-200 text-primary-700 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800"
            >
              <option value="monthly">Monthly</option>
              <option value="quarterly">Quarterly</option>
              <option value="yearly">Yearly</option>
            </select>
          </div>
        </div>

        <div>
            <label className="block text-sm font-medium text-primary-400 dark:text-primary-400">Timing</label>
            <select
              value={timing}
              onChange={(e) => setTiming(e.target.value)}
              className="mt-1 block w-full rounded-md border border-primary-200 text-primary-700 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800"
            >
              <option value="start_of_period">Start of Period (e.g. 1st of month)</option>
              <option value="end_of_period">End of Period (e.g. last day)</option>
            </select>
        </div>

        <div className="flex justify-end gap-3 pt-4">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg border border-primary-200 px-6 py-2.5 text-sm font-medium text-primary-700 hover:bg-slate-50 dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSaving}
            className="flex items-center gap-2 rounded-lg bg-primary-600 px-6 py-2.5 text-sm font-bold text-white uppercase tracking-widest text-xs hover:bg-primary-700 disabled:opacity-50"
          >
            {isSaving ? "Creating..." : t("common.create")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
