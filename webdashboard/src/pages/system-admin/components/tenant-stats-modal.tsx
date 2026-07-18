import { Activity, KeyRound, ShieldAlert } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import type { SystemTenant, TenantConnectionStats } from "@/lib/api/system";

interface TenantStatsModalProps {
  isOpen: boolean;
  isLoading: boolean;
  selectedTenant: SystemTenant | null;
  tenantStats: TenantConnectionStats | undefined;
  onClose: () => void;
}

function renderConnectionTemplate(
  stats: TenantConnectionStats | undefined,
  tenantId: string,
): string | null {
  if (!stats) return null;
  if (stats.connection.masked_db_url) {
    return stats.connection.masked_db_url;
  }
  const config = stats.config.tenant ?? stats.config;
  const dbUser = config.users?.[0]?.db_user ?? "user";
  const dbName = config.db_database ?? "db";
  const host = stats.connection.host;
  const port = stats.connection.port;

  // Masked connection string (Rule 2: Never log PII/Passwords)
  return `postgres://${dbUser}.${tenantId}:***@${host}:${port}/${dbName}`;
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

function PoolUsersList({ users }: { users: Array<{ db_user?: string; pool_size?: number }> }) {
  if (users.length === 0) {
    return (
      <div className="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700 text-sm text-slate-500 text-center">
        No explicit users configured for this tenant.
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 gap-3">
      {users.map((u, idx) => (
        <div
          className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-4 rounded-xl shadow-sm flex items-center justify-between"
          key={idx}
        >
          <div>
            <div className="font-mono text-sm text-slate-800 dark:text-slate-200 font-semibold flex items-center gap-2">
              <KeyRound className="w-4 h-4 text-slate-400" />
              {u.db_user}
            </div>
            <div className="text-xs text-slate-500 mt-1">
              Pool size:{" "}
              <span className="font-medium text-slate-700 dark:text-slate-300">
                {u.pool_size ?? "N/A"}
              </span>
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
  );
}

export function TenantStatsModal({
  isOpen,
  isLoading,
  selectedTenant,
  tenantStats,
  onClose,
}: TenantStatsModalProps) {
  const statsConfig = tenantStats
    ? (tenantStats.config.data ?? tenantStats.config.tenant ?? tenantStats.config)
    : undefined;
  const poolUsers: Array<{ db_user?: string; pool_size?: number }> = statsConfig?.users ?? [];
  const rawConfigString = JSON.stringify(statsConfig, null, 2);

  return (
    <Modal
      isOpen={isOpen}
      maxWidth="2xl"
      onClose={onClose}
      subtitle="Connection details and Supavisor configuration"
      title={`Tenant Details: ${selectedTenant?.id ?? ""}`}
    >
      <div className="py-4 space-y-6">
        {isLoading ? (
          <StatsSkeleton />
        ) : tenantStats ? (
          <>
            {/* Connection String Card */}
            <div className="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700">
              <div className="flex items-center gap-2 mb-3">
                <KeyRound className="w-4 h-4 text-primary-500" />
                <Typography className="text-sm font-semibold text-slate-800 dark:text-slate-200" variant="h4">
                  Connection String
                </Typography>
              </div>
              <div className="bg-slate-900 rounded-lg p-3 overflow-x-auto">
                <code className="text-xs text-green-400 whitespace-nowrap">
                  {renderConnectionTemplate(tenantStats, selectedTenant?.id ?? "tenant_id")}
                </code>
              </div>
              <Typography className="text-xs text-slate-500 mt-2" variant="p">
                * Password is masked for security reasons. Replace *** with the actual tenant database password.
              </Typography>
            </div>

            {/* Users Config */}
            <div className="space-y-3">
              <Typography
                className="text-sm font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2"
                variant="h4"
              >
                <Activity className="w-4 h-4 text-blue-500" />
                Database Users & Pool Modes
              </Typography>
              <PoolUsersList users={poolUsers} />
            </div>

            {/* Raw Details */}
            <div className="pt-4 border-t border-slate-200 dark:border-slate-800">
              <Typography
                className="text-sm font-semibold text-slate-800 dark:text-slate-200 mb-3 flex items-center gap-2"
                variant="h4"
              >
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
        <Button onClick={onClose} variant="outline">
          Close
        </Button>
      </div>
    </Modal>
  );
}
