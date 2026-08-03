import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Eye, Server, Globe, Clock, Database } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";
import ResponsiveTable from "@/components/ui/responsive-table";
import StatusBadge from "@/components/ui/status-badge";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import {
  deleteSystemTenant,
  fetchSystemTenants,
  fetchSystemTenantStats,
  updateTenantDatabase,
  type SystemTenant,
  type TenantConnectionStats,
} from "@/lib/api/system";

import { CreateTenantModal } from "./components/create-tenant-modal";
import { TenantDbUpdateModal } from "./components/tenant-db-update-modal";
import { TenantStatsModal } from "./components/tenant-stats-modal";

  // eslint-disable-next-line max-lines-per-function
export default function TenantsPage() {
  const { t } = useTranslation(["common", "system_roles"]);
  const queryClient = useQueryClient();
  const { ConfirmationDialog, confirm } = useConfirm();
  const toast = useToast();
  const { formatDate, formatDateTimeLocale } = useDateTimeFormatter();

  // ── Modal visibility ──────────────────────────────────────────────────────
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [statsModalOpen, setStatsModalOpen] = useState(false);
  const [dbUpdateModalOpen, setDbUpdateModalOpen] = useState(false);
  const [selectedTenant, setSelectedTenant] = useState<SystemTenant | null>(null);

  // ── Queries ───────────────────────────────────────────────────────────────
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

  // ── Mutations ─────────────────────────────────────────────────────────────
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
      toast.success(
        t("system_admin.tenants.messages.update_db_success", {
          defaultValue: "Database URL updated successfully",
        }),
      );
      setDbUpdateModalOpen(false);
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  // ── Handlers ──────────────────────────────────────────────────────────────
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
    setDbUpdateModalOpen(true);
  };

  const handleUpdateDbSubmit = async (id: string, url: string) => {
    const isConfirmed = await confirm({
      title: t("common.warning", { defaultValue: "Warning: Critical Action" }),
      message:
        "This will immediately update the connection pool for the tenant. The system DOES NOT copy data. Ensure the new database is fully migrated before proceeding. Continue?",
      variant: "danger",
    });
    if (isConfirmed) {
      updateDbMutation.mutate({ id, url });
    }
  };

  // ── Render ────────────────────────────────────────────────────────────────
  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          subtitle={t("system_admin.tenants.subtitle")}
          title={t("system_admin.tenants.title")}
        />
        <div className="flex items-center gap-3">
          <Button
            onClick={() => {
              setCreateModalOpen(true);
            }}
            variant="ghost"
          >
            <Plus className="w-4 h-4 mr-2" />
            {t("system_admin.tenants.add_btn")}
          </Button>
        </div>
      </div>

      {/* Table */}
      <ResponsiveTable<SystemTenant>
        data={tenants}
        emptyState={{
          icon: <Database className="h-10 w-10" />,
          title: t("system_admin.tenants.table.empty", "No tenants found."),
        }}
        getCardKey={(t) => t.id}
        isLoading={isLoading}
        renderCard={(tenant) => (
          <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
            {/* Header: Name + Type */}
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0 flex-1">
                <div className="truncate text-sm font-bold text-slate-800 dark:text-slate-100">
                  {tenant.name}
                </div>
                <div className="mt-0.5 font-mono text-[10px] text-slate-400 dark:text-slate-500">
                  {tenant.id}
                </div>
              </div>
              <StatusBadge variant={tenant.is_byod ? "success" : "neutral"}>
                {tenant.is_byod ? "BYOD" : "Standard"}
              </StatusBadge>
            </div>

            {/* Details grid */}
            <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
              <div className="flex items-center gap-2">
                <Globe className="h-3.5 w-3.5 text-slate-400" />
                <span className="text-xs font-medium text-slate-600 dark:text-slate-400">{tenant.timezone}</span>
              </div>
              <div className="flex items-center gap-2">
                <Clock className="h-3.5 w-3.5 text-slate-400" />
                <span className="text-xs font-medium text-slate-600 dark:text-slate-400">
                  {formatDate(tenant.created_at)}
                </span>
              </div>
            </div>

            {/* Actions */}
            <div className="mt-4 flex items-center justify-end gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
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
          </div>
        )}
        renderDesktopTable={() => (
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] overflow-hidden shadow-sm">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="bg-slate-50 dark:bg-slate-800/10">
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("system_admin.tenants.table.id")}</th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("system_admin.tenants.table.name", { defaultValue: "Name" })}</th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("system_admin.tenants.table.byod", { defaultValue: "Type" })}</th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("system_admin.tenants.table.timezone", { defaultValue: "Timezone" })}</th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("system_admin.tenants.table.created_at")}</th>
                  <th className="px-8 py-5 text-right text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">
                    {t("system_admin.tenants.table.actions")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {tenants.map((tenant) => (
                  <tr className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/50 dark:hover:bg-slate-800/10 transition-colors" key={tenant.id}>
                    <td className="px-8 py-6 font-mono text-xs">{tenant.id}</td>
                    <td className="px-8 py-6 text-sm font-medium">{tenant.name}</td>
                    <td className="px-8 py-6">
                      <StatusBadge variant={tenant.is_byod ? "success" : "neutral"}>
                        {tenant.is_byod ? "BYOD" : "Standard"}
                      </StatusBadge>
                    </td>
                    <td className="px-8 py-6 text-sm text-slate-500">{tenant.timezone}</td>
                    <td className="px-8 py-6 text-sm text-slate-500">
                      {formatDateTimeLocale(tenant.created_at)}
                    </td>
                    <td className="px-8 py-6 text-right">
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
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      />

      {/* Modals */}
      <CreateTenantModal
        isOpen={createModalOpen}
        isPending={false}
        onClose={() => {
          setCreateModalOpen(false);
        }}
        onSuccess={() => {
          void queryClient.invalidateQueries({ queryKey: ["system-tenants"] });
          setCreateModalOpen(false);
        }}
      />

      <TenantStatsModal
        isLoading={isStatsLoading}
        isOpen={statsModalOpen}
        onClose={() => {
          setStatsModalOpen(false);
          setSelectedTenant(null);
        }}
        selectedTenant={selectedTenant}
        tenantStats={tenantStats}
      />

      <TenantDbUpdateModal
        isOpen={dbUpdateModalOpen}
        isPending={updateDbMutation.isPending}
        isStatsLoading={isStatsLoading}
        onClose={() => {
          setDbUpdateModalOpen(false);
          setSelectedTenant(null);
        }}
        onSubmit={handleUpdateDbSubmit}
        selectedTenant={selectedTenant}
        tenantStats={tenantStats}
      />

      <ConfirmationDialog />
    </div>
  );
}
