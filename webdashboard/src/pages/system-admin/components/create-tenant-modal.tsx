import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import {
  createSystemTenant,
  type CreateTenantRequestPayload,
} from "@/lib/api/system";

interface CreateTenantModalProps {
  isOpen: boolean;
  isPending: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export function CreateTenantModal({
  isOpen,
  isPending,
  onClose,
  onSuccess,
}: CreateTenantModalProps) {
  const { t } = useTranslation(["common", "system_roles"]);
  const toast = useToast();

  const [dbName, setDbName] = useState("");
  const [createDb, setCreateDb] = useState(true);
  const [poolSize, setPoolSize] = useState("15");

  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!dbName) return;

    const payload: CreateTenantRequestPayload = {
      db_name: dbName,
      create_db: createDb,
      pool_size: poolSize ? parseInt(poolSize, 10) : undefined,
    };

    try {
      await createSystemTenant(payload);
      toast.success(t("system_admin.tenants.messages.create_success"));
      resetForm();
      onSuccess();
    } catch (error) {
      toast.error((error as Error).message);
    }
  };

  const resetForm = () => {
    setDbName("");
    setCreateDb(true);
    setPoolSize("15");
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      subtitle="Provide database credentials to initialize a new connection pool for Supavisor."
      title={t("system_admin.tenants.modal.create_title")}
    >
      <form onSubmit={(e) => { void handleSubmit(e); }}>
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
            onClick={handleClose}
            type="button"
            variant="outline"
          >
            {t("common.cancel", { defaultValue: "Cancel" })}
          </Button>
          <Button
            disabled={isPending || !dbName}
            loading={isPending}
            type="submit"
            variant="primary"
          >
            {isPending
              ? t("system_admin.tenants.modal.creating")
              : t("system_admin.tenants.modal.submit_btn")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
