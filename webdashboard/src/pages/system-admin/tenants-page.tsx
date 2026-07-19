import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Eye, Server, SearchX } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";
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
import { useConfirm } from "@/hooks/use-confirm";
import {
  deleteSystemTenant,
  fetchStaleTenants,
  fetchSystemTenants,
  fetchSystemTenantStats,
  updateTenantDatabase,
  type SystemTenant,
  type TenantConnectionStats,
} from "@/lib/api/system";

import { CreateTenantModal } from "./components/create-tenant-modal";
import { StaleTenantsModal } from "./components/stale-tenants-modal";
import { TenantDbUpdateModal } from "./components/tenant-db-update-modal";
import { TenantStatsModal } from "./components/tenant-stats-modal";

  // eslint-disable-next-line max-lines-per-function
export default function TenantsPage() {
  const { t } = useTranslation(["common", "system_roles"]);
  const queryClient = useQueryClient();
  const { ConfirmationDialog, confirm } = useConfirm();
  const toast = useToast();

  // ── Modal visibility ──────────────────────────────────────────────────────
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [statsModalOpen, setStatsModalOpen] = useState(false);
  const [dbUpdateModalOpen, setDbUpdateModalOpen] = useState(false);
  const [staleModalOpen, setStaleModalOpen] = useState(false);
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

  const {
    data: staleTenants = [],
    isLoading: isStaleLoading,
    refetch: refetchStale,
  } = useQuery({
    queryKey: ["system-tenants-stale"],
    queryFn: fetchStaleTenants,
    enabled: staleModalOpen,
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
              setStaleModalOpen(true);
              void refetchStale();
            }}
            variant="outline"
          >
            <SearchX className="w-4 h-4 mr-2" />
            {t("system_admin.tenants.check_stale_btn")}
          </Button>
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
                  <div className="w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full animate-spin" />
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
                      onClick={() => {
                        handleViewStats(tenant);
                      }}
                      size="sm"
                      title="View Connection & Stats"
                      variant="ghost"
                    >
                      <Eye className="w-4 h-4 text-slate-500" />
                    </Button>
                    <Button
                      onClick={() => {
                        handleManageDb(tenant);
                      }}
                      size="sm"
                      title="Manage Database"
                      variant="ghost"
                    >
                      <Server className="w-4 h-4 text-orange-500" />
                    </Button>
                    <Button
                      onClick={() => {
                        void handleDelete(tenant.id);
                      }}
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

      <StaleTenantsModal
        isCleaning={cleanStaleMutation.isPending}
        isLoading={isStaleLoading}
        isOpen={staleModalOpen}
        onCleanAll={() => {
          void handleCleanAllStale();
        }}
        onCleanOne={(id) => {
          void handleCleanStale(id);
        }}
        onClose={() => {
          setStaleModalOpen(false);
        }}
        staleTenants={staleTenants}
      />

      <ConfirmationDialog />
    </div>
  );
}
