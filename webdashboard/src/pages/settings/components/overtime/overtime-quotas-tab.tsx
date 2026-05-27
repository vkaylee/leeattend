import { Loader2, Plus, Edit2, Trash2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { useWorkspaceConfigs, useDeleteWorkspaceConfig } from "@/hooks/use-workspace-config";
import { useConfirm } from "@/hooks/use-confirm";

import { UpsertConfigModal } from "./upsert-config-modal";

export function OvertimeQuotasTab({ wsId }: { wsId: string }) {
  const { t } = useTranslation();
  const { data: configs, isLoading } = useWorkspaceConfigs(wsId);
  const { mutate: deleteConfig, isPending: isDeleting } = useDeleteWorkspaceConfig(wsId);
  const { confirm } = useConfirm();

  const [upsertModalOpen, setUpsertModalOpen] = useState(false);
  const [selectedConfig, setSelectedConfig] = useState<any | null>(null);

  // Filter only overtime-related configs if we want to isolate them
  const otConfigs = configs?.filter(c => c.config_key.startsWith("overtime.")) || [];

  const handleEdit = (config: any) => {
    setSelectedConfig(config);
    setUpsertModalOpen(true);
  };

  const handleCreate = () => {
    setSelectedConfig(null);
    setUpsertModalOpen(true);
  };

  const handleDelete = async (id: string) => {
    const isConfirmed = await confirm({
      title: t("common.delete_confirm"),
      description: t("settings.overtime.delete_config_desc", { defaultValue: "Are you sure you want to delete this config? This action cannot be undone." }),
      confirmText: t("common.delete"),
      cancelText: t("common.cancel"),
      variant: "destructive"
    });

    if (isConfirmed) {
      deleteConfig(id);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center border-b border-slate-50 dark:border-slate-800 pb-6">
        <div>
          <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">
            {t("settings.overtime.tabs.quotas", { defaultValue: "Quotas & EAV Configs" })}
          </Typography>
          <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 mt-1 uppercase tracking-tight" variant="p">
            {t("settings.overtime.quotas_desc", { defaultValue: "Manage EAV properties for Overtime like monthly limits or warning enforcement." })}
          </Typography>
        </div>
        <button
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-700 text-white px-5 py-2.5 rounded-2xl font-bold text-sm tracking-tight transition-all shadow-sm"
          onClick={handleCreate}
        >
          <Plus className="w-4 h-4" />
          {t("common.add_new")}
        </button>
      </div>

      {isLoading ? (
        <div className="flex justify-center p-12">
          <Loader2 className="w-8 h-8 animate-spin text-slate-400" />
        </div>
      ) : otConfigs.length === 0 ? (
        <div className="text-center py-16 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-3xl">
          <Typography className="text-slate-500 font-medium">
            {t("common.no_data")}
          </Typography>
        </div>
      ) : (
        <div className="overflow-x-auto rounded-3xl border border-slate-100 dark:border-slate-800">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50 dark:bg-slate-900 border-b border-slate-100 dark:border-slate-800 text-xs font-bold text-slate-500 uppercase tracking-widest">
                <th className="p-4">{t("common.key", { defaultValue: "Key" })}</th>
                <th className="p-4">{t("common.scope", { defaultValue: "Scope" })}</th>
                <th className="p-4">{t("common.value", { defaultValue: "Value" })}</th>
                <th className="p-4">{t("common.effective_from")}</th>
                <th className="p-4 text-right">{t("common.actions")}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 dark:divide-slate-800 text-sm font-medium">
              {otConfigs.map((cfg) => (
                <tr key={cfg.id} className="hover:bg-slate-50 dark:hover:bg-slate-900/50 transition-colors">
                  <td className="p-4">
                    <span className="font-mono text-xs bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded-md">
                      {cfg.config_key}
                    </span>
                  </td>
                  <td className="p-4 capitalize">
                    {cfg.scope_type}
                    {cfg.scope_id && <span className="ml-2 text-xs text-slate-400">({cfg.scope_id})</span>}
                  </td>
                  <td className="p-4">
                    {cfg.value_int !== null && cfg.value_int !== undefined ? cfg.value_int : ""}
                    {cfg.value_decimal !== null && cfg.value_decimal !== undefined ? cfg.value_decimal : ""}
                    {cfg.value_text !== null && cfg.value_text !== undefined ? cfg.value_text : ""}
                    {cfg.value_bool !== null && cfg.value_bool !== undefined ? (cfg.value_bool ? "Yes" : "No") : ""}
                    {cfg.value_json !== null && cfg.value_json !== undefined ? "{JSON}" : ""}
                  </td>
                  <td className="p-4 text-slate-500">{new Date(cfg.effective_from).toLocaleDateString()}</td>
                  <td className="p-4 text-right">
                    <div className="flex justify-end gap-2">
                      <button
                        className="p-2 text-slate-400 hover:text-blue-600 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl transition-colors"
                        onClick={() => handleEdit(cfg)}
                      >
                        <Edit2 className="w-4 h-4" />
                      </button>
                      <button
                        className="p-2 text-slate-400 hover:text-rose-600 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl transition-colors"
                        disabled={isDeleting}
                        onClick={() => handleDelete(cfg.id)}
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {upsertModalOpen && (
        <UpsertConfigModal
          config={selectedConfig}
          isOpen={upsertModalOpen}
          onClose={() => setUpsertModalOpen(false)}
          wsId={wsId}
        />
      )}
    </div>
  );
}
