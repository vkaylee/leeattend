import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Database, Power, PowerOff } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import PageHeader from "@/components/ui/page-header";
import StatusBadge from "@/components/ui/status-badge";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import {
  createSystemDbCluster,
  deleteSystemDbCluster,
  fetchSystemDbClusters,
  updateSystemDbClusterStatus,
  type CreateDbClusterRequestPayload,
  type SystemDbCluster,
} from "@/lib/api/system";

// eslint-disable-next-line max-lines-per-function -- Page component needs to hold form and state
export default function DbClustersPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { ConfirmationDialog, confirm } = useConfirm();
  const toast = useToast();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [formData, setFormData] = useState<CreateDbClusterRequestPayload>({
    name: "",
    admin_database_url: "",
    region: "",
    max_tenants: 100,
    is_active: true,
  });

  const { data: clusters = [], isLoading } = useQuery({
    queryKey: ["system-db-clusters"],
    queryFn: fetchSystemDbClusters,
  });

  const createMutation = useMutation({
    mutationFn: (data: CreateDbClusterRequestPayload) => createSystemDbCluster(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["system-db-clusters"] }).catch(console.error);
      toast.success(t("common.success", { defaultValue: "Successfully created DB cluster" }));
      setIsModalOpen(false);
      setFormData({
        name: "",
        admin_database_url: "",
        region: "",
        max_tenants: 100,
        is_active: true,
      });
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteSystemDbCluster(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["system-db-clusters"] }).catch(console.error);
      toast.success(t("common.success", { defaultValue: "Successfully deleted DB cluster" }));
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  const updateStatusMutation = useMutation({
    mutationFn: ({ id, isActive }: { id: string; isActive: boolean }) => updateSystemDbClusterStatus(id, isActive),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["system-db-clusters"] }).catch(console.error);
      toast.success(t("common.success", { defaultValue: "Successfully updated cluster status" }));
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  const handleCreate = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!formData.name || !formData.admin_database_url) return;
    createMutation.mutate(formData);
  };

  const handleDelete = async (cluster: SystemDbCluster) => {
    // Enterprise Standard: Hard block deletion if active tenants exist
    if (cluster.active_tenants > 0) {
      toast.error("Cannot delete a cluster with active tenants. Please migrate tenants first.");
      return;
    }

    const isConfirmed = await confirm({
      title: "Confirm Cluster Deletion",
      message: `Are you absolutely sure you want to delete the cluster "${cluster.name}"? This action cannot be undone.`,
      variant: "danger",
    });

    if (isConfirmed) {
      deleteMutation.mutate(cluster.id);
    }
  };

  const toggleStatus = async (cluster: SystemDbCluster) => {
    const action = cluster.is_active ? "disable" : "enable";
    const isConfirmed = await confirm({
      title: `Confirm ${action}`,
      message: `Are you sure you want to ${action} the cluster "${cluster.name}"? ${cluster.is_active ? "New tenants will no longer be provisioned here." : "New tenants will be provisioned here."}`,
      variant: "warning",
    });

    if (isConfirmed) {
      updateStatusMutation.mutate({ id: cluster.id, isActive: !cluster.is_active });
    }
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          subtitle="Manage Database Clusters across regions and monitor tenant capacities."
          title="Database Clusters"
        />
        <div className="flex items-center gap-3">
          <Button onClick={() => { setIsModalOpen(true); }}>
            <Plus className="w-4 h-4 mr-2" />
            Add Cluster
          </Button>
        </div>
      </div>

      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {[1, 2, 3].map((i) => (
            <div className="bg-slate-50 dark:bg-slate-800 animate-pulse rounded-xl h-64" key={i}></div>
          ))}
        </div>
      ) : clusters.length === 0 ? (
        <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-12 text-center shadow-sm">
          <Database className="w-12 h-12 text-slate-300 dark:text-slate-700 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-slate-900 dark:text-white mb-2">No Database Clusters</h3>
          <p className="text-slate-500 mb-6">You have not configured any database clusters yet.</p>
          <Button onClick={() => { setIsModalOpen(true); }}>
            <Plus className="w-4 h-4 mr-2" />
            Add Your First Cluster
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {clusters.map((cluster) => {
            const utilization = Math.round((cluster.active_tenants / cluster.max_tenants) * 100) || 0;
            const isFull = utilization >= 100;
            const progressColor = isFull ? "bg-red-500" : utilization > 80 ? "bg-amber-500" : "bg-emerald-500";

            return (
              <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm hover:shadow-md transition-shadow" key={cluster.id}>
                <div className="p-5 border-b border-slate-100 dark:border-slate-800 flex justify-between items-start">
                  <div>
                    <h3 className="font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
                      <Database className="w-4 h-4 text-primary-500" />
                      {cluster.name}
                    </h3>
                    <p className="text-xs text-slate-500 mt-1">Region: {cluster.region}</p>
                  </div>
                  <StatusBadge variant={cluster.is_active ? "success" : "neutral"}>
                    {cluster.is_active ? "Active" : "Inactive"}
                  </StatusBadge>
                </div>
                
                <div className="p-5 space-y-5">
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-slate-500">Utilization</span>
                      <span className="font-medium text-slate-700 dark:text-slate-300">{utilization}%</span>
                    </div>
                    <div className="w-full bg-slate-100 dark:bg-slate-800 rounded-full h-2.5">
                      <div className={`h-2.5 rounded-full ${progressColor}`} style={{ width: `${Math.min(utilization, 100)}%` }}></div>
                    </div>
                    <p className="text-xs text-slate-500 text-right">
                      {cluster.active_tenants} / {cluster.max_tenants} tenants
                    </p>
                  </div>
                </div>

                <div className="bg-slate-50 dark:bg-slate-800/50 p-4 flex justify-end gap-2 border-t border-slate-100 dark:border-slate-800">
                  <Button
                    onClick={() => { toggleStatus(cluster).catch(console.error); }}
                    size="sm"
                    title={cluster.is_active ? "Deactivate" : "Activate"}
                    variant="ghost"
                  >
                    {cluster.is_active ? <PowerOff className="w-4 h-4 text-slate-500" /> : <Power className="w-4 h-4 text-emerald-500" />}
                  </Button>
                  <Button
                    onClick={() => { handleDelete(cluster).catch(console.error); }}
                    size="sm"
                    title="Delete Cluster"
                    variant="ghost"
                  >
                    <Trash2 className="w-4 h-4 text-red-500" />
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      <Modal
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); }}
        subtitle="Add a new database cluster for provisioning tenants."
        title="Create Database Cluster"
      >
        <form onSubmit={handleCreate}>
          <div className="grid gap-5 py-4">
            <div className="space-y-2">
              <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="clusterName">
                Cluster Name <span className="text-red-500">*</span>
              </label>
              <input
                className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 dark:text-white"
                id="clusterName"
                onChange={(e) => { setFormData({ ...formData, name: e.target.value }); }}
                placeholder="e.g. US East Primary"
                required
                value={formData.name}
              />
            </div>
            
            <div className="space-y-2">
              <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="adminDbUrl">
                Admin Database URL <span className="text-red-500">*</span>
              </label>
              <input
                className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 dark:text-white"
                id="adminDbUrl"
                onChange={(e) => { setFormData({ ...formData, admin_database_url: e.target.value }); }}
                placeholder="postgres://admin:pass@host:5432/postgres"
                required
                value={formData.admin_database_url}
              />
              <p className="text-xs text-slate-500">Must be a highly privileged account to create databases.</p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="region">
                  Region
                </label>
                <input
                  className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 dark:text-white"
                  id="region"
                  onChange={(e) => { setFormData({ ...formData, region: e.target.value }); }}
                  placeholder="us-east-1"
                  value={formData.region}
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="maxTenants">
                  Max Tenants <span className="text-red-500">*</span>
                </label>
                <input
                  className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 dark:text-white"
                  id="maxTenants"
                  min="1"
                  onChange={(e) => { setFormData({ ...formData, max_tenants: parseInt(e.target.value, 10) }); }}
                  required
                  type="number"
                  value={formData.max_tenants}
                />
              </div>
            </div>

            <div className="flex items-center justify-between bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg border border-slate-200 dark:border-slate-700">
              <label className="text-sm font-medium text-slate-700 dark:text-slate-300 cursor-pointer select-none" htmlFor="autoProvision">
                Enable for Auto-provisioning
              </label>
              <input
                checked={formData.is_active}
                className="h-5 w-5 rounded border-slate-300 text-primary-600 focus:ring-primary-600 bg-white"
                id="autoProvision"
                onChange={(e) => { setFormData({ ...formData, is_active: e.target.checked }); }}
                type="checkbox"
              />
            </div>
          </div>
          
          <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800 mt-2">
            <Button
              onClick={() => { setIsModalOpen(false); }}
              type="button"
              variant="outline"
            >
              Cancel
            </Button>
            <Button
              disabled={createMutation.isPending || !formData.name || !formData.admin_database_url}
              loading={createMutation.isPending}
              type="submit"
              variant="primary"
            >
              {createMutation.isPending ? "Creating..." : "Create Cluster"}
            </Button>
          </div>
        </form>
      </Modal>

      <ConfirmationDialog />
    </div>
  );
}
