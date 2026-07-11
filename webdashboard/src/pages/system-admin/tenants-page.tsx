import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Eye, Server, Activity, ShieldAlert, KeyRound, SearchX } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import PageHeader from "@/components/ui/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import StatusBadge from "@/components/ui/status-badge";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import {
  createSystemTenant,
  deleteSystemTenant,
  fetchStaleTenants,
  fetchSystemTenants,
  fetchSystemTenantStats,
  updateTenantDatabase,
  type CreateTenantRequestPayload,
  type SystemTenant,
  type TenantConnectionStats,
} from "@/lib/api/system";


// eslint-disable-next-line max-lines-per-function, complexity -- Page component needs to encompass form and table state
export default function TenantsPage() {
  const { t } = useTranslation(["common", "system_roles"]);
  const queryClient = useQueryClient();
  const { ConfirmationDialog, confirm } = useConfirm();
  const toast = useToast();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [dbName, setDbName] = useState("");
  const [createDb, setCreateDb] = useState(true);
  const [poolSize, setPoolSize] = useState("15");

  const [statsModalOpen, setStatsModalOpen] = useState(false);
  const [dbUpdateModalOpen, setDbUpdateModalOpen] = useState(false);
  const [updateDbUrl, setUpdateDbUrl] = useState("");
  const [selectedTenant, setSelectedTenant] = useState<SystemTenant | null>(null);
  const [staleModalOpen, setStaleModalOpen] = useState(false);

  const { data: tenants = [], isLoading } = useQuery({
    queryKey: ["system-tenants"],
    queryFn: fetchSystemTenants,
  });

  const { data: tenantStats, isLoading: isStatsLoading } = useQuery<TenantConnectionStats>({
    queryKey: ["system-tenant-stats", selectedTenant?.id],
    queryFn: () => {
      if (!selectedTenant) return Promise.reject(new Error("No selected tenant"));
      return fetchSystemTenantStats(selectedTenant.id);
    },
    enabled: !!selectedTenant && (statsModalOpen || dbUpdateModalOpen),
  });

  const { data: staleTenants = [], isLoading: isStaleLoading, refetch: refetchStale } = useQuery({
    queryKey: ["system-tenants-stale"],
    queryFn: fetchStaleTenants,
    enabled: staleModalOpen,
  });

  const cleanStaleMutation = useMutation({
    mutationFn: (id: string) => deleteSystemTenant(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["system-tenants-stale"] });
      void queryClient.invalidateQueries({ queryKey: ["system-tenants"] });
      toast.success(t("system_admin.tenants.messages.clean_success"));
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  const statsConfig = tenantStats ? (tenantStats.config.data ?? tenantStats.config.tenant ?? tenantStats.config) : undefined;
  const poolUsers = statsConfig?.users ?? [];
  const rawConfigString = JSON.stringify(statsConfig, null, 2);

  const createMutation = useMutation({
    mutationFn: (data: CreateTenantRequestPayload) => createSystemTenant(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["system-tenants"] });
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
      void queryClient.invalidateQueries({ queryKey: ["system-tenants"] });
      toast.success(t("system_admin.tenants.messages.delete_success"));
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  const updateDbMutation = useMutation({
    mutationFn: ({ id, url }: { id: string; url: string }) => updateTenantDatabase(id, url),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["system-tenants"] });
      toast.success(t("system_admin.tenants.messages.update_db_success", { defaultValue: "Database URL updated successfully" }));
      setDbUpdateModalOpen(false);
      setUpdateDbUrl("");
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

  const handleManageDb = (tenant: SystemTenant) => {
    setSelectedTenant(tenant);
    setUpdateDbUrl("");
    setDbUpdateModalOpen(true);
  };

  const handleCleanStale = async (id: string) => {
    const isConfirmed = await confirm({
      title: t("common.warning", { defaultValue: "Warning: Critical Action" }),
      message: t("system_admin.tenants.stale_modal.confirm_clean"),
      variant: "danger",
    });
    if (isConfirmed) {
      cleanStaleMutation.mutate(id);
    }
  };

  const handleCleanAllStale = async () => {
    if (staleTenants.length === 0) return;
    const isConfirmed = await confirm({
      title: t("common.warning", { defaultValue: "Warning: Critical Action" }),
      message: t("system_admin.tenants.stale_modal.confirm_clean_all"),
      variant: "danger",
    });
    if (isConfirmed) {
      for (const tenant of staleTenants) {
        cleanStaleMutation.mutate(tenant.id);
      }
    }
  };

  const handleUpdateDbSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!selectedTenant || !updateDbUrl) return;
    const isConfirmed = await confirm({
      title: t("common.warning", { defaultValue: "Warning: Critical Action" }),
      message: "This will immediately update the connection pool for the tenant. The system DOES NOT copy data. Ensure the new database is fully migrated before proceeding. Continue?",
      variant: "danger",
    });
    if (isConfirmed) {
      updateDbMutation.mutate({ id: selectedTenant.id, url: updateDbUrl });
    }
  };

  const renderConnectionTemplate = (stats: TenantConnectionStats | undefined) => {
    if (!stats) return null;
    if (stats.connection.masked_db_url) {
      return stats.connection.masked_db_url;
    }
    const config = stats.config.tenant ?? stats.config;
    const dbUser = config.users?.[0]?.db_user ?? "user";
    const dbName = config.db_database ?? "db";
    const host = stats.connection.host;
    const port = stats.connection.port;
    const tenantId = selectedTenant?.id ?? "tenant_id";
    
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
        <div className="flex items-center gap-3">
          <Button onClick={() => { setStaleModalOpen(true); void refetchStale(); }} variant="outline">
            <SearchX className="w-4 h-4 mr-2" />
            {t("system_admin.tenants.check_stale_btn")}
          </Button>
          <Button onClick={() => { setIsModalOpen(true); }} variant="ghost">
            <Plus className="w-4 h-4 mr-2" />
            {t("system_admin.tenants.add_btn")}
          </Button>
        </div>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{t("system_admin.tenants.table.id")}</TableHead>
            <TableHead>{t("system_admin.tenants.table.pool_size")}</TableHead>
            <TableHead>{t("system_admin.tenants.table.created_at")}</TableHead>
            <TableHead>{t("system_admin.tenants.table.updated_at")}</TableHead>
            <TableHead className="text-right">
              {t("system_admin.tenants.table.actions")}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell className="text-center" colSpan={5}>
                <div className="flex justify-center py-4">
                  <div className="w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
                </div>
              </TableCell>
            </TableRow>
          ) : tenants.length === 0 ? (
            <TableRow>
              <TableCell className="text-center text-slate-500" colSpan={5}>
                No tenants found.
              </TableCell>
            </TableRow>
          ) : (
            tenants.map((tenant) => (
              <TableRow key={tenant.id}>
                <TableCell className="font-mono text-xs">{tenant.id}</TableCell>
                <TableCell>
                  <StatusBadge variant="neutral">{tenant.pool_size}</StatusBadge>
                </TableCell>
                <TableCell className="text-sm text-slate-500">
                  {new Date(tenant.created_at).toLocaleString()}
                </TableCell>
                <TableCell className="text-sm text-slate-500">
                  {new Date(tenant.updated_at).toLocaleString()}
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex items-center justify-end gap-2">
                    <Button
                      onClick={() => { handleViewStats(tenant); }}
                      size="sm"
                      title="View Connection & Stats"
                      variant="ghost"
                    >
                      <Eye className="w-4 h-4 text-slate-500" />
                    </Button>
                    <Button
                      onClick={() => { handleManageDb(tenant); }}
                      size="sm"
                      title="Manage Database"
                      variant="ghost"
                    >
                      <Server className="w-4 h-4 text-orange-500" />
                    </Button>

                    <Button
                      onClick={() => { void handleDelete(tenant.id); }}
                      size="sm"
                      variant="ghost"
                    >
                      <Trash2 className="w-4 h-4 text-red-500" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>

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
        maxWidth="2xl"
        onClose={() => { setStatsModalOpen(false); setSelectedTenant(null); }}
        subtitle="Connection details and Supavisor configuration"
        title={`Tenant Details: ${selectedTenant?.id ?? ""}`}
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
                  <Typography className="text-sm font-semibold text-slate-800 dark:text-slate-200" variant="h4">Connection String</Typography>
                </div>
                <div className="bg-slate-900 rounded-lg p-3 overflow-x-auto">
                  <code className="text-xs text-green-400 whitespace-nowrap">
                    {renderConnectionTemplate(tenantStats)}
                  </code>
                </div>
                <Typography className="text-xs text-slate-500 mt-2" variant="p">
                  * Password is masked for security reasons. Replace *** with the actual tenant database password.
                </Typography>
              </div>

              {/* Users Config */}
              <div className="space-y-3">
                <Typography className="text-sm font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2" variant="h4">
                  <Activity className="w-4 h-4 text-blue-500" />
                  Database Users & Pool Modes
                </Typography>
                {poolUsers.length > 0 ? (
                  <div className="grid grid-cols-1 gap-3">
                    {poolUsers.map((u: { db_user?: string; pool_size?: number }, idx: number) => (
                      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-4 rounded-xl shadow-sm flex items-center justify-between" key={idx}>
                        <div>
                          <div className="font-mono text-sm text-slate-800 dark:text-slate-200 font-semibold flex items-center gap-2">
                            <KeyRound className="w-4 h-4 text-slate-400" />
                            {u.db_user}
                          </div>
                          <div className="text-xs text-slate-500 mt-1">
                            Pool size: <span className="font-medium text-slate-700 dark:text-slate-300">{u.pool_size ?? "N/A"}</span>
                          </div>
                        </div>
                        <div className="flex flex-col items-end">
                          <span className="px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300">
                            Active
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700 text-sm text-slate-500 text-center">
                    No explicit users configured for this tenant.
                  </div>
                )}
              </div>

              {/* Raw Details */}
              <div className="pt-4 border-t border-slate-200 dark:border-slate-800">
                <Typography className="text-sm font-semibold text-slate-800 dark:text-slate-200 mb-3 flex items-center gap-2" variant="h4">
                  <ShieldAlert className="w-4 h-4 text-orange-500" />
                  Raw Configuration
                </Typography>
                <div className="bg-slate-50 dark:bg-slate-900 rounded-lg p-3 border border-slate-200 dark:border-slate-800 max-h-48 overflow-y-auto">
                  <pre className="text-xs text-slate-600 dark:text-slate-400">
                    {rawConfigString}
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

      {/* Update Database Modal */}
      <Modal
        isOpen={dbUpdateModalOpen}
        maxWidth="2xl"
        onClose={() => { setDbUpdateModalOpen(false); setSelectedTenant(null); }}
        subtitle="Update the custom BYOD database URL for this tenant."
        title={`Update Database: ${selectedTenant?.id ?? ""}`}
      >
        <form onSubmit={(e) => { void handleUpdateDbSubmit(e); }}>
          <div className="grid gap-6 py-4">
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 flex gap-3">
              <ShieldAlert className="w-5 h-5 text-red-600 dark:text-red-400 shrink-0" />
              <div className="text-sm text-red-800 dark:text-red-200">
                <Typography className="font-semibold mb-1" variant="p">Critical Warning</Typography>
                <Typography variant="p">
                  This operation only updates the connection string in Supavisor and the system database. 
                  <strong> It DOES NOT copy or migrate data.</strong> You must ensure the target database 
                  already contains the necessary schema and data for this tenant.
                </Typography>
              </div>
            </div>

            <div className="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700">
              <div className="flex items-center gap-2 mb-3">
                <KeyRound className="w-4 h-4 text-primary-500" />
                <Typography className="text-sm font-semibold text-slate-800 dark:text-slate-200" variant="h4">Current Connection URL</Typography>
              </div>
              <div className="bg-slate-900 rounded-lg p-3 overflow-x-auto min-h-[44px] flex items-center">
                {isStatsLoading ? (
                  <Skeleton className="h-4 w-64 bg-slate-800" />
                ) : (
                  <code className="text-xs text-green-400 whitespace-nowrap">
                    {renderConnectionTemplate(tenantStats) ?? "Not configured"}
                  </code>
                )}
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="updateDbUrl">
                New Custom Database URL <span className="text-red-500">*</span>
              </label>
              <input
                className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent dark:text-white"
                id="updateDbUrl"
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => { setUpdateDbUrl(e.target.value); }}
                placeholder="postgres://user:password@host:5432/db"
                required
                type="text"
                value={updateDbUrl}
              />
            </div>
          </div>
          <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800 mt-2">
            <Button
              onClick={() => { setDbUpdateModalOpen(false); }}
              type="button"
              variant="outline"
            >
              Cancel
            </Button>
            <Button
              disabled={updateDbMutation.isPending || !updateDbUrl}
              loading={updateDbMutation.isPending}
              type="submit"
              variant="destructive"
            >
              {updateDbMutation.isPending ? "Updating..." : "Update Database"}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Stale Tenants Modal */}
      <Modal
        isOpen={staleModalOpen}
        maxWidth="2xl"
        onClose={() => { setStaleModalOpen(false); }}
        subtitle={t("system_admin.tenants.stale_modal.subtitle")}
        title={t("system_admin.tenants.stale_modal.title")}
      >
        <div className="py-4 space-y-4">
          {isStaleLoading ? (
            <div className="space-y-3">
              <div className="h-12 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-lg"></div>
              <div className="h-12 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-lg"></div>
              <div className="h-12 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-lg"></div>
              <Typography className="text-xs text-center text-slate-400" variant="p">{t("system_admin.tenants.stale_modal.scanning")}</Typography>
            </div>
          ) : staleTenants.length === 0 ? (
            <div className="text-center py-10">
              <div className="w-16 h-16 rounded-full bg-emerald-50 dark:bg-emerald-900/20 flex items-center justify-center mx-auto mb-4">
                <SearchX className="w-8 h-8 text-emerald-500" />
              </div>
              <Typography className="text-slate-600 dark:text-slate-300 font-medium" variant="p">
                {t("system_admin.tenants.stale_modal.no_stale")}
              </Typography>
            </div>
          ) : (
            <>
              <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-3 flex gap-3 items-center">
                <ShieldAlert className="w-5 h-5 text-amber-600 dark:text-amber-400 shrink-0" />
                <Typography className="text-sm text-amber-800 dark:text-amber-200" variant="p">
                  {staleTenants.length} stale configuration{staleTenants.length > 1 ? "s" : ""} detected.
                </Typography>
              </div>
              <div className="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
                {staleTenants.map((tenant) => (
                  <div
                    className="flex items-center justify-between px-4 py-3 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
                    key={tenant.id}
                  >
                    <div>
                      <Typography className="font-mono text-xs text-slate-700 dark:text-slate-300" variant="p">{tenant.id}</Typography>
                      <Typography className="text-xs text-slate-400 mt-0.5" variant="p">
                        Pool: {tenant.pool_size} &middot; Created: {new Date(tenant.created_at).toLocaleDateString()}
                      </Typography>
                    </div>
                    <Button
                      disabled={cleanStaleMutation.isPending}
                      onClick={() => { void handleCleanStale(tenant.id); }}
                      size="sm"
                      variant="destructive"
                    >
                      <Trash2 className="w-3.5 h-3.5 mr-1.5" />
                      Delete
                    </Button>
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
        <div className="flex justify-between pt-4 border-t border-slate-100 dark:border-slate-800">
          <div>
            {staleTenants.length > 1 && (
              <Button
                disabled={cleanStaleMutation.isPending}
                onClick={() => { void handleCleanAllStale(); }}
                variant="destructive"
              >
                {t("system_admin.tenants.stale_modal.clean_all")}
              </Button>
            )}
          </div>
          <Button onClick={() => { setStaleModalOpen(false); }} variant="outline">
            Close
          </Button>
        </div>
      </Modal>

      <ConfirmationDialog />
    </div>
  );
}
