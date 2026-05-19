import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Eye, Server, Activity, ShieldAlert, KeyRound } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import PageHeader from "@/components/ui/page-header";
import StatusBadge from "@/components/ui/status-badge";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import {
  createSystemTenant,
  deleteSystemTenant,
  fetchSystemTenants,
  fetchSystemTenantStats,
  type CreateTenantRequestPayload,
  type SystemTenant,
} from "@/lib/api/system";

// eslint-disable-next-line max-lines-per-function -- Page component needs to encompass form and table state
export default function TenantsPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { ConfirmationDialog, confirm } = useConfirm();
  const toast = useToast();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [dbName, setDbName] = useState("");
  const [createDb, setCreateDb] = useState(true);
  const [poolSize, setPoolSize] = useState("15");

  const [statsModalOpen, setStatsModalOpen] = useState(false);
  const [selectedTenant, setSelectedTenant] = useState<SystemTenant | null>(null);

  const { data: tenants = [], isLoading } = useQuery({
    queryKey: ["system-tenants"],
    queryFn: fetchSystemTenants,
  });

  const { data: tenantStats, isLoading: isStatsLoading } = useQuery({
    queryKey: ["system-tenant-stats", selectedTenant?.id],
    queryFn: () => fetchSystemTenantStats(selectedTenant!.id),
    enabled: !!selectedTenant && statsModalOpen,
  });

  const createMutation = useMutation({
    mutationFn: (data: CreateTenantRequestPayload) => createSystemTenant(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["system-tenants"] }).catch(console.error);
      toast.success(t("system_admin.tenants.messages.create_success"));
      setIsModalOpen(false);
      setDbName("");
      setCreateDb(true);
      setPoolSize("15");
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteSystemTenant(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["system-tenants"] }).catch(console.error);
      toast.success(t("system_admin.tenants.messages.delete_success"));
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  const handleCreate = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!dbName) return;
    createMutation.mutate({
      db_name: dbName,
      create_db: createDb,
      pool_size: poolSize ? parseInt(poolSize, 10) : undefined,
    });
  };

  const handleDelete = async (id: string) => {
    const isConfirmed = await confirm({
      title: t("common.delete_confirm"),
      message: t("system_admin.tenants.confirm_delete"),
      variant: "danger",
    });
    if (isConfirmed) {
      deleteMutation.mutate(id);
    }
  };

  const handleViewStats = (tenant: SystemTenant) => {
    setSelectedTenant(tenant);
    setStatsModalOpen(true);
  };

  const renderConnectionTemplate = (tenantStats: any) => {
    if (!tenantStats || !tenantStats.config) return null;
    const config = tenantStats.config.tenant || tenantStats.config;
    const dbUser = config.users?.[0]?.db_user || "user";
    const dbName = config.db_database || "db";
    const host = tenantStats.connection?.host || "localhost";
    const port = tenantStats.connection?.port || "6543";
    const tenantId = selectedTenant?.id || "tenant_id";
    
    // Masked connection string (Rule 2: Never log PII/Passwords)
    return `postgres://${dbUser}.${tenantId}:***@${host}:${port}/${dbName}`;
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          subtitle={t("system_admin.tenants.subtitle")}
          title={t("system_admin.tenants.title")}
        />
        <Button onClick={() => { setIsModalOpen(true); }}>
          <Plus className="w-4 h-4 mr-2" />
          {t("system_admin.tenants.add_btn")}
        </Button>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm text-slate-600 dark:text-slate-400">
            <thead className="bg-slate-50 dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700 text-xs uppercase font-semibold text-slate-500 dark:text-slate-400">
              <tr>
                <th className="px-6 py-4">{t("system_admin.tenants.table.id")}</th>
                <th className="px-6 py-4">{t("system_admin.tenants.table.pool_size")}</th>
                <th className="px-6 py-4">{t("system_admin.tenants.table.created_at")}</th>
                <th className="px-6 py-4">{t("system_admin.tenants.table.updated_at")}</th>
                <th className="px-6 py-4 text-right">
                  {t("system_admin.tenants.table.actions")}
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200 dark:divide-slate-800">
              {isLoading ? (
                <tr>
                  <td className="px-6 py-8 text-center" colSpan={5}>
                    <div className="flex justify-center">
                      <div className="w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
                    </div>
                  </td>
                </tr>
              ) : tenants.length === 0 ? (
                <tr>
                  <td className="px-6 py-8 text-center text-slate-500" colSpan={5}>
                    No tenants found.
                  </td>
                </tr>
              ) : (
                tenants.map((tenant) => (
                  <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors" key={tenant.id}>
                    <td className="px-6 py-4 font-mono text-xs">{tenant.id}</td>
                    <td className="px-6 py-4">
                      <StatusBadge variant="neutral">{tenant.pool_size}</StatusBadge>
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-500">
                      {new Date(tenant.created_at).toLocaleString()}
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-500">
                      {new Date(tenant.updated_at).toLocaleString()}
                    </td>
                    <td className="px-6 py-4 text-right">
                      <div className="flex items-center justify-end gap-2">
                        <Button
                          onClick={() => { handleViewStats(tenant); }}
                          size="sm"
                          variant="ghost"
                          title="View Connection & Stats"
                        >
                          <Eye className="w-4 h-4 text-slate-500" />
                        </Button>
                        <Button
                          onClick={() => { handleDelete(tenant.id).catch(console.error); }}
                          size="sm"
                          variant="ghost"
                        >
                          <Trash2 className="w-4 h-4 text-red-500" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      <Modal
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); }}
        subtitle="Provide database credentials to initialize a new connection pool for Supavisor."
        title={t("system_admin.tenants.modal.create_title")}
      >
        <form onSubmit={handleCreate}>
          <div className="grid gap-6 py-4">
            <div className="space-y-2">
              <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="dbName">
                {t("system_admin.tenants.modal.db_name_label")} <span className="text-red-500">*</span>
              </label>
              <input
                className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent dark:text-white"
                id="dbName"
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => { setDbName(e.target.value); }}
                placeholder={t("system_admin.tenants.modal.db_name_placeholder")}
                required
                value={dbName}
              />
            </div>
            <div className="flex items-center justify-between bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg border border-slate-200 dark:border-slate-700">
              <label className="text-sm font-medium text-slate-700 dark:text-slate-300 cursor-pointer select-none" htmlFor="createDb">
                {t("system_admin.tenants.modal.create_db_label")}
              </label>
              <input
                checked={createDb}
                className="h-5 w-5 rounded border-slate-300 text-primary-600 focus:ring-primary-600 bg-white"
                id="createDb"
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => { setCreateDb(e.target.checked); }}
                type="checkbox"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="poolSize">
                {t("system_admin.tenants.modal.pool_size_label")}
              </label>
              <input
                className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent dark:text-white"
                id="poolSize"
                min="1"
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => { setPoolSize(e.target.value); }}
                type="number"
                value={poolSize}
              />
            </div>
          </div>
          <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800 mt-2">
            <Button
              onClick={() => { setIsModalOpen(false); }}
              type="button"
              variant="outline"
            >
              {t("common.cancel", { defaultValue: "Cancel" })}
            </Button>
            <Button
              disabled={createMutation.isPending || !dbName}
              loading={createMutation.isPending}
              type="submit"
              variant="primary"
            >
              {createMutation.isPending
                ? t("system_admin.tenants.modal.creating")
                : t("system_admin.tenants.modal.submit_btn")}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Stats Modal */}
      <Modal
        isOpen={statsModalOpen}
        onClose={() => { setStatsModalOpen(false); setSelectedTenant(null); }}
        title={`Tenant Details: ${selectedTenant?.id || ""}`}
        subtitle="Connection details and Supavisor configuration"
      >
        <div className="py-4 space-y-6">
          {isStatsLoading ? (
            <div className="space-y-4">
              <div className="h-24 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-xl"></div>
              <div className="grid grid-cols-2 gap-4">
                <div className="h-32 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-xl"></div>
                <div className="h-32 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-xl"></div>
              </div>
            </div>
          ) : tenantStats ? (
            <>
              {/* Connection String Card */}
              <div className="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700">
                <div className="flex items-center gap-2 mb-3">
                  <KeyRound className="w-4 h-4 text-primary-500" />
                  <h4 className="text-sm font-semibold text-slate-800 dark:text-slate-200">Connection String</h4>
                </div>
                <div className="bg-slate-900 rounded-lg p-3 overflow-x-auto">
                  <code className="text-xs text-green-400 whitespace-nowrap">
                    {renderConnectionTemplate(tenantStats)}
                  </code>
                </div>
                <p className="text-xs text-slate-500 mt-2">
                  * Password is masked for security reasons. Replace *** with the actual tenant database password.
                </p>
              </div>

              {/* Stats Cards */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-4 rounded-xl shadow-sm flex flex-col gap-1">
                  <div className="flex items-center gap-2 text-slate-500 mb-2">
                    <Server className="w-4 h-4" />
                    <span className="text-xs font-medium uppercase tracking-wider">Pool Config</span>
                  </div>
                  <div className="text-2xl font-bold text-slate-800 dark:text-white">
                    {tenantStats.config?.tenant?.users?.[0]?.pool_size || "N/A"}
                  </div>
                  <span className="text-xs text-slate-500">Max connections</span>
                </div>
                
                <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-4 rounded-xl shadow-sm flex flex-col gap-1">
                  <div className="flex items-center gap-2 text-slate-500 mb-2">
                    <Activity className="w-4 h-4" />
                    <span className="text-xs font-medium uppercase tracking-wider">Mode</span>
                  </div>
                  <div className="text-xl font-bold text-slate-800 dark:text-white capitalize">
                    {tenantStats.config?.tenant?.users?.[0]?.mode_type || "Transaction"}
                  </div>
                  <span className="text-xs text-slate-500">Pooling strategy</span>
                </div>
              </div>

              {/* Raw Details */}
              <div className="pt-4 border-t border-slate-200 dark:border-slate-800">
                <h4 className="text-sm font-semibold text-slate-800 dark:text-slate-200 mb-3 flex items-center gap-2">
                  <ShieldAlert className="w-4 h-4 text-orange-500" />
                  Raw Configuration
                </h4>
                <div className="bg-slate-50 dark:bg-slate-900 rounded-lg p-3 border border-slate-200 dark:border-slate-800 max-h-48 overflow-y-auto">
                  <pre className="text-xs text-slate-600 dark:text-slate-400">
                    {JSON.stringify(tenantStats.config?.tenant || tenantStats.config, null, 2)}
                  </pre>
                </div>
              </div>
            </>
          ) : (
            <div className="text-center py-8 text-slate-500">
              Failed to load tenant details.
            </div>
          )}
        </div>
        <div className="flex justify-end pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={() => { setStatsModalOpen(false); setSelectedTenant(null); }} variant="outline">
            Close
          </Button>
        </div>
      </Modal>

      <ConfirmationDialog />
    </div>
  );
}
