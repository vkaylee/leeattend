import { Building2, Globe } from "lucide-react";
import React from "react";

import type { SystemTenant, TenantConnectionStats } from "@/lib/api/system";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

interface TenantStatsModalProps {
  isLoading: boolean;
  isOpen: boolean;
  onClose: () => void;
  selectedTenant: SystemTenant | null;
  tenantStats: TenantConnectionStats | undefined;
}

function StatsSkeleton() {
  return (
    <div className="space-y-4">
      <div className="h-24 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-xl" />
      <div className="grid grid-cols-2 gap-4">
        <div className="h-32 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-xl" />
        <div className="h-32 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-xl" />
      </div>
    </div>
  );
}

function WorkspaceInfoCard({ tenant }: { tenant: SystemTenant }) {
  const { formatDateTimeLocale } = useDateTimeFormatter();
  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700">
      <div className="flex items-center gap-2 mb-3">
        <Building2 className="w-4 h-4 text-primary-500" />
        <Typography className="text-sm font-semibold text-slate-800 dark:text-slate-200" variant="h4">
          Workspace Information
        </Typography>
      </div>
      <div className="grid grid-cols-2 gap-4 text-sm">
        <div>
          <span className="text-slate-500">Name</span>
          <Typography className="font-medium text-slate-800 dark:text-slate-200" variant="p">{tenant.name}</Typography>
        </div>
        <div>
          <span className="text-slate-500">ID</span>
          <Typography className="font-mono text-xs text-slate-800 dark:text-slate-200" variant="p">{tenant.id}</Typography>
        </div>
        <div>
          <span className="text-slate-500">Type</span>
          <Typography variant="p">
            <StatusBadge variant={tenant.is_byod ? "success" : "neutral"}>
              {tenant.is_byod ? "BYOD" : "Standard"}
            </StatusBadge>
          </Typography>
        </div>
        <div>
          <span className="text-slate-500">Timezone</span>
          <Typography className="font-medium text-slate-800 dark:text-slate-200" variant="p">{tenant.timezone}</Typography>
        </div>
        <div>
          <span className="text-slate-500">Created</span>
          <Typography className="font-medium text-slate-800 dark:text-slate-200" variant="p">
            {formatDateTimeLocale(tenant.created_at)}
          </Typography>
        </div>
      </div>
    </div>
  );
}

function ConnectionDetailsCard({ stats }: { stats: TenantConnectionStats | undefined }) {
  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700">
      <div className="flex items-center gap-2 mb-3">
        <Globe className="w-4 h-4 text-blue-500" />
        <Typography className="text-sm font-semibold text-slate-800 dark:text-slate-200" variant="h4">
          Connection Details
        </Typography>
      </div>
      {stats ? (
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-slate-500">Host</span>
            <Typography className="font-mono text-xs text-slate-800 dark:text-slate-200" variant="p">{stats.connection.host}</Typography>
          </div>
          <div>
            <span className="text-slate-500">Port</span>
            <Typography className="font-mono text-xs text-slate-800 dark:text-slate-200" variant="p">{stats.connection.port}</Typography>
          </div>
          {stats.masked_db_url ? (
            <div className="col-span-2">
              <span className="text-slate-500">Database URL</span>
              <div className="bg-slate-900 rounded-lg p-3 mt-1 overflow-x-auto">
                <code className="text-xs text-green-400 whitespace-nowrap">
                  {stats.masked_db_url}
                </code>
              </div>
            </div>
          ) : (
            <div className="col-span-2">
              <Typography className="text-xs text-slate-500" variant="p">
                Standard tenant — database URL is not exposed.
              </Typography>
            </div>
          )}
        </div>
      ) : (
        <Typography className="text-xs text-slate-500" variant="p">
          Connection details not available.
        </Typography>
      )}
    </div>
  );
}

export function TenantStatsModal({
  isOpen,
  isLoading,
  selectedTenant,
  tenantStats,
  onClose,
}: TenantStatsModalProps) {
  return (
    <Modal
      isOpen={isOpen}
      maxWidth="2xl"
      onClose={onClose}
      subtitle="Workspace tenant connection details"
      title={`Tenant Details: ${selectedTenant?.name ?? selectedTenant?.id ?? ""}`}
    >
      <div className="py-4 space-y-6">
        {isLoading ? (
          <StatsSkeleton />
        ) : selectedTenant ? (
          <>
            <WorkspaceInfoCard tenant={selectedTenant} />
            <ConnectionDetailsCard stats={tenantStats} />
          </>
        ) : (
          <div className="text-center py-8 text-slate-500">
            Failed to load tenant details.
          </div>
        )}
      </div>
      <div className="flex justify-end pt-4 border-t border-slate-100 dark:border-slate-800">
        <Button onClick={onClose} variant="outline">
          Close
        </Button>
      </div>
    </Modal>
  );
}
