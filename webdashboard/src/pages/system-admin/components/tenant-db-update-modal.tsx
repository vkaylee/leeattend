import { Globe, ShieldAlert } from "lucide-react";
import React, { useState } from "react";

import type { SystemTenant, TenantConnectionStats } from "@/lib/api/system";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";

interface TenantDbUpdateModalProps {
  isOpen: boolean;
  isPending: boolean;
  isStatsLoading: boolean;
  onClose: () => void;
  onSubmit: (id: string, url: string) => Promise<void>;
  selectedTenant: SystemTenant | null;
  tenantStats: TenantConnectionStats | undefined;
}

export function TenantDbUpdateModal({
  isOpen,
  isPending,
  isStatsLoading,
  selectedTenant,
  tenantStats,
  onClose,
  onSubmit,
}: TenantDbUpdateModalProps) {
  const [updateDbUrl, setUpdateDbUrl] = useState("");

  const handleClose = () => {
    setUpdateDbUrl("");
    onClose();
  };

  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!selectedTenant || !updateDbUrl) return;
    await onSubmit(selectedTenant.id, updateDbUrl);
    setUpdateDbUrl("");
  };

  return (
    <Modal
      isOpen={isOpen}
      maxWidth="2xl"
      onClose={handleClose}
      subtitle="Update the custom BYOD database URL for this tenant."
      title={`Update Database: ${selectedTenant?.name ?? selectedTenant?.id ?? ""}`}
    >
      <form onSubmit={(e) => { void handleSubmit(e); }}>
        <div className="grid gap-6 py-4">
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 flex gap-3">
            <ShieldAlert className="w-5 h-5 text-red-600 dark:text-red-400 shrink-0" />
            <div className="text-sm text-red-800 dark:text-red-200">
              <Typography className="font-semibold mb-1" variant="p">
                Critical Warning
              </Typography>
              <Typography variant="p">
                This operation updates the connection URL for this tenant.
                <strong> It DOES NOT copy or migrate data.</strong> You must ensure the target database
                already contains the necessary schema and data for this tenant.
              </Typography>
            </div>
          </div>

          <div className="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700">
            <div className="flex items-center gap-2 mb-3">
              <Globe className="w-4 h-4 text-primary-500" />
              <Typography
                className="text-sm font-semibold text-slate-800 dark:text-slate-200"
                variant="h4"
              >
                Current Database URL
              </Typography>
            </div>
            <div className="bg-slate-900 rounded-lg p-3 overflow-x-auto min-h-[44px] flex items-center">
              {isStatsLoading ? (
                <Skeleton className="h-4 w-64 bg-slate-800" />
              ) : (
                <code className="text-xs text-green-400 whitespace-nowrap">
                  {tenantStats?.masked_db_url ?? "Not configured"}
                </code>
              )}
            </div>
          </div>

          <div className="space-y-2">
            <label
              className="text-sm font-medium text-slate-700 dark:text-slate-300"
              htmlFor="updateDbUrl"
            >
              New Custom Database URL <span className="text-red-500">*</span>
            </label>
            <input
              className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent dark:text-white"
              id="updateDbUrl"
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                setUpdateDbUrl(e.target.value);
              }}
              placeholder="postgres://user:password@host:5432/db"
              required
              type="text"
              value={updateDbUrl}
            />
          </div>
        </div>
        <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800 mt-2">
          <Button onClick={handleClose} type="button" variant="outline">
            Cancel
          </Button>
          <Button
            disabled={isPending || !updateDbUrl}
            loading={isPending}
            type="submit"
            variant="destructive"
          >
            {isPending ? "Updating..." : "Update Database"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
