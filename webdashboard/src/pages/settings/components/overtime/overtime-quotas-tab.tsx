import { Loader2, Plus, Edit2, Trash2, Calendar, Key } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import ResponsiveTable from "@/components/ui/responsive-table";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import { useWorkspaceConfigs, useDeleteWorkspaceConfig } from "@/hooks/use-workspace-config";
import { type WorkspaceConfigEntry } from "@/lib/api/types";

import { UpsertConfigModal } from "./upsert-config-modal";

function formatConfigValue(cfg: WorkspaceConfigEntry): string | number {
  if (typeof cfg.value_int === "number") {
    return cfg.value_int;
  }
  if (typeof cfg.value_decimal === "number") {
    return cfg.value_decimal;
  }
  if (typeof cfg.value_text === "string") {
    return cfg.value_text;
  }
  if (typeof cfg.value_bool === "boolean") {
    return cfg.value_bool ? "Yes" : "No";
  }
  if (cfg.value_json) {
    return "{JSON}";
  }
  return "";
}

export function OvertimeQuotasTab({ wsId }: { wsId: string }) {
  const { t } = useTranslation();
  const { data: configs, isLoading } = useWorkspaceConfigs();
  const { mutate: deleteConfig, isPending: isDeleting } = useDeleteWorkspaceConfig();
  const { confirm } = useConfirm();

  const [upsertModalOpen, setUpsertModalOpen] = useState(false);
  const [selectedConfig, setSelectedConfig] = useState<WorkspaceConfigEntry | null>(null);

  // Filter only overtime-related configs if we want to isolate them
  const otConfigs = configs?.filter(c => c.config_key.startsWith("overtime.")) ?? [];

  const handleEdit = (config: WorkspaceConfigEntry) => {
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
      message: t("settings.overtime.delete_config_desc", { defaultValue: "Are you sure you want to delete this config? This action cannot be undone." }),
      confirmLabel: t("common.delete"),
      cancelLabel: t("common.cancel"),
      variant: "danger"
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
        <Button
          icon={<Plus className="w-4 h-4" />}
          onClick={handleCreate}
          variant="primary"
        >
          {t("common.add_new")}
        </Button>
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
        <ResponsiveTable<WorkspaceConfigEntry>
          data={otConfigs}
          emptyState={{
            title: t("common.no_data"),
          }}
          getCardKey={(cfg) => cfg.id}
          isLoading={false}
          renderCard={(cfg) => (
            <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
              {/* Header: Config key */}
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 shadow-inner">
                  <Key size={16} />
                </div>
                <div className="min-w-0 flex-1">
                  <span className="font-mono text-xs bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded-md">
                    {cfg.config_key}
                  </span>
                </div>
              </div>

              {/* Detail grid */}
              <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
                <div>
                  <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("common.scope", { defaultValue: "Scope" })}</div>
                  <div className="mt-1 text-xs font-medium capitalize text-slate-600 dark:text-slate-400">
                    {cfg.scope_type}
                    {!!cfg.scope_id && <span className="ml-1 text-[10px] text-slate-400">({cfg.scope_id})</span>}
                  </div>
                </div>
                <div>
                  <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("common.value", { defaultValue: "Value" })}</div>
                  <div className="mt-1 text-xs font-medium text-slate-700 dark:text-slate-300">
                    {formatConfigValue(cfg)}
                  </div>
                </div>
              </div>

              <div className="mt-3 flex items-center gap-2">
                <Calendar className="h-3 w-3 text-slate-400" />
                <span className="text-[10px] text-slate-500">{new Date(cfg.effective_from).toLocaleDateString()}</span>
              </div>

              {/* Actions */}
              <div className="mt-4 flex items-center justify-end gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
                <Button
                  onClick={() => { handleEdit(cfg); }}
                  size="icon"
                  variant="outline"
                >
                  <Edit2 className="w-4 h-4" />
                </Button>
                <Button
                  disabled={isDeleting}
                  onClick={() => handleDelete(cfg.id)}
                  size="icon"
                  variant="outline"
                >
                  <Trash2 className="w-4 h-4" />
                </Button>
              </div>
            </div>
          )}
          renderDesktopTable={() => (
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
                    <tr className="hover:bg-slate-50 dark:hover:bg-slate-900/50 transition-colors" key={cfg.id}>
                      <td className="p-4">
                        <span className="font-mono text-xs bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded-md">
                          {cfg.config_key}
                        </span>
                      </td>
                      <td className="p-4 capitalize">
                        {cfg.scope_type}
                        {!!cfg.scope_id && <span className="ml-2 text-xs text-slate-400">({cfg.scope_id})</span>}
                      </td>
                      <td className="p-4">
                        {formatConfigValue(cfg)}
                      </td>
                      <td className="p-4 text-slate-500">{new Date(cfg.effective_from).toLocaleDateString()}</td>
                      <td className="p-4 text-right">
                        <div className="flex justify-end gap-2">
                          <Button
                            onClick={() => { handleEdit(cfg); }}
                            size="icon"
                            variant="outline"
                          >
                            <Edit2 className="w-4 h-4" />
                          </Button>
                          <Button
                            disabled={isDeleting}
                            onClick={() => handleDelete(cfg.id)}
                            size="icon"
                            variant="outline"
                          >
                            <Trash2 className="w-4 h-4" />
                          </Button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        />
      )}

      {upsertModalOpen ? (
        <UpsertConfigModal
          config={selectedConfig}
          isOpen={upsertModalOpen}
          onClose={() => { setUpsertModalOpen(false); }}
          wsId={wsId}
        />
      ) : null}
    </div>
  );
}
