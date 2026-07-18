import { SearchX, ShieldAlert, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import type { SystemTenant } from "@/lib/api/system";

interface StaleTenantsModalProps {
  isOpen: boolean;
  isLoading: boolean;
  isCleaning: boolean;
  staleTenants: SystemTenant[];
  onClose: () => void;
  onCleanOne: (id: string) => void;
  onCleanAll: () => void;
}

function StaleTenantsSkeleton() {
  const { t } = useTranslation(["common", "system_roles"]);
  return (
    <div className="space-y-3">
      <div className="h-12 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-lg" />
      <div className="h-12 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-lg" />
      <div className="h-12 bg-slate-100 dark:bg-slate-800 animate-pulse rounded-lg" />
      <Typography className="text-xs text-center text-slate-400" variant="p">
        {t("system_admin.tenants.stale_modal.scanning")}
      </Typography>
    </div>
  );
}

function EmptyStaleState() {
  const { t } = useTranslation(["common", "system_roles"]);
  return (
    <div className="text-center py-10">
      <div className="w-16 h-16 rounded-full bg-emerald-50 dark:bg-emerald-900/20 flex items-center justify-center mx-auto mb-4">
        <SearchX className="w-8 h-8 text-emerald-500" />
      </div>
      <Typography className="text-slate-600 dark:text-slate-300 font-medium" variant="p">
        {t("system_admin.tenants.stale_modal.no_stale")}
      </Typography>
    </div>
  );
}

export function StaleTenantsModal({
  isOpen,
  isLoading,
  isCleaning,
  staleTenants,
  onClose,
  onCleanOne,
  onCleanAll,
}: StaleTenantsModalProps) {
  const { t } = useTranslation(["common", "system_roles"]);

  return (
    <Modal
      isOpen={isOpen}
      maxWidth="2xl"
      onClose={onClose}
      subtitle={t("system_admin.tenants.stale_modal.subtitle")}
      title={t("system_admin.tenants.stale_modal.title")}
    >
      <div className="py-4 space-y-4">
        {isLoading ? (
          <StaleTenantsSkeleton />
        ) : staleTenants.length === 0 ? (
          <EmptyStaleState />
        ) : (
          <>
            <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-3 flex gap-3 items-center">
              <ShieldAlert className="w-5 h-5 text-amber-600 dark:text-amber-400 shrink-0" />
              <Typography className="text-sm text-amber-800 dark:text-amber-200" variant="p">
                {staleTenants.length} stale configuration
                {staleTenants.length > 1 ? "s" : ""} detected.
              </Typography>
            </div>
            <div className="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
              {staleTenants.map((tenant) => (
                <div
                  className="flex items-center justify-between px-4 py-3 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
                  key={tenant.id}
                >
                  <div>
                    <Typography
                      className="font-mono text-xs text-slate-700 dark:text-slate-300"
                      variant="p"
                    >
                      {tenant.id}
                    </Typography>
                    <Typography className="text-xs text-slate-400 mt-0.5" variant="p">
                      Pool: {tenant.pool_size} &middot; Created:{" "}
                      {new Date(tenant.created_at).toLocaleDateString()}
                    </Typography>
                  </div>
                  <Button
                    disabled={isCleaning}
                    onClick={() => { onCleanOne(tenant.id); }}
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
              disabled={isCleaning}
              onClick={() => { onCleanAll(); }}
              variant="destructive"
            >
              {t("system_admin.tenants.stale_modal.clean_all")}
            </Button>
          )}
        </div>
        <Button onClick={onClose} variant="outline">
          Close
        </Button>
      </div>
    </Modal>
  );
}
