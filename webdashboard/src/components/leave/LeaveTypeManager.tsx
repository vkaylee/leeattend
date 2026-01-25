import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { fetchLeaveTypes, updateLeaveTypeConfig } from "@/lib/api/leaves";
import { LeaveType, LeaveTypeConfig } from "@/lib/api/types";
import { ShieldCheck, FileText, Wallet, AlertTriangle, Edit2 } from "lucide-react";
import { useToast } from "@/components/Toast";
import Modal from "@/components/ui/Modal";

interface LeaveTypeManagerProps {
  wsId: string;
}

export function LeaveTypeManager({ wsId }: LeaveTypeManagerProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [editingType, setEditingType] = useState<LeaveType | null>(null);

  const { data: leaveTypes = [], isLoading } = useQuery({
    queryKey: ["leave-types", wsId],
    queryFn: () => fetchLeaveTypes(wsId),
  });

  const updateMutation = useMutation({
    mutationFn: (data: { typeId: string; config: LeaveTypeConfig }) =>
      updateLeaveTypeConfig(wsId, data.typeId, data.config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-types"] });
      toast.success(t("settings.leave.messages.update_success", "Leave type updated successfully"));
      setEditingType(null);
    },
    onError: () => {
      toast.error(t("settings.leave.messages.update_error", "Failed to update leave type"));
    },
  });

  if (isLoading) return <div className="p-8 text-center text-primary-400">Loading leave types...</div>;

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {leaveTypes.map((type) => (
          <div
            key={type.id}
            className="relative flex flex-col justify-between rounded-xl border border-primary-200/50 bg-white p-6 shadow-sm transition-all hover:shadow-md dark:border-slate-800 dark:bg-slate-800"
          >
            <div>
              <div className="flex items-center justify-between">
                <div className={`flex h-10 w-10 items-center justify-center rounded-lg ${
                  type.is_paid ? "bg-emerald-100 text-emerald-600 dark:bg-emerald-900/30" : "bg-amber-100 text-amber-600 dark:bg-amber-900/30"
                }`}>
                  {type.is_paid ? <Wallet className="h-5 w-5" /> : <AlertTriangle className="h-5 w-5" />}
                </div>
                <button
                  onClick={() => setEditingType(type)}
                  className="rounded-full p-2 text-primary-400 hover:bg-slate-100 hover:text-primary-600 dark:hover:bg-slate-800 dark:hover:text-slate-300"
                >
                  <Edit2 className="h-4 w-4" />
                </button>
              </div>
              <h3 className="mt-4 text-lg font-bold text-primary-600 uppercase tracking-tight dark:text-white">{type.name}</h3>
              <p className="text-sm text-primary-400 dark:text-primary-400">Code: {type.code}</p>
              
              <div className="mt-4 space-y-2">
                <div className="flex items-center gap-2 text-sm text-primary-400 dark:text-primary-400">
                  <ShieldCheck className="h-4 w-4 text-primary-500" />
                  <span>{t("settings.leave.fields.approval", "Approval")}: {type.policy_config.approval_required ? t("common.required") : t("common.auto")}</span>
                </div>
                <div className="flex items-center gap-2 text-sm text-primary-400 dark:text-primary-400">
                  <FileText className="h-4 w-4 text-blue-500" />
                  <span>{t("settings.leave.fields.document", "Document")}: {type.policy_config.requires_document ? t("common.required") : t("common.optional")}</span>
                </div>
              </div>
            </div>
            
            <div className="mt-4 border-t border-primary-100/50 pt-4 dark:border-slate-800">
               <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                 type.is_paid ? "bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-400" : "bg-slate-100 text-primary-600 dark:bg-slate-800 dark:text-primary-400"
               }`}>
                 {type.is_paid ? t("settings.leave.types.paid", "Paid Leave") : t("settings.leave.types.unpaid", "Unpaid Leave")}
               </span>
            </div>
          </div>
        ))}
      </div>

      {editingType && (
        <EditConfigModal
          isOpen={!!editingType}
          onClose={() => setEditingType(null)}
          leaveType={editingType}
          onSave={(config) => updateMutation.mutate({ typeId: editingType.id, config })}
          isSaving={updateMutation.isPending}
        />
      )}
    </div>
  );
}

function EditConfigModal({
  isOpen,
  onClose,
  leaveType,
  onSave,
  isSaving,
}: {
  isOpen: boolean;
  onClose: () => void;
  leaveType: LeaveType;
  onSave: (config: LeaveTypeConfig) => void;
  isSaving: boolean;
}) {
  const [config, setConfig] = useState<LeaveTypeConfig>(leaveType.policy_config);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(config);
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={`Configure ${leaveType.name}`}>
      <form onSubmit={handleSubmit} className="space-y-6">
        <div className="space-y-4">
          <label className="flex items-start gap-3 rounded-lg border border-primary-200/50 p-4 dark:border-slate-700">
            <input
              type="checkbox"
              checked={config.approval_required}
              onChange={(e) => setConfig({ ...config, approval_required: e.target.checked })}
              className="mt-1 h-4 w-4 rounded border-primary-200 text-primary-600 focus:ring-primary-500"
            />
            <div>
              <span className="block text-sm font-medium text-primary-600 dark:text-white">Require Approval</span>
              <span className="block text-xs text-primary-400">Manager must approve requests of this type</span>
            </div>
          </label>

          <label className="flex items-start gap-3 rounded-lg border border-primary-200/50 p-4 dark:border-slate-700">
            <input
              type="checkbox"
              checked={config.requires_document}
              onChange={(e) => setConfig({ ...config, requires_document: e.target.checked })}
              className="mt-1 h-4 w-4 rounded border-primary-200 text-primary-600 focus:ring-primary-500"
            />
            <div>
              <span className="block text-sm font-medium text-primary-600 dark:text-white">Require Document</span>
              <span className="block text-xs text-primary-400">Employee must attach a file (e.g. medical certificate)</span>
            </div>
          </label>

          <label className="flex items-start gap-3 rounded-lg border border-primary-200/50 p-4 dark:border-slate-700">
            <input
              type="checkbox"
              checked={config.allow_negative_balance}
              onChange={(e) => setConfig({ ...config, allow_negative_balance: e.target.checked })}
              className="mt-1 h-4 w-4 rounded border-primary-200 text-primary-600 focus:ring-primary-500"
            />
            <div>
              <span className="block text-sm font-medium text-primary-600 dark:text-white">Allow Negative Balance</span>
              <span className="block text-xs text-primary-400">Allow taking leave even if balance is insufficient</span>
            </div>
          </label>

          {config.allow_negative_balance && (
             <div>
               <label className="block text-sm font-medium text-primary-400 dark:text-primary-400">Negative Limit (Days)</label>
               <input
                 type="number"
                 step="0.5"
                 value={config.negative_balance_limit || ""}
                 onChange={(e) => setConfig({ ...config, negative_balance_limit: parseFloat(e.target.value) })}
                 className="mt-1 block w-full rounded-md border border-primary-200 text-primary-600 px-3 py-2 text-sm text-primary-600 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800"
                 placeholder="e.g. 5.0"
               />
             </div>
          )}
        </div>

        <div className="flex justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg border border-primary-200 px-6 py-2.5 text-sm font-medium text-primary-600 hover:bg-slate-50 dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSaving}
            className="flex items-center gap-2 rounded-lg bg-primary-600 px-6 py-2.5 text-sm font-bold text-white uppercase tracking-widest text-xs hover:bg-primary-700 disabled:opacity-50"
          >
            {isSaving ? "Saving..." : "Save Changes"}
          </button>
        </div>
      </form>
    </Modal>
  );
}
